require 'minitest/autorun'
require 'json'
require 'open3'
require 'tmpdir'

class DumpResultToMarkdownTest < Minitest::Test
  SCRIPT = File.expand_path('../dump_result_to_markdown.rb', __dir__)

  def result(path, type, oks, stdout: '', stderr: '')
    {
      'path' => path,
      'type' => type,
      'results' => oks.map.with_index do |ok, i|
        { 'option' => "opt#{i}", 'ok' => ok, 'timedout' => false, 'stdout' => stdout, 'stderr' => stderr }
      end
    }
  end

  def run_dump(results, *args)
    Dir.mktmpdir do |dir|
      path = File.join(dir, 'result.json')
      File.write(path, JSON.generate(results))
      Open3.capture3('ruby', SCRIPT, '--result-path', path, *args)
    end
  end

  def test_all_passed
    results = [{ 'lang' => 'cpp',
                 'src_results' => [result('/src/a.hpp', 'check_compile', [true, true])],
                 'test_results' => [result('/test/a.cpp', 'run_test', [true, true])] }]
    _out, _err, status = run_dump(results, '--check-test-fail')
    assert status.success?
  end

  def test_src_failed
    results = [{ 'lang' => 'cpp',
                 'src_results' => [result('/src/a.hpp', 'check_compile', [true, false])],
                 'test_results' => [] }]
    _out, err, status = run_dump(results, '--check-test-fail')
    refute status.success?
    assert_match(/count = 1/, err)
  end

  def test_test_failed_while_all_src_passed
    results = [{ 'lang' => 'cpp',
                 'src_results' => [result('/src/a.hpp', 'check_compile', [true])],
                 'test_results' => [result('/test/a.cpp', 'run_test', [false], stderr: 'compile error')] }]
    out, err, status = run_dump(results, '--check-test-fail')
    refute status.success?
    assert_match(/count = 1/, err)
    failed_section = out.split('## Passed').first
    assert_includes failed_section, '/test/a.cpp'
    assert_includes failed_section, 'compile error'
  end

  def test_without_check_option_exits_zero_even_if_failed
    results = [{ 'lang' => 'cpp',
                 'src_results' => [],
                 'test_results' => [result('/test/a.cpp', 'run_test', [false])] }]
    _out, _err, status = run_dump(results)
    assert status.success?
  end

  def test_passed_tests_are_listed_while_all_src_failed
    results = [{ 'lang' => 'cpp',
                 'src_results' => [result('/src/a.hpp', 'check_compile', [false])],
                 'test_results' => [result('/test/a.cpp', 'run_test', [true])] }]
    out, _err, _status = run_dump(results)
    passed_section = out.split('## Passed').last
    assert_includes passed_section, '/test/a.cpp'
  end

  def test_nil_output_does_not_crash
    results = [{ 'lang' => 'cpp',
                 'src_results' => [result('/src/a.hpp', 'check_compile', [false], stdout: nil, stderr: nil)],
                 'test_results' => [] }]
    out, err, status = run_dump(results, '--check-test-fail')
    refute_match(/NoMethodError/, err)
    assert_includes out, 'stdout: (empty)'
    assert_equal 1, status.exitstatus
  end
end
