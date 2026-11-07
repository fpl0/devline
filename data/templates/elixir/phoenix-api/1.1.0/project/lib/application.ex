defmodule DevKitPhoenix.Health do
  use Phoenix.Controller, formats: [:json]

  def show(conn, _params), do: json(conn, %{status: "ok"})
end

defmodule DevKitPhoenix.Router do
  use Phoenix.Router

  get "/healthz", DevKitPhoenix.Health, :show
end

defmodule DevKitPhoenix.Endpoint do
  use Phoenix.Endpoint, otp_app: :devkit_phoenix
  plug DevKitPhoenix.Router
end

defmodule DevKitPhoenix.Application do
  use Application

  def start(_type, _args) do
    Supervisor.start_link([{Phoenix.PubSub, name: DevKitPhoenix.PubSub}, DevKitPhoenix.Endpoint], strategy: :one_for_one)
  end
end
