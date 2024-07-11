require 'json'
require 'logger'
require 'tmpdir'
require 'optparse'
require_relative 'lib/tester'

# Parse options
option_parser = OptionParser.new do |opts|
  opts.on('--basedir path') { |v| @basedir = v }
  opts.on('--collection-path path') { |v| @collection_path = v }
  opts.on('--tempdir path') { |v| @tempdir = v }
  opts.on('--output-result-path path') { |v| @output_result_path = v }
end
option_parser.parse!(ARGV)

if !@collection_path || !@output_result_path
  puts option_parser.help
  exit 1
end

unless @tempdir
  @tempdir = Dir.mktmpdir
  at_exit { FileUtils.remove_entry_secure(@tempdir) }
end

@logger = Logger.new(STDERR)

collection = JSON.parse(File.read(@collection_path))

# Set basedir if not set
@basedir ||= collection['base_path']

total_results = []
all_ok = true

# TODO: 並列化。並列化するときは、tmpdirもそれぞれのスレッドで作成する必要がある
collection['source_sets'].each do |source_set|
  lang = source_set['lang']
  @logger.info("lang: #{lang}")
  tester = Tester.select_tester(lang)
  unless tester
    @logger.warn("Unsupported language: #{lang}")
    next
  end

  src_results = source_set['src_paths'].map do |src_path|
    true_src_path = File.join(@basedir, src_path)
    @logger.info("check_compile #{true_src_path}")
    result = tester.check_compile(true_src_path, @basedir, @tempdir)
    {
      'path' => src_path,
      'type' => 'check_compile',
      'results' => result.map do |k, v|
                     {
                       'option' => k, 'ok' => v[:ok], 'timedout' => v[:timedout],
                       'stdout' => v[:stdout], 'stderr' => v[:stderr]
                     }
                   end
    }
  end

  test_results = source_set['test_paths'].map do |test_path|
    true_test_path = File.join(@basedir, test_path)
    @logger.info("run_test #{true_test_path}")
    result = tester.run_test(true_test_path, @basedir, @tempdir)
    {
      'path' => test_path,
      'type' => 'run_test',
      'results' => result.map do |k, v|
                     {
                       'option' => k, 'ok' => v[:ok], 'timedout' => v[:timedout],
                       'stdout' => v[:stdout], 'stderr' => v[:stderr]
                     }
                   end
    }
  end

  total_results << {
    'lang' => lang,
    'src_results' => src_results,
    'test_results' => test_results
  }
end

File.write(@output_result_path, JSON.generate(total_results))
