require_relative 'common/runner'

module Tester
  module Cpp
    @timeout_build_sec = 10
    @timeout_run_sec = 10
    @timeout_format_sec = 10

    class << self
      attr_accessor :timeout_build_sec, :timeout_run_sec, :timeout_format_sec

      def default_options(basedir)
        ['g++', '-I', basedir, '-lm', '-Wall', '-Wno-comment']
      end

      def check_compile(input_src_path, basedir, _tempdir)
        results = {}
        std_list = ['c++11', 'c++14', 'c++17', 'c++20']
        std_list.each do |std|
          key = std
          cmd = default_options(basedir) + ['-fsyntax-only', '-std=' + std, input_src_path]
          result = Runner.join_spawn_with_timeout(cmd, timeout_build_sec)

          results[key] = {
            ok: result.exit_code == 0,
            timedout: result.timedout,
            stdout: result.stdout,
            stderr: result.stderr
          }
        end
        results
      end

      def run_test(input_src_path, basedir, tempdir)
        results = {}
        std = 'c++20'
        cmd = default_options(basedir) + ['-std=' + std, input_src_path, '-o', tempdir + '/a.out']
        result_compile = Runner.join_spawn_with_timeout(cmd, timeout_build_sec)

        results['compile'] = {
          ok: result_compile.exit_code == 0,
          timedout: result_compile.timedout,
          stdout: result_compile.stdout,
          stderr: result_compile.stderr
        }

        return results if result_compile.exit_code != 0

        # TODO: chdir?

        cmd = [tempdir + '/a.out']
        result_run = Runner.join_spawn_with_timeout(cmd, timeout_run_sec)

        results['run'] = {
          ok: result_run.exit_code == 0,
          timedout: result_run.timedout,
          stdout: result_run.stdout,
          stderr: result_run.stderr
        }

        results
      end

      def refactor_code(path, inplace)
        cmd = ['clang-format', '-style=file']
        cmd += inplace ? ['-i'] : ['--dry-run', '-Werror']
        result = Runner.join_spawn_with_timeout(cmd + [path], timeout_format_sec)
        return true if result.exit_code == 0

        puts "clang-format assertion: #{path}"
        puts result.stderr
        puts ''
        false
      end
    end
  end

  class << self
    def select_tester(lang)
      case lang
      when 'cpp'
        Cpp
      end
    end
  end
end
