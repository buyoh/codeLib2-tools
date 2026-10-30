require 'etc'
require 'fileutils'
require 'json'
require 'logger'
require 'tmpdir'
require 'optparse'
require_relative 'lib/tester'

# Parse options
@jobs = Etc.nprocessors
option_parser = OptionParser.new do |opts|
  opts.on('--basedir path') { |v| @basedir = v }
  opts.on('--collection-path path') { |v| @collection_path = v }
  opts.on('--tempdir path') { |v| @tempdir = v }
  opts.on('--output-result-path path') { |v| @output_result_path = v }
  opts.on('-j', '--jobs num', Integer, 'number of parallel jobs (default: number of CPUs)') { |v| @jobs = v }
  opts.on('--timeout-build sec', Float) { |v| Tester::Cpp.timeout_build_sec = v }
  opts.on('--timeout-run sec', Float) { |v| Tester::Cpp.timeout_run_sec = v }
end
option_parser.parse!(ARGV)

if !@collection_path || !@output_result_path || @jobs < 1
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

def to_result_entries(result)
  result.map { |k, v| { 'option' => k, **v.transform_keys(&:to_s) } }
end

# Collect jobs
Job = Struct.new(:lang, :type, :path, :tester)
jobs = []
skipped_results = []
collection['source_sets'].each do |source_set|
  lang = source_set['lang']
  tester = Tester.select_tester(lang)
  unless tester
    @logger.warn("Unsupported language: #{lang}")
    skipped_results << {
      'lang' => lang,
      'skipped' => true,
      'skipped_paths' => source_set['src_paths'] + source_set['test_paths'],
      'src_results' => [],
      'test_results' => []
    }
    next
  end
  source_set['src_paths'].each { |path| jobs << Job.new(lang, 'check_compile', path, tester) }
  source_set['test_paths'].each { |path| jobs << Job.new(lang, 'run_test', path, tester) }
end

# Run jobs in parallel. Each worker has its own tempdir because run_test writes an executable.
queue = Queue.new
jobs.each_with_index { |job, i| queue << [job, i] }
queue.close
job_results = Array.new(jobs.size)

workers = Array.new([@jobs, jobs.size].min) do |worker_id|
  worker_tempdir = File.join(@tempdir, "worker#{worker_id}")
  FileUtils.mkdir_p(worker_tempdir)
  Thread.new do
    while (job, i = queue.pop)
      @logger.info("#{job.type} #{job.path}")
      result =
        begin
          true_path = File.join(@basedir, job.path)
          if job.type == 'check_compile'
            job.tester.check_compile(true_path, @basedir, worker_tempdir)
          else
            job.tester.run_test(true_path, @basedir, worker_tempdir)
          end
        rescue StandardError => e
          # Record as a failure not to lose other results
          @logger.error("#{job.type} #{job.path}: #{e.message}")
          { 'exception' => Runner::Result.new('', e.full_message(highlight: false), nil, false).to_h }
        end
      job_results[i] = { 'path' => job.path, 'type' => job.type, 'results' => to_result_entries(result) }
    end
  end
end
workers.each(&:join)

total_results = jobs.each_with_index.group_by { |job, _i| job.lang }.map do |lang, indexed_jobs|
  results = indexed_jobs.map { |_job, i| job_results[i] }
  {
    'lang' => lang,
    'src_results' => results.select { |r| r['type'] == 'check_compile' },
    'test_results' => results.select { |r| r['type'] == 'run_test' }
  }
end

File.write(@output_result_path, JSON.generate(total_results + skipped_results))
