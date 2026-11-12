defmodule DevlineWorker.Queue do
  @moduledoc "Durable single-node report queue with bounded supervised tasks and retry backoff."
  use GenServer

  def start_link(options) do
    name = Keyword.get(options, :name, __MODULE__)
    GenServer.start_link(__MODULE__, options, if(name, do: [name: name], else: []))
  end

  def enqueue(key, input, output, server \\ __MODULE__),
    do: GenServer.call(server, {:enqueue, key, Path.expand(input), Path.expand(output)})

  def list(server \\ __MODULE__), do: GenServer.call(server, :list)
  def work(server \\ __MODULE__), do: GenServer.call(server, :work)
  def retry(id, server \\ __MODULE__), do: GenServer.call(server, {:retry, id})

  @impl true
  def init(options) do
    path = Keyword.get(options, :path, System.get_env("REPORT_QUEUE_DATABASE", "data/queue.json"))
    File.mkdir_p!(Path.dirname(path))

    journal =
      case File.read(path) do
        {:ok, bytes} -> JSON.decode!(bytes)
        {:error, :enoent} -> %{"version" => 1, "next_id" => 1, "jobs" => []}
        {:error, reason} -> raise File.Error, reason: reason, action: "read", path: path
      end

    unless journal["version"] == 1 and is_list(journal["jobs"]) and is_integer(journal["next_id"]),
      do: raise("Invalid queue journal")

    jobs =
      Enum.map(journal["jobs"], fn job ->
        if job["status"] == "running",
          do: %{
            job
            | "status" => "queued",
              "available_at" => 0,
              "error" => "Recovered interrupted attempt"
          },
          else: job
      end)

    state = %{
      path: path,
      journal: %{journal | "jobs" => jobs},
      active: %{},
      enabled: false,
      timer: nil,
      tasks: Keyword.get(options, :tasks, DevlineWorker.Tasks),
      concurrency: Keyword.get(options, :concurrency, 2),
      retry_ms: Keyword.get(options, :retry_ms, 500),
      worker: Keyword.get(options, :worker, &DevlineWorker.Report.run/2)
    }

    {:ok, state}
  end

  @impl true
  def handle_call(:list, _from, state), do: {:reply, state.journal["jobs"], state}

  def handle_call(:work, _from, state) do
    {:reply, :ok, schedule(%{state | enabled: true})}
  end

  def handle_call({:enqueue, key, input, output}, _from, state) do
    prior = Enum.find(state.journal["jobs"], &(&1["key"] == key))

    cond do
      not is_binary(key) or String.length(String.trim(key)) not in 1..80 ->
        {:reply, {:error, "Key needs 1–80 characters"}, state}

      prior && prior["input"] == input && prior["output"] == output ->
        {:reply, {:ok, prior}, state}

      prior ->
        {:reply, {:error, "Key already belongs to another job"}, state}

      input == output ->
        {:reply, {:error, "Input and output must differ"}, state}

      File.exists?(output) ->
        {:reply, {:error, "Output already exists; choose a new report path"}, state}

      Enum.any?(state.journal["jobs"], &(&1["output"] == output)) ->
        {:reply, {:error, "Another job owns this output path"}, state}

      true ->
        job = %{
          "id" => state.journal["next_id"],
          "key" => key,
          "input" => input,
          "output" => output,
          "status" => "queued",
          "attempts" => 0,
          "available_at" => 0,
          "error" => nil,
          "result" => nil
        }

        journal = %{
          state.journal
          | "next_id" => job["id"] + 1,
            "jobs" => state.journal["jobs"] ++ [job]
        }

        state = persist!(%{state | journal: journal})
        {:reply, {:ok, job}, schedule(state)}
    end
  end

  def handle_call({:retry, id}, _from, state) do
    case Enum.find(state.journal["jobs"], &(&1["id"] == id)) do
      %{"status" => "failed"} = job ->
        next = %{job | "status" => "queued", "attempts" => 0, "available_at" => 0, "error" => nil}
        state = state |> replace(next) |> persist!()
        {:reply, {:ok, next}, schedule(state)}

      _ ->
        {:reply, {:error, "Only failed jobs can be retried"}, state}
    end
  end

  @impl true
  def handle_info(:tick, state) do
    state = %{state | timer: nil}
    state = if state.enabled, do: dispatch(state), else: state
    {:noreply, schedule(state, 50)}
  end

  def handle_info({reference, result}, state) when is_reference(reference) do
    case Map.pop(state.active, reference) do
      {nil, _} ->
        {:noreply, state}

      {id, active} ->
        Process.demonitor(reference, [:flush])
        {:noreply, complete(id, result, %{state | active: active})}
    end
  end

  def handle_info({:DOWN, reference, :process, _pid, reason}, state) do
    case Map.pop(state.active, reference) do
      {nil, _} ->
        {:noreply, state}

      {id, active} ->
        {:noreply,
         complete(id, {:error, "Worker exited: #{inspect(reason)}"}, %{state | active: active})}
    end
  end

  defp dispatch(state) do
    available = state.concurrency - map_size(state.active)

    ready =
      state.journal["jobs"]
      |> Enum.filter(
        &(&1["status"] == "queued" and &1["available_at"] <= System.system_time(:millisecond))
      )
      |> Enum.take(max(available, 0))

    Enum.reduce(ready, state, fn job, current ->
      running = %{job | "status" => "running", "attempts" => job["attempts"] + 1}
      current = current |> replace(running) |> persist!()
      worker = current.worker

      task =
        Task.Supervisor.async_nolink(current.tasks, fn -> worker.(job["input"], job["output"]) end)

      %{current | active: Map.put(current.active, task.ref, job["id"])}
    end)
  end

  defp complete(id, result, state) do
    job = Enum.find(state.journal["jobs"], &(&1["id"] == id))

    next =
      case result do
        {:ok, summary} ->
          %{job | "status" => "completed", "result" => summary, "error" => nil}

        {:error, reason} ->
          %{
            job
            | "status" => if(job["attempts"] >= 3, do: "failed", else: "queued"),
              "error" => to_string(reason),
              "available_at" =>
                System.system_time(:millisecond) + state.retry_ms * 2 ** (job["attempts"] - 1)
          }

        _ ->
          %{job | "status" => "failed", "error" => "Worker returned an invalid result"}
      end

    state = state |> replace(next) |> persist!()
    schedule(state)
  end

  defp schedule(state, delay \\ 0) do
    if state.enabled and is_nil(state.timer) and
         Enum.any?(state.journal["jobs"], &(&1["status"] == "queued")) do
      %{state | timer: Process.send_after(self(), :tick, delay)}
    else
      state
    end
  end

  defp replace(state, job),
    do: %{
      state
      | journal: %{
          state.journal
          | "jobs" =>
              Enum.map(state.journal["jobs"], fn existing ->
                if existing["id"] == job["id"], do: job, else: existing
              end)
        }
    }

  defp persist!(state) do
    temporary = state.path <> ".tmp"
    File.write!(temporary, JSON.encode!(state.journal), [:sync])
    File.rename!(temporary, state.path)
    state
  end
end
