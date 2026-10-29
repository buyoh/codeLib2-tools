require 'json'
require 'optparse'

# Parse options
option_parser = OptionParser.new do |opts|
  opts.on('--result-path path') { |v| @result_path = v }
  opts.on('--check-test-fail') { |_v| @check_test_fail = true }
end
option_parser.parse!(ARGV)

unless @result_path
  puts option_parser.help
  exit 1
end

# -----------------------------------------------------------------------------

results_json = JSON.parse(File.read(@result_path))

results_json = results_json.map do |result|
  lang = result['lang']
  src_results = result['src_results']
  test_results = result['test_results']

  src_failed_results, src_passed_results = src_results.partition do |src_result|
    src_result['results'].any? { |r| !r['ok'] }
  end

  test_failed_results, test_passed_results = test_results.partition do |test_result|
    test_result['results'].any? { |r| !r['ok'] }
  end

  {
    'lang' => lang,
    'src_failed_results' => src_failed_results,
    'src_passed_results' => src_passed_results,
    'test_failed_results' => test_failed_results,
    'test_passed_results' => test_passed_results
  }
end

# -----------------------------------------------------------------------------

def print_with_indent(str, indent)
  str.each_line do |line|
    puts ' ' * indent + line
  end
end

def dump_result(results_list)
  path = results_list['path']
  type = results_list['type']
  results = results_list['results']
  puts "- [#{type}] `#{path}`"
  results.each do |r|
    option = r['option']
    ok = r['ok']
    timedout = r['timedout']
    stdout = r['stdout']
    stderr = r['stderr']

    puts "  - #{option}: #{ok ? 'OK' : 'NG'}"
    next if ok

    puts "    - timedout: #{timedout ? 'YES' : 'NO'}"
    if stdout.to_s.empty?
      puts '    - stdout: (empty)'
    else
      puts '    - stdout:'
      print_with_indent("```\n#{stdout}\n```", 6)
    end
    if stderr.to_s.empty?
      puts '    - stderr: (empty)'
    else
      puts '    - stderr: '
      print_with_indent("```\n#{stderr}\n```", 6)
    end
  end
end

puts '# Results'
puts ''
puts "scripts: #{$0}"
puts ''

puts "## Failed\n"

results_json.each do |result|
  lang = result['lang']
  src_failed_results = result['src_failed_results']
  test_failed_results = result['test_failed_results']

  puts "### #{lang}\n"

  unless src_failed_results.empty?
    puts "#### Build results\n"
    src_failed_results.each { |src_result| dump_result(src_result) }
  end
  unless test_failed_results.empty?
    puts "#### Test results\n"
    test_failed_results.each { |test_result| dump_result(test_result) }
  end
end

puts "## Passed\n"

results_json.each do |result|
  lang = result['lang']
  src_passed_results = result['src_passed_results']
  test_passed_results = result['test_passed_results']

  puts "### #{lang}\n"

  unless src_passed_results.empty?
    puts "#### Build results\n"
    src_passed_results.each { |src_result| dump_result(src_result) }
  end
  unless test_passed_results.empty?
    puts "#### Test results\n"
    test_passed_results.each { |test_result| dump_result(test_result) }
  end
end

# Count failures independently of the output above
num_test_failed = results_json.sum do |result|
  result['src_failed_results'].size + result['test_failed_results'].size
end

abort "Some tests failed: count = #{num_test_failed}" if @check_test_fail && num_test_failed > 0
