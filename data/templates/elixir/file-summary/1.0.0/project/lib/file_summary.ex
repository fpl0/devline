defmodule FileSummary do
  def read(path) do
    with {:ok, text} <- File.read(path) do
      if String.valid?(text) do
        lines = Regex.split(~r/\r\n|\r|\n/, text)
        count = length(lines) - if List.last(lines) == "", do: 1, else: 0
        {:ok, %{lines: count, words: length(String.split(text)), bytes: byte_size(text)}}
      else
        {:error, :invalid_utf8}
      end
    end
  end
end
