defmodule IncidentRoom.ChannelTest do
  use ExUnit.Case, async: false
  import Phoenix.ChannelTest
  @endpoint DevlinePhoenix.Endpoint

  test "two subscribers see persisted updates and invalid commands return errors" do
    {:ok, _, first} =
      socket(IncidentRoom.Socket, "first", %{})
      |> subscribe_and_join(IncidentRoom.Channel, "room:lobby")

    {:ok, _, _second} =
      socket(IncidentRoom.Socket, "second", %{})
      |> subscribe_and_join(IncidentRoom.Channel, "room:lobby")

    ref =
      push(first, "create", %{
        "title" => "Channel incident",
        "actor" => "Alex",
        "severity" => "high"
      })

    assert_reply(ref, :ok, %{"id" => identity})
    assert_push("snapshot", %{"incidents" => incidents})
    assert Enum.any?(incidents, &(&1["id"] == identity))
    assert_push("snapshot", %{"incidents" => _})
    invalid = push(first, "update", %{"id" => identity, "revision" => 0})
    assert_reply(invalid, :error, %{message: _})
  end
end
