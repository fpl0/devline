defmodule DevKitPhoenix.DependencyKitTest do
  use ExUnit.Case, async: true

  test "runs a query against a locally built SQLite NIF" do
    {:ok, database} = Exqlite.Sqlite3.open(":memory:")
    :ok = Exqlite.Sqlite3.execute(database, "CREATE TABLE notes (body TEXT NOT NULL)")
    :ok = Exqlite.Sqlite3.execute(database, "INSERT INTO notes VALUES ('offline')")
    {:ok, statement} = Exqlite.Sqlite3.prepare(database, "SELECT body FROM notes")
    assert {:row, ["offline"]} = Exqlite.Sqlite3.step(database, statement)
    :ok = Exqlite.Sqlite3.release(database, statement)
    :ok = Exqlite.Sqlite3.close(database)
  end

  test "loads database, Redis and AMQP clients without opening connections" do
    assert %{start: {_, :start_link, [{Postgrex.Protocol, _}]}} =
             Postgrex.child_spec(hostname: "localhost")
    assert %{start: {_, :start_link, [{MyXQL.Connection, _}]}} =
             MyXQL.child_spec(hostname: "localhost")
    assert %{start: {Redix, :start_link, _}} = Redix.child_spec(host: "localhost")
    assert {:ok, _} = :amqp_uri.parse(~c"amqp://guest:guest@127.0.0.1/devline")
    assert Code.ensure_loaded?(Ecto.Adapters.SQLite3)
  end

  test "signs S3 requests and verifies JWTs locally" do
    config =
      ExAws.Config.new(:s3,
        access_key_id: "offline-example",
        secret_access_key: "test-only-secret",
        region: "us-east-1",
        scheme: "http://",
        host: "127.0.0.1",
        port: 9000
      )

    assert {:ok, url} = ExAws.S3.presigned_url(config, :get, "devline-test", "readme.txt")
    assert url =~ "X-Amz-Signature="

    signer = Joken.Signer.create("HS256", String.duplicate("a", 32))
    assert {:ok, token, _claims} = Joken.generate_and_sign(%{}, %{"sub" => "offline-user"}, signer)
    assert {:ok, %{"sub" => "offline-user"}} = Joken.verify(token, signer)
  end
end
