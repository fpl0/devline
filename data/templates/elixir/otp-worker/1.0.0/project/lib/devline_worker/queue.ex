defmodule DevlineWorker.Queue do
  @moduledoc "Serializes local file-report jobs under an OTP supervisor."
  use GenServer

  def start_link(_options), do: GenServer.start_link(__MODULE__, [], name: __MODULE__)

  def enqueue(input, output, recipient \\ self()) do
    GenServer.call(__MODULE__, {:enqueue, Path.expand(input), Path.expand(output), recipient})
  end

  @impl true
  def init(_options), do: {:ok, %{pending: :queue.new(), active: nil}}

  @impl true
  def handle_call({:enqueue, path, path, _recipient}, _from, state) do
    {:reply, {:error, :same_path}, state}
  end

  def handle_call({:enqueue, input, output, recipient}, _from, state) do
    id = make_ref()
    pending = :queue.in({id, input, output, recipient}, state.pending)
    {:reply, {:ok, id}, start_next(%{state | pending: pending})}
  end

  @impl true
  def handle_info({reference, result}, %{active: {reference, id, recipient}} = state) do
    Process.demonitor(reference, [:flush])
    send(recipient, {:report, id, result})
    {:noreply, start_next(%{state | active: nil})}
  end

  def handle_info({:DOWN, reference, :process, _pid, reason},
        %{active: {reference, id, recipient}} = state) do
    send(recipient, {:report, id, {:error, reason}})
    {:noreply, start_next(%{state | active: nil})}
  end

  defp start_next(%{active: active} = state) when not is_nil(active), do: state

  defp start_next(state) do
    case :queue.out(state.pending) do
      {{:value, {id, input, output, recipient}}, remaining} ->
        task = Task.Supervisor.async_nolink(DevlineWorker.Tasks, fn -> report(input, output) end)
        %{state | pending: remaining, active: {task.ref, id, recipient}}

      {:empty, _queue} ->
        state
    end
  end

  defp report(input, output) do
    with {:ok, %{type: :regular, size: bytes}} <- File.stat(input),
         :ok <- File.mkdir_p(Path.dirname(output)) do
      lines = input |> File.stream!() |> Enum.count()
      summary = %{"bytes" => bytes, "lines" => lines}
      temporary = output <> ".#{System.unique_integer([:positive])}.tmp"

      try do
        with :ok <- File.write(temporary, :json.encode(summary), [:exclusive]),
             :ok <- File.rename(temporary, output) do
          {:ok, summary}
        end
      after
        File.rm(temporary)
      end
    else
      {:ok, _stat} -> {:error, :not_a_regular_file}
      {:error, _reason} = error -> error
    end
  end
end
