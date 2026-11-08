defmodule ReleaseDesk.Test do
  use ExUnit.Case, async: true
  import Plug.Test

  test "normalizes and validates submitted release notes" do
    assert {:ok, %{title: "Version 1", notes: "The changelog"}} =
             ReleaseDesk.Controller.validate(%{
               "title" => " Version 1 ",
               "notes" => "The changelog"
             })

    assert :invalid =
             ReleaseDesk.Controller.validate(%{"title" => "", "notes" => "The changelog"})

    assert :invalid =
             ReleaseDesk.Controller.validate(%{
               "title" => "Version 1\nBCC: x",
               "notes" => "The changelog"
             })
  end

  test "keeps the local API available without configuring live services" do
    endpoint = DevlinePhoenix.Endpoint
    response = endpoint.call(conn(:get, "/"), [])
    assert response.status == 200
    assert %{"health" => "/healthz"} = Jason.decode!(response.resp_body)

    request =
      conn(:post, "/releases", ~s({"title":"Offline","notes":"No services"}))
      |> Plug.Conn.put_req_header("content-type", "application/json")

    assert endpoint.call(request, []).status == 503
  end
end
