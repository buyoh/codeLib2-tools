require_relative 'common/runner'

module Formatter
  module Cpp
    TIMEOUT_SEC = 10

    class << self
      # inplace: rewrite the file. Otherwise only check it.
      def format(path, inplace)
        cmd = ['clang-format', '-style=file']
        cmd += inplace ? ['-i'] : ['--dry-run', '-Werror']
        result = Runner.join_spawn_with_timeout(cmd + [path], TIMEOUT_SEC)
        return true if result.ok?

        puts "clang-format assertion: #{path}"
        puts result.stderr
        puts ''
        false
      end
    end
  end

  class << self
    def select_formatter(lang)
      case lang
      when 'cpp'
        Cpp
      end
    end
  end
end
