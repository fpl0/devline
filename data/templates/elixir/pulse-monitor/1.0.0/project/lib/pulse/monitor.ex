defmodule Pulse.Monitor do
  @moduledoc "Bounded concurrent HTTP checks; every target produces a result, including failures."

  def load(path) do
    with {:ok, bytes} <- File.read(path),
         {:ok, data} <- JSON.decode(bytes),
         true <- is_list(data) and length(data) in 1..100 do
      Enum.reduce_while(data, {:ok, []}, fn entry, {:ok, result} ->
        case validate(entry) do
          {:ok, entry} -> {:cont, {:ok, result ++ [entry]}}
          error -> {:halt, error}
        end
      end)
      |> case do
        {:ok, entries} ->
          if length(Enum.uniq_by(entries, & &1["name"])) == length(entries),
            do: {:ok, entries},
            else: {:error, "Monitor names must be unique"}

        error ->
          error
      end
    else
      false -> {:error, "Configuration needs 1–100 monitor objects"}
      {:error, reason} -> {:error, "Could not load configuration: #{inspect(reason)}"}
    end
  end

  defp validate(%{"name" => name, "url" => url} = entry)
       when is_binary(name) and is_binary(url) do
    uri = URI.parse(url)
    expected = Map.get(entry, "expected_status", 200)

    if String.length(String.trim(name)) in 1..80 and uri.scheme in ["http", "https"] and
         is_binary(uri.host) and
         uri.host != "" and is_nil(uri.userinfo) and is_nil(uri.fragment) and is_integer(expected) and
         expected in 100..599 do
      {:ok, %{"name" => String.trim(name), "url" => url, "expected_status" => expected}}
    else
      {:error,
       "Each monitor needs a name, an HTTP(S) URL without credentials, and a valid expected_status"}
    end
  end

  defp validate(_), do: {:error, "Each monitor needs name and url strings"}

  def run(monitors, options \\ []) do
    timeout = Keyword.get(options, :timeout, 3000)
    concurrency = Keyword.get(options, :concurrency, 4)
    checker = Keyword.get(options, :checker, &check/2)

    results =
      monitors
      |> Task.async_stream(fn monitor -> checker.(monitor, timeout) end,
        max_concurrency: concurrency,
        timeout: timeout + 500,
        on_timeout: :kill_task,
        ordered: true
      )
      |> Enum.zip(monitors)
      |> Enum.map(fn
        {{:ok, result}, _} ->
          result

        {{:exit, reason}, monitor} ->
          failure(monitor, "Check exited: #{inspect(reason)}", timeout)
      end)

    %{
      "at" => DateTime.utc_now() |> DateTime.to_iso8601(),
      "healthy" => Enum.count(results, & &1["healthy"]),
      "total" => length(results),
      "results" => results
    }
  end

  def check(monitor, timeout) do
    start = System.monotonic_time(:millisecond)

    options = [
      timeout: timeout,
      connect_timeout: timeout,
      autoredirect: false,
      ssl: [
        verify: :verify_peer,
        cacerts: :public_key.cacerts_get(),
        customize_hostname_check: [match_fun: :public_key.pkix_verify_hostname_match_fun(:https)]
      ]
    ]

    result =
      :httpc.request(:get, {String.to_charlist(monitor["url"]), []}, options,
        body_format: :binary
      )

    elapsed = System.monotonic_time(:millisecond) - start

    case result do
      {:ok, {{_, status, _}, _, _}} ->
        Map.merge(monitor, %{
          "healthy" => status == monitor["expected_status"],
          "status" => status,
          "latency_ms" => elapsed,
          "error" => nil
        })

      {:error, reason} ->
        failure(monitor, inspect(reason), elapsed)
    end
  rescue
    error -> failure(monitor, Exception.message(error), 0)
  end

  defp failure(monitor, error, elapsed),
    do:
      Map.merge(monitor, %{
        "healthy" => false,
        "status" => nil,
        "latency_ms" => elapsed,
        "error" => error
      })
end
