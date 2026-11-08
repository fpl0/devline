defmodule DevlinePhoenix.Health do
  use Phoenix.Controller, formats: [:json]

  def show(conn, _params), do: json(conn, %{status: "ok"})
end

defmodule DevlinePhoenix.Router do
  use Phoenix.Router

  pipeline :release_json do
    plug(Plug.Parsers, parsers: [:json], json_decoder: Jason, length: 128_000)
  end

  get("/healthz", DevlinePhoenix.Health, :show)

  scope "/" do
    pipe_through(:release_json)

    get("/", ReleaseDesk.Controller, :index)
    post("/releases", ReleaseDesk.Controller, :submit)
    get("/releases/:id", ReleaseDesk.Controller, :show)
    get("/releases/:id/audit", ReleaseDesk.Controller, :audit)
    get("/releases/:id/artifact", ReleaseDesk.Controller, :artifact)
    get("/search", ReleaseDesk.Controller, :search)
  end
end

defmodule DevlinePhoenix.Endpoint do
  use Phoenix.Endpoint, otp_app: :devline_phoenix
  plug(DevlinePhoenix.Router)
end

defmodule DevlinePhoenix.Application do
  use Application

  def start(_type, _args) do
    desk = if System.get_env("RELEASE_DESK") == "1", do: ReleaseDesk.Store.children(), else: []

    Supervisor.start_link(
      desk ++ [{Phoenix.PubSub, name: DevlinePhoenix.PubSub}, DevlinePhoenix.Endpoint],
      strategy: :one_for_one
    )
  end
end
