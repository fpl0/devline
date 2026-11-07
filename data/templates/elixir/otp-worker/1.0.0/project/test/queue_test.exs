defmodule DevlineWorker.QueueTest do
  use ExUnit.Case, async: false

  test "keeps running after a missing file and writes the next report" do
    directory = Path.join(System.tmp_dir!(), "devline-worker-#{System.unique_integer([:positive])}")
    File.mkdir_p!(directory)
    on_exit(fn -> File.rm_rf(directory) end)
    input = Path.join(directory, "input.txt")
    output = Path.join(directory, "report.json")
    File.write!(input, "one\ntwo\n")

    assert {:ok, failed} = DevlineWorker.Queue.enqueue(input <> ".missing", output)
    assert {:ok, succeeded} = DevlineWorker.Queue.enqueue(input, output)
    assert_receive {:report, ^failed, {:error, :enoent}}, 1_000
    assert_receive {:report, ^succeeded, {:ok, %{"lines" => 2, "bytes" => 8}}}, 1_000
    assert %{"lines" => 2, "bytes" => 8} = :json.decode(File.read!(output))
    assert {:error, :same_path} = DevlineWorker.Queue.enqueue(input, input)
  end
end
