defmodule DevlinePhoenix.Router do
  use Plug.Router
  plug(:match)
  plug(:dispatch)

  get "/healthz" do
    conn |> put_resp_content_type("application/json") |> send_resp(200, "{\"status\":\"ok\"}")
  end

  get "/" do
    conn
    |> put_resp_content_type("text/html")
    |> send_file(200, Application.app_dir(:devline_phoenix, "priv/static/index.html"))
  end

  get "/incidents" do
    conn
    |> put_resp_content_type("application/json")
    |> send_resp(200, Jason.encode!(IncidentRoom.Room.snapshot()))
  end

  match _ do
    send_resp(conn, 404, "Not found")
  end
end

defmodule DevlinePhoenix.Endpoint do
  use Phoenix.Endpoint, otp_app: :devline_phoenix
  socket("/socket", IncidentRoom.Socket, websocket: [max_frame_size: 32_768], longpoll: false)

  plug(Plug.Static,
    at: "/assets",
    from: {:devline_phoenix, "priv/static"},
    only: ~w(app.js style.css phoenix.mjs)
  )

  plug(DevlinePhoenix.Router)
end

defmodule DevlinePhoenix.Application do
  use Application
  @impl true
  def start(_type, _args) do
    Supervisor.start_link(
      [
        {Phoenix.PubSub, name: DevlinePhoenix.PubSub},
        IncidentRoom.Room,
        DevlinePhoenix.Endpoint
      ], strategy: :one_for_one, name: DevlinePhoenix.Supervisor)
  end
end
