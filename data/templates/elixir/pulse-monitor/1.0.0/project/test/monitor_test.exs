defmodule Pulse.MonitorTest do
  use ExUnit.Case

  setup do
    root = Path.join(System.tmp_dir!(), "pulse-#{System.unique_integer([:positive])}")
    File.mkdir_p!(root)
    on_exit(fn -> File.rm_rf!(root) end)
    %{root: root}
  end

  test "real HTTP status and refused connections produce useful results" do
    {:ok, listener} = :gen_tcp.listen(0, [:binary, active: false, ip: {127, 0, 0, 1}])
    {:ok, {_, port}} = :inet.sockname(listener)

    server =
      Task.async(fn ->
        {:ok, client} = :gen_tcp.accept(listener)
        {:ok, _} = :gen_tcp.recv(client, 0, 2000)

        :gen_tcp.send(
          client,
          "HTTP/1.1 503 Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )

        :gen_tcp.close(client)
      end)

    monitor = %{
      "name" => "service",
      "url" => "http://127.0.0.1:#{port}",
      "expected_status" => 200
    }

    result = Pulse.Monitor.run([monitor], timeout: 1000)
    assert result["healthy"] == 0
    assert hd(result["results"])["status"] == 503
    Task.await(server)
    :gen_tcp.close(listener)
    result = Pulse.Monitor.run([monitor], timeout: 300)
    assert hd(result["results"])["error"] != nil
  end

  test "bounded concurrency, timeout and stable ordering" do
    {:ok, counter} = Agent.start_link(fn -> {0, 0} end)
    monitors = for n <- 1..8, do: %{"name" => "#{n}"}

    checker = fn monitor, _ ->
      Agent.update(counter, fn {active, peak} -> {active + 1, max(peak, active + 1)} end)
      Process.sleep(10)
      Agent.update(counter, fn {active, peak} -> {active - 1, peak} end)
      Map.merge(monitor, %{"healthy" => true, "latency_ms" => 10})
    end

    run = Pulse.Monitor.run(monitors, checker: checker, concurrency: 2)
    assert run["healthy"] == 8
    assert Agent.get(counter, &elem(&1, 1)) == 2
    assert Enum.map(run["results"], & &1["name"]) == Enum.map(monitors, & &1["name"])

    timed =
      Pulse.Monitor.run([hd(monitors)], timeout: 1, checker: fn _, _ -> Process.sleep(1000) end)

    refute hd(timed["results"])["healthy"]
  end

  test "configuration validation and persistent history", %{root: root} do
    config = Path.join(root, "monitors.json")
    File.write!(config, ~s([{"name":"app","url":"file:///etc/passwd"}]))
    assert {:error, _} = Pulse.Monitor.load(config)
    File.write!(config, ~s([{"name":"app","url":"http://localhost:4000"}]))
    assert {:ok, [_]} = Pulse.Monitor.load(config)
    history = Path.join(root, "history.json")
    run = %{"results" => [%{"name" => "app", "healthy" => true, "latency_ms" => 12}]}
    assert :ok = Pulse.History.append(history, run)
    assert {:ok, [^run]} = Pulse.History.read(history)
    assert [%{"p95_ms" => 12, "checks" => 1}] = Pulse.History.summary([run])
    File.write!(history, "corrupt")
    assert {:error, _} = Pulse.History.append(history, run)
    assert File.read!(history) == "corrupt"
  end
end
