module Runner
  class Result
    attr_reader :stdout, :stderr, :exit_code, :timedout

    def initialize(stdout, stderr, exit_code, timedout)
      @stdout = stdout
      @stderr = stderr
      @exit_code = exit_code
      @timedout = timedout
    end
  end

  class << self
    def join_spawn_with_timeout(command, timeout)
      # p command
      result_stdout = nil
      result_stderr = nil
      result_code = nil
      result_timedout = false

      # stdout_rx, stdout_tx = IO.pipe
      stderr_rx, stderr_tx = IO.pipe

      # spawn(command, :out => stdout_tx, :err => stderr_tx)
      stdout_rx = IO.popen(command, err: stderr_tx)
      pid = stdout_rx.pid
      # stdout_tx.close
      stderr_tx.close

      thread1 = nil
      thread2 = nil

      thread1 = Thread.new do
        # read stdout and stderr
        result_stdout = stdout_rx.read
        result_stderr = stderr_rx.read
        Process.waitpid(pid)
        result_code = $?.exitstatus
        stdout_rx.close
        stderr_rx.close
        thread1 = nil
        thread2.kill if thread2
      end

      thread2 = Thread.new do
        sleep timeout
        Process.kill('KILL', pid)
        result_timedout = true
        thread2 = nil
        thread1.kill if thread1
      end

      [thread1, thread2].each do |t|
        t.join if t
      end

      Result.new(result_stdout, result_stderr, result_code, result_timedout)
    end
  end
end

# test

# res = Runner::join_spawn_with_timeout(['sleep', "5"], 2)
# p res.stdout, res.stderr, res.exit_code, res
