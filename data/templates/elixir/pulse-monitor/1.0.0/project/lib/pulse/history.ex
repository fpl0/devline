defmodule Pulse.History do
  @moduledoc "Keeps the latest 500 runs with atomic file replacement."
  def read(path) do
    case File.read(path) do
      {:ok, bytes} ->
        case JSON.decode(bytes) do
          {:ok, %{"version" => 1, "runs" => runs}} when is_list(runs) -> {:ok, runs}
          _ -> {:error, "History file is invalid; preserve it before choosing a new history path"}
        end

      {:error, :enoent} ->
        {:ok, []}

      {:error, reason} ->
        {:error, inspect(reason)}
    end
  end

  def append(path, run) do
    :global.trans({{__MODULE__, Path.expand(path)}, self()}, fn ->
      with {:ok, runs} <- read(path), :ok <- File.mkdir_p(Path.dirname(path)) do
        temporary = path <> ".#{System.unique_integer([:positive])}.tmp"

        try do
          with :ok <-
                 File.write(
                   temporary,
                   JSON.encode!(%{"version" => 1, "runs" => Enum.take([run | runs], 500)}),
                   [:sync]
                 ),
               :ok <- File.rename(temporary, path),
               do: :ok
        after
          File.rm(temporary)
        end
      end
    end)
  end

  def summary(runs) do
    runs
    |> Enum.flat_map(& &1["results"])
    |> Enum.group_by(& &1["name"])
    |> Enum.sort_by(&elem(&1, 0))
    |> Enum.map(fn {name, results} ->
      latencies = Enum.map(results, & &1["latency_ms"]) |> Enum.sort()

      %{
        "name" => name,
        "checks" => length(results),
        "healthy" => Enum.count(results, & &1["healthy"]),
        "p95_ms" => Enum.at(latencies, max(0, ceil(length(latencies) * 0.95) - 1))
      }
    end)
  end
end
