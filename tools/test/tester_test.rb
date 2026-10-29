require 'minitest/autorun'
require 'tmpdir'
require_relative '../lib/tester'

class TesterCppTest < Minitest::Test
  def setup
    @dir = Dir.mktmpdir
    # Paths containing shell metacharacters must be passed as-is
    @src_dir = File.join(@dir, 'a b;touch injected')
    Dir.mkdir(@src_dir)
    @work_dir = File.join(@dir, 'tmp $(touch injected)')
    Dir.mkdir(@work_dir)
    # Fix the style regardless of .clang-format in ancestor directories
    File.write(File.join(@src_dir, '.clang-format'), "BasedOnStyle: LLVM\n")
  end

  def require_command(name)
    skip "#{name} not found" unless system('which', name, out: File::NULL)
  end

  def teardown
    FileUtils.remove_entry_secure(@dir)
  end

  def write_src(name, code)
    path = File.join(@src_dir, name)
    File.write(path, code)
    path
  end

  def assert_not_injected
    refute File.exist?(File.join(Dir.pwd, 'injected'))
    refute File.exist?(File.join(@dir, 'injected'))
  end

  def test_check_compile_with_special_path
    require_command('g++')
    path = write_src('x $(touch injected).hpp', "inline int f() { return 1; }\n")
    results = Tester::Cpp.check_compile(path, @src_dir, @work_dir)
    assert_equal %w[c++11 c++14 c++17 c++20], results.keys
    results.each_value { |r| assert r[:ok], r[:stderr] }
    assert_not_injected
  end

  def test_check_compile_error
    require_command('g++')
    path = write_src('error.hpp', "int f() { return }\n")
    results = Tester::Cpp.check_compile(path, @src_dir, @work_dir)
    results.each_value do |r|
      refute r[:ok]
      refute_empty r[:stderr]
    end
  end

  def test_run_test_with_special_path
    require_command('g++')
    path = write_src('t $(touch injected).cpp', "int main() { return 0; }\n")
    results = Tester::Cpp.run_test(path, @src_dir, @work_dir)
    assert results['compile'][:ok], results['compile'][:stderr]
    assert results['run'][:ok]
    assert_not_injected
  end

  def test_run_test_failure
    require_command('g++')
    path = write_src('t.cpp', "int main() { return 1; }\n")
    results = Tester::Cpp.run_test(path, @src_dir, @work_dir)
    assert results['compile'][:ok]
    refute results['run'][:ok]
  end

  def test_refactor_code_check_mode
    require_command('clang-format')
    formatted = write_src('ok $(touch injected).cpp', "int main() { return 0; }\n")
    unformatted = write_src('ng.cpp', "int   main(  ){return 0;}\n")
    assert Tester::Cpp.refactor_code(formatted, false)
    out, = capture_io { refute Tester::Cpp.refactor_code(unformatted, false) }
    assert_match(/clang-format assertion/, out)
    assert_equal "int   main(  ){return 0;}\n", File.read(unformatted)
    assert_not_injected
  end

  def test_refactor_code_inplace
    require_command('clang-format')
    path = write_src('ng $(touch injected).cpp', "int   main(  ){return 0;}\n")
    assert Tester::Cpp.refactor_code(path, true)
    assert_equal "int main() { return 0; }\n", File.read(path)
    assert_not_injected
  end
end
