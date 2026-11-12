defmodule IncidentRoom.Socket do
  use Phoenix.Socket
  channel("room:lobby", IncidentRoom.Channel)
  @impl true
  def connect(_params, socket, _info), do: {:ok, socket}
  @impl true
  def id(_socket), do: nil
end

defmodule IncidentRoom.Channel do
  use Phoenix.Channel
  alias IncidentRoom.Room

  @impl true
  def join("room:lobby", _params, socket) do
    Phoenix.PubSub.subscribe(DevlinePhoenix.PubSub, "incident-room")
    {:ok, Room.snapshot(), socket}
  end

  @impl true
  def handle_in("create", params, socket) when is_map(params),
    do: respond(Room.create(params), socket)

  def handle_in("update", %{"id" => id} = params, socket) when is_integer(id),
    do: respond(Room.update(id, params), socket)

  def handle_in(_, _, socket), do: {:reply, {:error, %{message: "Unknown command"}}, socket}

  @impl true
  def handle_info({:snapshot, snapshot}, socket) do
    push(socket, "snapshot", snapshot)
    {:noreply, socket}
  end

  defp respond({:ok, incident}, socket), do: {:reply, {:ok, incident}, socket}
  defp respond({:error, message}, socket), do: {:reply, {:error, %{message: message}}, socket}
end
