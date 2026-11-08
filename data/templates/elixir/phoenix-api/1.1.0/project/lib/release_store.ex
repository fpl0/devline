defmodule ReleaseDesk.Store do
  @moduledoc "PostgreSQL source of truth, Redis cache, and independent MySQL audit trail."

  @pg ReleaseDesk.Postgres
  @mysql ReleaseDesk.MySQL
  @redis ReleaseDesk.Redis

  def children do
    [
      Supervisor.child_spec({Postgrex, database_options("DEVLINE_POSTGRES_URL", @pg)}, id: @pg),
      Supervisor.child_spec({MyXQL, database_options("DEVLINE_MYSQL_URL", @mysql)}, id: @mysql),
      {Redix, {System.fetch_env!("DEVLINE_REDIS_URL"), [name: @redis]}},
      ReleaseDesk.Worker
    ]
  end

  defp database_options(variable, name) do
    uri = URI.parse(System.fetch_env!(variable))
    [username, password] = String.split(uri.userinfo, ":", parts: 2)

    [
      name: name,
      hostname: uri.host,
      port: uri.port,
      username: URI.decode(username),
      password: URI.decode(password),
      database: String.trim_leading(uri.path, "/"),
      pool_size: 2
    ]
  end

  def migrate! do
    pg!("""
    CREATE TABLE IF NOT EXISTS release_desk (
      id UUID PRIMARY KEY,
      title TEXT NOT NULL,
      notes TEXT NOT NULL,
      status TEXT NOT NULL DEFAULT 'queued',
      published_at TIMESTAMPTZ
    )
    """)

    MyXQL.query!(@mysql, """
    CREATE TABLE IF NOT EXISTS release_desk_audit (
      id CHAR(36) PRIMARY KEY,
      status VARCHAR(20) NOT NULL,
      recorded_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
    """)
  end

  def enabled?, do: Process.whereis(ReleaseDesk.Worker) != nil

  def submit(%{title: title, notes: notes}) do
    id = Ecto.UUID.generate()

    pg!("INSERT INTO release_desk (id, title, notes) VALUES ($1, $2, $3)", [
      uuid(id),
      title,
      notes
    ])

    id
  end

  def unpublished do
    pg!("SELECT id::text FROM release_desk WHERE published_at IS NULL LIMIT 20").rows
    |> List.flatten()
  end

  def published(id),
    do: pg!("UPDATE release_desk SET published_at = NOW() WHERE id = $1", [uuid(id)])

  def complete(id),
    do: pg!("UPDATE release_desk SET status = 'completed' WHERE id = $1", [uuid(id)])

  def release(id) do
    query = "SELECT id::text, title, notes, status FROM release_desk WHERE id = $1"

    case pg!(query, [uuid(id)]).rows do
      [[id, title, notes, status]] -> %{id: id, title: title, notes: notes, status: status}
      [] -> nil
    end
  end

  def status(id) do
    key = "release:#{id}"

    case Redix.command!(@redis, ["GET", key]) do
      nil ->
        case release(id) do
          nil ->
            nil

          record ->
            record = Map.delete(record, :notes)

            if record.status == "completed" do
              Redix.command!(@redis, ["SET", key, Jason.encode!(record), "EX", 60])
            end

            Map.put(record, :source, "postgresql")
        end

      cached ->
        cached |> Jason.decode!() |> Map.put("source", "redis")
    end
  end

  def record_audit(id) do
    MyXQL.query!(
      @mysql,
      """
      INSERT INTO release_desk_audit (id, status)
      VALUES (?, 'completed') ON DUPLICATE KEY UPDATE id = id
      """,
      [id]
    )
  end

  def audit(id) do
    case MyXQL.query!(@mysql, "SELECT id, status FROM release_desk_audit WHERE id = ?", [id]).rows do
      [[id, status]] -> %{id: id, status: status}
      [] -> nil
    end
  end

  defp pg!(sql, arguments \\ []), do: Postgrex.query!(@pg, sql, arguments)
  defp uuid(id), do: Ecto.UUID.dump!(id)
end
