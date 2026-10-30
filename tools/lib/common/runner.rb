require 'open3'

module Runner
  class Result
    attr_reader :stdout, :stderr, :exit_code, :timedout

    def initialize(stdout, stderr, exit_code, timedout)
      @stdout = stdout
      @stderr = stderr
      @exit_code = exit_code
      @timedout = timedout
    end

    def ok?
      @exit_code == 0
    end

    def to_h
      { ok: ok?, timedout: @timedout, stdout: @stdout, stderr: @stderr }
    end
  end

  PIPE_CLOSE_WAIT_SEC = 1

  class << self
    # Run command (argv array, without shell) and wait for it at most `timeout` seconds.
    # On timeout, the whole process group is killed and exit_code is nil.
    # stdout and stderr are always String.
    def join_spawn_with_timeout(command, timeout)
      # [cmd, argv0] form prevents a single-element command from being interpreted by shell
      Open3.popen3([command[0], command[0]], *command[1..], pgroup: true) do |stdin, stdout, stderr, wait_thr|
        stdin.close
        # Read both pipes concurrently to avoid deadlock when one of them is full
        stdout_reader = Thread.new { read_all(stdout) }
        stderr_reader = Thread.new { read_all(stderr) }

        timedout = wait_thr.join(timeout).nil?
        # Kill descendants remaining in the process group (and the command itself on timeout)
        kill_process_group(wait_thr.pid)
        status = wait_thr.value

        # Descendants which left the process group (e.g. setsid) may keep the pipes open
        unless [stdout_reader, stderr_reader].all? { |t| t.join(PIPE_CLOSE_WAIT_SEC) }
          stdout.close
          stderr.close
        end

        result_stderr = stderr_reader.value
        result_stderr += "\n(terminated by signal #{Signal.signame(status.termsig)})" if status.signaled? && !timedout

        Result.new(stdout_reader.value, result_stderr, timedout ? nil : status.exitstatus, timedout)
      end
    rescue SystemCallError => e
      # e.g. command not found
      Result.new('', "#{e.class}: #{e.message}", nil, false)
    end

    private

    # Read until EOF. If the pipe is closed by another thread, return the data read so far.
    def read_all(io)
      buf = +''
      loop { buf << io.readpartial(65_536) }
    rescue EOFError, IOError
      # readpartial returns BINARY; output of compilers is assumed to be UTF-8
      buf.force_encoding(Encoding::UTF_8)
    end

    def kill_process_group(pgid)
      Process.kill('KILL', -pgid)
    rescue Errno::ESRCH
      # already exited
    end
  end
end
