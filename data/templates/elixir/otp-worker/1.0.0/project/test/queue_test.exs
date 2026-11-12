defmodule DevlineWorker.QueueTest do
  use ExUnit.Case, async: true
  alias DevlineWorker.Queue

  setup do
    root = Path.join(System.tmp_dir!(), "report-queue-#{System.unique_integer([:positive])}")
    File.mkdir_p!(root)
    on_exit(fn -> File.rm_rf!(root) end)
    input = Path.join(root, "input.json")

    File.write!(
      input,
      ~s([{"category":"Books","amount_cents":1200},{"category":"Books","amount_cents":-200}])
    )

    tasks = start_supervised!({Task.Supervisor, []})
    %{root: root, input: input, tasks: tasks}
  end

  defp wait(queue, predicate, remaining \\ 200)
  defp wait(_, _, 0), do: flunk("Queue did not reach expected state")

  defp wait(queue, predicate, remaining) do
    jobs = Queue.list(queue)

    if predicate.(jobs),
      do: jobs,
      else:
        (
          Process.sleep(10)
          wait(queue, predicate, remaining - 1)
        )
  end

  test "persistent enqueue idempotency complete report and restart", context do
    path = Path.join(context.root, "queue.json")
    options = [name: nil, path: path, tasks: context.tasks, retry_ms: 5]
    queue = start_supervised!({Queue, options})
    output = Path.join(context.root, "report.json")
    assert {:ok, job} = Queue.enqueue("report", context.input, output, queue)
    assert {:ok, ^job} = Queue.enqueue("report", context.input, output, queue)
    assert {:error, _} = Queue.enqueue("report", context.input, output <> ".other", queue)
    stop_supervised!(Queue)
    restarted = start_supervised!({Queue, options})
    assert [%{"status" => "queued"}] = Queue.list(restarted)
    Queue.work(restarted)
    wait(restarted, &Enum.all?(&1, fn item -> item["status"] == "completed" end))
    assert JSON.decode!(File.read!(output))["total_cents"] == 1000
    assert hd(JSON.decode!(File.read!(path))["jobs"])["error"] == nil
  end

  @tag :capture_log
  test "worker crashes are supervised and exhausted jobs can be retried", context do
    {:ok, attempts} = Agent.start_link(fn -> 0 end)

    worker = fn input, output ->
      attempt = Agent.get_and_update(attempts, &{&1, &1 + 1})
      if attempt < 3, do: exit(:simulated_failure), else: DevlineWorker.Report.run(input, output)
    end

    queue =
      start_supervised!(
        {Queue,
         name: nil,
         path: Path.join(context.root, "queue.json"),
         tasks: context.tasks,
         worker: worker,
         retry_ms: 5}
      )

    {:ok, job} =
      Queue.enqueue("retry", context.input, Path.join(context.root, "report.json"), queue)

    Queue.work(queue)

    assert [%{"attempts" => 3}] =
             wait(queue, &Enum.all?(&1, fn item -> item["status"] == "failed" end))

    assert Process.alive?(queue)
    assert {:ok, _} = Queue.retry(job["id"], queue)
    wait(queue, &Enum.all?(&1, fn item -> item["status"] == "completed" end))
  end

  test "running tasks respect the concurrency limit", context do
    {:ok, counts} = Agent.start_link(fn -> {0, 0} end)

    worker = fn input, output ->
      Agent.update(counts, fn {active, peak} -> {active + 1, max(peak, active + 1)} end)
      Process.sleep(20)
      result = DevlineWorker.Report.run(input, output)
      Agent.update(counts, fn {active, peak} -> {active - 1, peak} end)
      result
    end

    queue =
      start_supervised!(
        {Queue,
         name: nil,
         path: Path.join(context.root, "queue.json"),
         tasks: context.tasks,
         worker: worker,
         concurrency: 2}
      )

    for number <- 1..8 do
      {:ok, _} =
        Queue.enqueue(
          "job-#{number}",
          context.input,
          Path.join(context.root, "report-#{number}.json"),
          queue
        )
    end

    Queue.work(queue)
    wait(queue, &Enum.all?(&1, fn item -> item["status"] == "completed" end))
    assert Agent.get(counts, &elem(&1, 1)) == 2
  end

  test "invalid data fails without publishing a partial report", context do
    output = Path.join(context.root, "report.json")
    File.write!(context.input, ~s([{"category":"Books","amount_cents":1.5}]))
    assert {:error, _} = DevlineWorker.Report.run(context.input, output)
    refute File.exists?(output)
  end

  test "an interrupted journal recovers running work", context do
    path = Path.join(context.root, "queue.json")
    output = Path.join(context.root, "report.json")

    job = %{
      "id" => 1,
      "key" => "interrupted",
      "input" => context.input,
      "output" => output,
      "status" => "running",
      "attempts" => 1,
      "available_at" => 0,
      "error" => nil,
      "result" => nil
    }

    File.write!(path, JSON.encode!(%{"version" => 1, "next_id" => 2, "jobs" => [job]}))
    queue = start_supervised!({Queue, name: nil, path: path, tasks: context.tasks})
    assert [%{"status" => "queued"}] = Queue.list(queue)
    Queue.work(queue)

    assert [%{"attempts" => 2}] =
             wait(queue, &Enum.all?(&1, fn item -> item["status"] == "completed" end))
  end
end
