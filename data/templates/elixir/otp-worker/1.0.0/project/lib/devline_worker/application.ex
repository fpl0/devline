defmodule DevlineWorker.Application do
  use Application

  @impl true
  def start(_type, _args) do
    Supervisor.start_link(
      [{Task.Supervisor, name: DevlineWorker.Tasks}, DevlineWorker.Queue],
      strategy: :one_for_one,
      name: DevlineWorker.Supervisor
    )
  end
end
