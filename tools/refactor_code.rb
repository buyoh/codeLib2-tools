#!/usr/bin/env ruby
require 'json'
require 'logger'
require 'optparse'
require_relative 'lib/tester'

# require_relative './lib/collector/collector'
# require_relative './lib/code/codeparser'

option_parser = OptionParser.new do |opts|
  opts.on('--basedir path') { |v| @basedir = v }
  opts.on('--collection-path path') { |v| @collection_path = v }
  opts.on('--inplace') { |_v| @inplace = true }
end
option_parser.parse!(ARGV)

unless @collection_path
  puts option_parser.help
  exit 1
end

@logger = Logger.new(STDERR)

collection = JSON.parse(File.read(@collection_path))
@basedir ||= collection['base_path']

success = true
# TODO: 並列化。並列化するときは、tmpdirもそれぞれのスレッドで作成する必要がある
collection['source_sets'].each do |source_set|
  lang = source_set['lang']
  tester = Tester.select_tester(lang)
  unless tester
    @logger.warn("Unsupported language: #{lang}")
    next
  end

  source_set['src_paths'].each do |src_path|
    true_src_path = File.join(@basedir, src_path)
    success &= tester.refactor_code(true_src_path, @inplace)
  end

  source_set['test_paths'].each do |test_path|
    true_test_path = File.join(@basedir, test_path)
    success &= tester.refactor_code(true_test_path, @inplace)
  end
end

abort 'refactor check failed' unless success
exit 0
