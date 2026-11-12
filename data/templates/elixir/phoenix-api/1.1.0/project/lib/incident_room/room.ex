defmodule IncidentRoom.Room do
  @moduledoc "Serializes incident changes and durably publishes each revision before broadcasting."
  use GenServer

  @transitions %{
    "investigating" => ["identified", "resolved"],
    "identified" => ["monitoring", "resolved"],
    "monitoring" => ["investigating", "resolved"],
    "resolved" => ["investigating"]
  }

  def start_link(options) do
    name = Keyword.get(options, :name, __MODULE__)
    GenServer.start_link(__MODULE__, options, if(name, do: [name: name], else: []))
  end

  def snapshot(server \\ __MODULE__), do: GenServer.call(server, :snapshot)
  def create(params, server \\ __MODULE__), do: GenServer.call(server, {:create, params})
  def update(id, params, server \\ __MODULE__), do: GenServer.call(server, {:update, id, params})

  @impl true
  def init(options) do
    path = Keyword.get(options, :path, System.get_env("INCIDENT_DATABASE", "data/incidents.json"))
    File.mkdir_p!(Path.dirname(path))

    data =
      case File.read(path) do
        {:ok, bytes} -> Jason.decode!(bytes)
        {:error, :enoent} -> %{"version" => 1, "next_id" => 1, "incidents" => []}
        {:error, reason} -> raise File.Error, reason: reason, action: "read", path: path
      end

    unless data["version"] == 1 and is_integer(data["next_id"]) and is_list(data["incidents"]),
      do: raise("Unsupported incident store")

    {:ok, %{path: path, data: data, broadcast: Keyword.get(options, :broadcast, true)}}
  end

  @impl true
  def handle_call(:snapshot, _from, state), do: {:reply, public(state.data), state}

  def handle_call({:create, params}, _from, state) do
    with {:ok, title} <- text(params["title"], "title", 120),
         {:ok, actor} <- text(params["actor"], "actor", 60),
         true <- params["severity"] in ["low", "medium", "high"] do
      now = DateTime.utc_now() |> DateTime.to_iso8601()

      incident = %{
        "id" => state.data["next_id"],
        "title" => title,
        "severity" => params["severity"],
        "status" => "investigating",
        "assignee" => actor,
        "revision" => 1,
        "created_at" => now,
        "timeline" => [
          %{
            "actor" => actor,
            "message" => "Incident opened",
            "at" => now,
            "status" => "investigating"
          }
        ]
      }

      data = %{
        state.data
        | "next_id" => incident["id"] + 1,
          "incidents" => [incident | state.data["incidents"]]
      }

      commit(data, incident, state)
    else
      false -> {:reply, {:error, "Choose low, medium or high severity"}, state}
      error -> {:reply, error, state}
    end
  end

  def handle_call({:update, id, params}, _from, state) do
    incident = Enum.find(state.data["incidents"], &(&1["id"] == id))

    with true <- not is_nil(incident),
         :ok <- revision(incident, params["revision"]),
         {:ok, actor} <- text(params["actor"], "actor", 60),
         {:ok, message} <- text(params["message"], "message", 1000),
         {:ok, next} <- change(incident, params) do
      event = %{
        "actor" => actor,
        "message" => message,
        "at" => DateTime.utc_now() |> DateTime.to_iso8601(),
        "status" => next["status"]
      }

      updated = %{
        next
        | "revision" => next["revision"] + 1,
          "timeline" => next["timeline"] ++ [event]
      }

      data = %{
        state.data
        | "incidents" =>
            Enum.map(state.data["incidents"], fn current ->
              if current["id"] == id, do: updated, else: current
            end)
      }

      commit(data, updated, state)
    else
      false -> {:reply, {:error, "Incident not found"}, state}
      error -> {:reply, error, state}
    end
  end

  defp revision(%{"revision" => revision}, revision), do: :ok
  defp revision(_, _), do: {:error, "Incident changed; review the latest update and try again"}

  defp change(incident, %{"status" => status} = params) do
    allowed = Map.fetch!(@transitions, incident["status"])

    if status == incident["status"] or status in allowed do
      case text(Map.get(params, "assignee", incident["assignee"]), "assignee", 60) do
        {:ok, assignee} -> {:ok, %{incident | "status" => status, "assignee" => assignee}}
        error -> error
      end
    else
      {:error, "That status transition is not allowed"}
    end
  end

  defp change(_, _), do: {:error, "Status is required"}

  defp text(value, field, maximum) when is_binary(value) do
    value = String.trim(value)

    if String.valid?(value) and String.length(value) in 1..maximum,
      do: {:ok, value},
      else: {:error, "#{field} must contain 1–#{maximum} characters"}
  end

  defp text(_, field, _), do: {:error, "#{field} is required"}

  defp public(data), do: %{"incidents" => data["incidents"], "transitions" => @transitions}

  defp commit(data, result, state) do
    temporary = state.path <> ".tmp"

    with :ok <- File.write(temporary, Jason.encode!(data), [:sync]),
         :ok <- File.rename(temporary, state.path) do
      if state.broadcast,
        do:
          Phoenix.PubSub.broadcast(
            DevlinePhoenix.PubSub,
            "incident-room",
            {:snapshot, public(data)}
          )

      {:reply, {:ok, result}, %{state | data: data}}
    else
      {:error, reason} -> {:reply, {:error, "Could not save incident: #{inspect(reason)}"}, state}
    end
  end
end
