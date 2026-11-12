defmodule DevlineWorker.CLI do
  alias DevlineWorker.Queue

  def main(arguments) do
    result =
      case arguments do
        ["enqueue", key, input, output] ->
          Queue.enqueue(key, input, output)

        ["list"] ->
          {:ok, Queue.list()}

        ["retry", id] ->
          case Integer.parse(id) do
            {id, ""} -> Queue.retry(id)
            _ -> {:error, "Job id must be an integer"}
          end

        ["work"] ->
          :ok = Queue.work()
          await()

        ["--help"] ->
          {:ok,
           %{
             "commands" => ["enqueue KEY INPUT.json OUTPUT.json", "list", "work", "retry ID"],
             "database" => "Set REPORT_QUEUE_DATABASE to choose a journal"
           }}

        _ ->
          {:error, "Use enqueue, list, work or retry. See --help."}
      end

    case result do
      {:ok, data} ->
        IO.puts(JSON.encode!(data))

      {:error, message} ->
        IO.puts(:stderr, "report-queue: #{message}")
        System.halt(1)
    end
  end

  def await do
    jobs = Queue.list()

    if Enum.any?(jobs, &(&1["status"] in ["queued", "running"])) do
      Process.sleep(50)
      await()
    else
      if Enum.any?(jobs, &(&1["status"] == "failed")),
        do:
          {:error, "Some jobs failed; use list to inspect and retry after correcting their input"},
        else: {:ok, jobs}
    end
  end
end
