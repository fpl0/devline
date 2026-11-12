# A real local HTTP endpoint keeps the example runnable without external services.
{:ok, listener} =
  :gen_tcp.listen(0, [:binary, active: false, reuseaddr: true, ip: {127, 0, 0, 1}])

{:ok, {_, port}} = :inet.sockname(listener)

server =
  Task.async(fn ->
    {:ok, client} = :gen_tcp.accept(listener)
    {:ok, _} = :gen_tcp.recv(client, 0, 3000)
    :gen_tcp.send(client, "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
    :gen_tcp.close(client)
  end)

try do
  monitor = %{
    "name" => "Local example",
    "url" => "http://127.0.0.1:#{port}/health",
    "expected_status" => 200
  }

  result = Pulse.Monitor.run([monitor])
  :ok = Pulse.History.append("data/history.json", result)
  IO.puts(JSON.encode!(result))
  Task.await(server)
after
  :gen_tcp.close(listener)
end
