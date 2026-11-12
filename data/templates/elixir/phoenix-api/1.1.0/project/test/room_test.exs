defmodule IncidentRoom.RoomTest do
  use ExUnit.Case, async: true
  alias IncidentRoom.Room

  setup do
    root = Path.join(System.tmp_dir!(), "incident-room-#{System.unique_integer([:positive])}")
    File.mkdir_p!(root)
    on_exit(fn -> File.rm_rf!(root) end)
    path = Path.join(root, "room.json")
    room = start_supervised!({Room, name: nil, path: path, broadcast: false})
    %{room: room, path: path}
  end

  test "complete response workflow, revisions and restart", %{room: room, path: path} do
    assert {:ok, incident} =
             Room.create(
               %{"title" => "Checkout errors", "severity" => "high", "actor" => "Alex"},
               room
             )

    assert {:ok, next} =
             Room.update(
               incident["id"],
               %{
                 "revision" => 1,
                 "actor" => "Sam",
                 "status" => "identified",
                 "message" => "Database pool exhausted",
                 "assignee" => "Sam"
               },
               room
             )

    assert next["revision"] == 2

    assert {:error, _} =
             Room.update(
               incident["id"],
               %{
                 "revision" => 1,
                 "actor" => "Alex",
                 "status" => "resolved",
                 "message" => "Stale"
               },
               room
             )

    assert {:ok, final} =
             Room.update(
               incident["id"],
               %{
                 "revision" => 2,
                 "actor" => "Sam",
                 "status" => "resolved",
                 "message" => "Pool restored"
               },
               room
             )

    assert length(final["timeline"]) == 3
    stop_supervised!(Room)
    restarted = start_supervised!({Room, name: nil, path: path, broadcast: false})
    assert hd(Room.snapshot(restarted)["incidents"])["status"] == "resolved"
  end

  test "concurrent submissions have distinct durable ids", %{room: room} do
    results =
      1..20
      |> Task.async_stream(
        fn n ->
          Room.create(
            %{"title" => "Incident #{n}", "severity" => "low", "actor" => "Operator"},
            room
          )
        end, max_concurrency: 4)
      |> Enum.to_list()

    assert Enum.all?(results, &match?({:ok, {:ok, _}}, &1))
    incidents = Room.snapshot(room)["incidents"]
    assert length(Enum.uniq_by(incidents, & &1["id"])) == 20
  end

  test "invalid input and transitions do not change state", %{room: room} do
    assert {:error, _} =
             Room.create(%{"title" => " ", "severity" => "high", "actor" => "Sam"}, room)

    assert {:error, _} =
             Room.create(%{"title" => "Issue", "severity" => "critical", "actor" => "Sam"}, room)

    assert Room.snapshot(room)["incidents"] == []

    {:ok, incident} =
      Room.create(%{"title" => "Issue", "severity" => "medium", "actor" => "Sam"}, room)

    assert {:error, _} =
             Room.update(
               incident["id"],
               %{
                 "revision" => 1,
                 "actor" => "Sam",
                 "status" => "monitoring",
                 "message" => "Skip identification"
               },
               room
             )

    assert hd(Room.snapshot(room)["incidents"])["revision"] == 1
  end
end
