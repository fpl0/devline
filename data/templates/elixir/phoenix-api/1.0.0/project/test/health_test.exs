defmodule DevKitPhoenix.HealthTest do
  use ExUnit.Case, async: true
  import Plug.Test

  test "GET /healthz" do
    conn = DevKitPhoenix.Endpoint.call(conn(:get, "/healthz"), [])
    assert conn.status == 200
    assert JSON.decode!(conn.resp_body) == %{"status" => "ok"}
  end
end
