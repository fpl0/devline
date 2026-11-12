defmodule Pulse.CLI do
  def main(arguments) do
    case execute(arguments) do
      {:ok, status} ->
        if status != 0, do: System.halt(status)

      {:error, message} ->
        IO.puts(:stderr, "pulse: #{message}")
        System.halt(1)
    end
  end

  def execute(arguments) do
    {options, positional, invalid} =
      OptionParser.parse(arguments,
        strict: [
          config: :string,
          history: :string,
          timeout: :integer,
          concurrency: :integer,
          help: :boolean
        ]
      )

    cond do
      options[:help] ->
        IO.puts(
          "pulse check --config monitors.json [--history data/history.json] [--timeout 3000] [--concurrency 4]\npulse history [--history data/history.json]"
        )

        {:ok, 0}

      invalid != [] ->
        {:error, "Unknown or invalid options: #{inspect(invalid)}"}

      positional == ["history"] ->
        show_history(options)

      positional == ["check"] ->
        check(options)

      true ->
        {:error, "Use check or history; see --help"}
    end
  end

  defp check(options) do
    timeout = Keyword.get(options, :timeout, 3000)
    concurrency = Keyword.get(options, :concurrency, 4)

    with true <- timeout in 100..60_000 and concurrency in 1..32,
         {:ok, monitors} <- Pulse.Monitor.load(Keyword.get(options, :config, "monitors.json")) do
      run = Pulse.Monitor.run(monitors, timeout: timeout, concurrency: concurrency)

      case Pulse.History.append(Keyword.get(options, :history, "data/history.json"), run) do
        :ok ->
          IO.puts(JSON.encode!(run))
          {:ok, if(run["healthy"] == run["total"], do: 0, else: 2)}

        {:error, reason} ->
          {:error, "Could not save history: #{inspect(reason)}"}
      end
    else
      false -> {:error, "Timeout must be 100–60000 ms and concurrency 1–32"}
      error -> error
    end
  end

  defp show_history(options) do
    with {:ok, runs} <- Pulse.History.read(Keyword.get(options, :history, "data/history.json")) do
      IO.puts(JSON.encode!(%{"runs" => length(runs), "monitors" => Pulse.History.summary(runs)}))
      {:ok, 0}
    end
  end
end
