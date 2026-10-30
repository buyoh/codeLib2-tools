require 'minitest/autorun'
require 'tmpdir'
require_relative '../lib/formatter'

class FormatterCppTest < Minitest::Test
  def setup
    @dir = Dir.mktmpdir
    # Paths containing shell metacharacters must be passed as-is
    @src_dir = File.join(@dir, 'a b;touch injected')
    Dir.mkdir(@src_dir)
    # Fix the style regardless of .clang-format in ancestor directories
    File.write(File.join(@src_dir, '.clang-format'), "BasedOnStyle: LLVM\n")
  end

  def teardown
    FileUtils.remove_entry_secure(@dir) if @dir
  end

  def require_clang_format
    skip 'clang-format not found' unless system('which', 'clang-format', out: File::NULL)
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

  def test_check_mode
    require_clang_format
    formatted = write_src('ok $(touch injected).cpp', "int main() { return 0; }\n")
    unformatted = write_src('ng.cpp', "int   main(  ){return 0;}\n")
    assert Formatter::Cpp.format(formatted, false)
    out, = capture_io { refute Formatter::Cpp.format(unformatted, false) }
    assert_match(/clang-format assertion/, out)
    assert_equal "int   main(  ){return 0;}\n", File.read(unformatted)
    assert_not_injected
  end

  def test_inplace
    require_clang_format
    path = write_src('ng $(touch injected).cpp', "int   main(  ){return 0;}\n")
    assert Formatter::Cpp.format(path, true)
    assert_equal "int main() { return 0; }\n", File.read(path)
    assert_not_injected
  end

  def test_select_formatter
    assert_equal Formatter::Cpp, Formatter.select_formatter('cpp')
    assert_nil Formatter.select_formatter('go')
  end
end
