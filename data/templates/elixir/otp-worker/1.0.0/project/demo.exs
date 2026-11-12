key = "example-#{System.system_time(:microsecond)}"
{:ok, _} = DevlineWorker.Queue.enqueue(key, "fixtures/sales.json", "data/reports/#{key}.json")
:ok = DevlineWorker.Queue.work()

case DevlineWorker.CLI.await() do
  {:ok, jobs} ->
    IO.puts(JSON.encode!(jobs))

  {:error, error} ->
    IO.puts(:stderr, error)
    System.halt(1)
end
