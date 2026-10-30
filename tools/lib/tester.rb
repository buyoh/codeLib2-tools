require_relative 'common/runner'

module Tester
  module Cpp
    @timeout_build_sec = 10
    @timeout_run_sec = 10

    class << self
      attr_accessor :timeout_build_sec, :timeout_run_sec

      def default_options(basedir)
        ['g++', '-I', basedir, '-lm', '-Wall', '-Wno-comment']
      end

      def check_compile(input_src_path, basedir, _tempdir)
        results = {}
        std_list = ['c++11', 'c++14', 'c++17', 'c++20']
        std_list.each do |std|
          key = std
          cmd = default_options(basedir) + ['-fsyntax-only', '-std=' + std, input_src_path]
          results[key] = Runner.join_spawn_with_timeout(cmd, timeout_build_sec).to_h
        end
        results
      end

      def run_test(input_src_path, basedir, tempdir)
        results = {}
        std = 'c++20'
        cmd = default_options(basedir) + ['-std=' + std, input_src_path, '-o', tempdir + '/a.out']
        result_compile = Runner.join_spawn_with_timeout(cmd, timeout_build_sec)

        results['compile'] = result_compile.to_h

        return results unless result_compile.ok?

        # TODO: chdir?

        cmd = [tempdir + '/a.out']
        results['run'] = Runner.join_spawn_with_timeout(cmd, timeout_run_sec).to_h

        results
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
