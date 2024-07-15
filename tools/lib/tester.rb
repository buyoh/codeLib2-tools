require_relative 'common/runner'

module Tester
  module Cpp
    class << self
      TIMEOUT_BUILD_SEC = 3
      TIMEOUT_RUN_SEC = 10
      def default_options(basedir)
        ['g++', '-I', basedir, '-lm', '-Wall', '-Wno-comment']
      end

      def check_compile(input_src_path, basedir, _tempdir)
        results = {}
        std_list = ['c++11', 'c++14', 'c++17', 'c++20']
        std_list.each do |std|
          key = std
          cmd = default_options(basedir) + ['-fsyntax-only', '-std=' + std, input_src_path]
          # TODO: ?
          cmd2 = ['bash', '-l', '-c', cmd * ' ']
          result = Runner.join_spawn_with_timeout(cmd2, TIMEOUT_BUILD_SEC)

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
        # TODO: ?
        cmd2 = ['bash', '-l', '-c', cmd * ' ']
        result_compile = Runner.join_spawn_with_timeout(cmd2, TIMEOUT_BUILD_SEC)

        results['compile'] = {
          ok: result_compile.exit_code == 0,
          timedout: result_compile.timedout,
          stdout: result_compile.stdout,
          stderr: result_compile.stderr
        }

        return results if result_compile.exit_code != 0

        # TODO: chdir?

        cmd = [tempdir + '/a.out']
        result_run = Runner.join_spawn_with_timeout(cmd, TIMEOUT_RUN_SEC)

        results['run'] = {
          ok: result_run.exit_code == 0,
          timedout: result_run.timedout,
          stdout: result_run.stdout,
          stderr: result_run.stderr
        }

        results
      end

      def refactor_code(path, inplace)
        ok = true
        if inplace
          ok = system("clang-format -i -style=file #{path}")
        else
          diff = `bash -c "diff <( clang-format -style=file #{path} ) #{path}"`
          # TODO: What is this? (Copied from old code)
          unless diff =~ /^\s*$/
            ok = false
            puts "clang-format assertion: #{path}"
            puts 'diff...'
            puts diff
            puts ''
          end
        end
        # Always return true
        ok
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
