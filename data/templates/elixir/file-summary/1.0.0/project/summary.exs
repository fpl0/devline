case System.argv() do
  [path] ->
    case FileSummary.read(path) do
      {:ok, summary} ->
        IO.puts(:json.encode(summary))

      {:error, reason} ->
        IO.puts(:stderr, "Could not read #{path}: #{inspect(reason)}")
        System.halt(1)
    end

  _ ->
    IO.puts(:stderr, "Usage: mix run summary.exs -- FILE")
    System.halt(2)
end
