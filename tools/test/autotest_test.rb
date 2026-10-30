require 'minitest/autorun'
require 'json'
require 'open3'
require 'tmpdir'

class AutotestTest < Minitest::Test
  SCRIPT = File.expand_path('../autotest.rb', __dir__)

  def setup
    skip 'g++ not found' unless system('which', 'g++', out: File::NULL)
    @dir = Dir.mktmpdir
  end

  def teardown
    FileUtils.remove_entry_secure(@dir) if @dir
  end

  def write(path, content)
    full_path = File.join(@dir, path)
    FileUtils.mkdir_p(File.dirname(full_path))
    File.write(full_path, content)
  end

  def run_autotest(collection, *args)
    collection_path = File.join(@dir, 'collection.json')
    result_path = File.join(@dir, 'result.json')
    File.write(collection_path, JSON.generate(collection))
    _out, err, status = Open3.capture3('ruby', SCRIPT, '--collection-path', collection_path,
                                       '--output-result-path', result_path, *args)
    assert status.success?, err
    JSON.parse(File.read(result_path))
  end

  def test_parallel_results_keep_order
    src_paths = (0...6).map do |i|
      write("src/cpp/s#{i}.hpp", i == 3 ? "int f#{i}() { return }\n" : "inline int f#{i}() { return #{i}; }\n")
      "/src/cpp/s#{i}.hpp"
    end
    test_paths = (0...4).map do |i|
      write("test/cpp/t#{i}.cpp", "int main() { return #{i == 1 ? 1 : 0}; }\n")
      "/test/cpp/t#{i}.cpp"
    end
    collection = {
      'base_path' => @dir,
      'source_sets' => [
        { 'lang' => 'cpp', 'src_paths' => src_paths, 'test_paths' => test_paths },
        { 'lang' => 'go', 'src_paths' => ['/src/go/a.go'], 'test_paths' => ['/test/go/a.go'] }
      ]
    }

    results = run_autotest(collection, '--jobs', '3', '--tempdir', File.join(@dir, 'tmp'))

    assert_equal %w[cpp go], results.map { |r| r['lang'] }
    assert_equal true, results[1]['skipped']
    assert_equal ['/src/go/a.go', '/test/go/a.go'], results[1]['skipped_paths']
    cpp = results[0]
    assert_equal src_paths, cpp['src_results'].map { |r| r['path'] }
    assert_equal test_paths, cpp['test_results'].map { |r| r['path'] }
    assert(cpp['src_results'].all? { |r| r['type'] == 'check_compile' })
    assert(cpp['test_results'].all? { |r| r['type'] == 'run_test' })

    src_ok = cpp['src_results'].map { |r| r['results'].all? { |x| x['ok'] } }
    assert_equal [true, true, true, false, true, true], src_ok
    test_ok = cpp['test_results'].map { |r| r['results'].all? { |x| x['ok'] } }
    assert_equal [true, false, true, true], test_ok
  end

  def test_exception_in_job_is_recorded_as_failure
    write('src/cpp/a.hpp', "inline int a() { return 0; }\n")
    collection = {
      'base_path' => @dir,
      'source_sets' => [{ 'lang' => 'cpp', 'src_paths' => ["/src/cpp/bad\0.hpp", '/src/cpp/a.hpp'], 'test_paths' => [] }]
    }

    results = run_autotest(collection, '--jobs', '2')

    bad, ok = results[0]['src_results']
    assert_equal 'exception', bad['results'][0]['option']
    refute bad['results'][0]['ok']
    assert_match(/ArgumentError/, bad['results'][0]['stderr'])
    assert(ok['results'].all? { |r| r['ok'] })
  end

  def test_timeout_option
    write('test/cpp/t.cpp', "int main() { while (true) {} }\n")
    collection = {
      'base_path' => @dir,
      'source_sets' => [{ 'lang' => 'cpp', 'src_paths' => [], 'test_paths' => ['/test/cpp/t.cpp'] }]
    }

    results = run_autotest(collection, '--timeout-run', '0.5', '--jobs', '1')

    run = results[0]['test_results'][0]['results'].find { |r| r['option'] == 'run' }
    refute run['ok']
    assert run['timedout']
  end
end
