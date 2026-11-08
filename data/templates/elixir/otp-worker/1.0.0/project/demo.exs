output = "reports/example.json"
{:ok, id} = DevlineWorker.Queue.enqueue("fixtures/example.txt", output)

receive do
  {:report, ^id, {:ok, report}} ->
    IO.puts(:json.encode(report))
    IO.puts("Saved #{output}")

  {:report, ^id, {:error, reason}} ->
    IO.puts(:stderr, "Report failed: #{inspect(reason)}")
    System.halt(1)
after
  5_000 ->
    IO.puts(:stderr, "Report did not complete within five seconds")
    System.halt(1)
end
