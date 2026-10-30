require 'minitest/autorun'
require 'tmpdir'
require_relative '../lib/common/runner'

class RunnerTest < Minitest::Test
  def test_captures_stdout_stderr_and_exit_code
    result = Runner.join_spawn_with_timeout(['sh', '-c', 'echo out; echo err >&2; exit 3'], 5)
    assert_equal "out\n", result.stdout
    assert_equal "err\n", result.stderr
    assert_equal 3, result.exit_code
    refute result.timedout
  end

  def test_to_h
    assert_equal({ ok: true, timedout: false, stdout: "o\n", stderr: '' },
                 Runner.join_spawn_with_timeout(['echo', 'o'], 5).to_h)
    assert_equal({ ok: false, timedout: false, stdout: '', stderr: '' },
                 Runner.join_spawn_with_timeout(['false'], 5).to_h)
  end

  def test_output_encoding_is_utf8
    result = Runner.join_spawn_with_timeout(['printf', "\u2018x\u2019"], 5)
    assert_equal Encoding::UTF_8, result.stdout.encoding
    assert_equal "\u2018x\u2019", result.stdout
  end

  def test_large_stderr_does_not_deadlock
    cmd = ['ruby', '-e', '$stderr.write("e" * 200_000); $stdout.write("o" * 200_000)']
    result = Runner.join_spawn_with_timeout(cmd, 5)
    refute result.timedout
    assert_equal 0, result.exit_code
    assert_equal 200_000, result.stderr.size
    assert_equal 200_000, result.stdout.size
  end

  def test_timeout
    started = Time.now
    result = Runner.join_spawn_with_timeout(['sh', '-c', 'echo begin; sleep 30'], 1)
    assert_operator Time.now - started, :<, 10
    assert result.timedout
    assert_nil result.exit_code
    assert_equal "begin\n", result.stdout
    assert_equal '', result.stderr
  end

  def test_timeout_kills_grandchildren
    Dir.mktmpdir do |dir|
      pid_file = File.join(dir, 'pid')
      cmd = ['sh', '-c', "sleep 30 & echo $! > '#{pid_file}'; wait"]
      result = Runner.join_spawn_with_timeout(cmd, 1)
      assert result.timedout
      grandchild = File.read(pid_file).to_i
      assert_operator grandchild, :>, 0
      killed = 50.times.any? do
        break true unless process_alive?(grandchild)

        sleep 0.1
        false
      end
      assert killed, 'grandchild process should be killed'
    end
  end

  def test_signal_is_reported
    result = Runner.join_spawn_with_timeout(['sh', '-c', 'kill -SEGV $$'], 5)
    refute result.timedout
    assert_nil result.exit_code
    assert_match(/terminated by signal SEGV/, result.stderr)
  end

  def test_single_element_command_is_not_interpreted_by_shell
    Dir.mktmpdir do |dir|
      injected = File.join(dir, 'injected')
      result = Runner.join_spawn_with_timeout(["echo $(touch '#{injected}')"], 5)
      assert_nil result.exit_code
      refute File.exist?(injected)
    end
  end

  def test_descendant_leaving_process_group
    started = Time.now
    result = Runner.join_spawn_with_timeout(['sh', '-c', 'echo hi; setsid sleep 3 &'], 10)
    assert_operator Time.now - started, :<, 5
    assert_equal 0, result.exit_code
    assert_equal "hi\n", result.stdout
  end

  def test_background_grandchild_holding_pipe
    started = Time.now
    result = Runner.join_spawn_with_timeout(['sh', '-c', 'echo done; sleep 30 &'], 10)
    assert_operator Time.now - started, :<, 5
    refute result.timedout
    assert_equal 0, result.exit_code
    assert_equal "done\n", result.stdout
  end

  def test_command_not_found
    result = Runner.join_spawn_with_timeout(['/nonexistent/command'], 5)
    assert_nil result.exit_code
    refute result.timedout
    assert_equal '', result.stdout
    refute_empty result.stderr
  end

  private

  def process_alive?(pid)
    Process.kill(0, pid)
    # A zombie (not yet reaped by init) is regarded as dead
    !File.read("/proc/#{pid}/stat").split(') ').last.start_with?('Z')
  rescue Errno::ESRCH, Errno::ENOENT
    false
  end
end
