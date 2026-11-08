defmodule DevlinePhoenix.Health do
  use Phoenix.Controller, formats: [:json]

  def show(conn, _params), do: json(conn, %{status: "ok"})
end

defmodule DevlinePhoenix.Router do
  use Phoenix.Router

  get "/healthz", DevlinePhoenix.Health, :show
end

defmodule DevlinePhoenix.Endpoint do
  use Phoenix.Endpoint, otp_app: :devline_phoenix
  plug DevlinePhoenix.Router
end

defmodule DevlinePhoenix.Application do
  use Application

  def start(_type, _args) do
    Supervisor.start_link([{Phoenix.PubSub, name: DevlinePhoenix.PubSub}, DevlinePhoenix.Endpoint], strategy: :one_for_one)
  end
end
