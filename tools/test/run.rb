# Run all tests: ruby tools/test/run.rb
Dir.glob(File.join(__dir__, '*_test.rb')).sort.each { |f| require f }
