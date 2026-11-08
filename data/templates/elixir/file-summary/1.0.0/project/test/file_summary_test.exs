defmodule FileSummaryTest do
  use ExUnit.Case, async: true

  test "summarizes a UTF-8 file" do
    assert FileSummary.read("fixtures/notes.txt") == {:ok, %{lines: 2, words: 4, bytes: 23}}
  end

  test "returns an error for an unreadable input" do
    assert FileSummary.read("missing.txt") == {:error, :enoent}
  end
end
