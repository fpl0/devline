defmodule DevlineWorker.Report do
  @moduledoc "Builds an atomic category report from an explicit JSON dataset."
  def run(input, output) do
    with {:ok, %{type: :regular, size: bytes}} when bytes <= 10_000_000 <- File.stat(input),
         {:ok, source} <- File.read(input),
         {:ok, rows} <- JSON.decode(source),
         {:ok, summary} <- summarize(rows),
         :ok <- File.mkdir_p(Path.dirname(output)) do
      temporary = output <> ".#{System.unique_integer([:positive])}.tmp"

      try do
        with :ok <- File.write(temporary, JSON.encode!(summary), [:exclusive, :sync]),
             :ok <- File.rename(temporary, output),
             do: {:ok, summary}
      after
        File.rm(temporary)
      end
    else
      {:ok, _} -> {:error, "Input must be a regular JSON file no larger than 10 MB"}
      {:error, reason} -> {:error, inspect(reason)}
    end
  rescue
    error -> {:error, Exception.message(error)}
  end

  def summarize(rows) when is_list(rows) and length(rows) in 1..100_000 do
    valid =
      Enum.all?(rows, fn
        %{"category" => category, "amount_cents" => amount}
        when is_binary(category) and is_integer(amount) ->
          String.length(String.trim(category)) in 1..80 and amount in -100_000_000..100_000_000

        _ ->
          false
      end)

    if valid do
      categories =
        rows
        |> Enum.group_by(&String.trim(&1["category"]))
        |> Enum.sort_by(&elem(&1, 0))
        |> Enum.map(fn {category, items} ->
          amounts = Enum.map(items, & &1["amount_cents"])

          %{
            "category" => category,
            "rows" => length(items),
            "total_cents" => Enum.sum(amounts),
            "minimum_cents" => Enum.min(amounts),
            "maximum_cents" => Enum.max(amounts)
          }
        end)

      {:ok,
       %{
         "rows" => length(rows),
         "total_cents" => Enum.sum(Enum.map(rows, & &1["amount_cents"])),
         "categories" => categories
       }}
    else
      {:error, "Every row needs a nonblank category and integer amount_cents"}
    end
  end

  def summarize(_), do: {:error, "Input must contain 1–100000 rows"}
end
