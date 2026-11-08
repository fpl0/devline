defmodule ReleaseDesk.Delivery do
  @moduledoc "Publishes notes to S3, updates the search index, and sends a local notification."

  @bucket "devline"
  @index "elixir-releases"

  def ship(%{id: id, title: title, notes: notes} = release) do
    s3(ExAws.S3.put_object(@bucket, key(id), notes, content_type: "text/plain; charset=utf-8"))

    %{"taskUid" => task} =
      meili!(
        "/indexes/#{@index}/documents?primaryKey=id",
        [%{id: id, title: title, notes: notes}]
      )

    wait_for_index(task, 25)
    ReleaseDesk.Store.record_audit(id)
    notify!(release)
  end

  def artifact(id) do
    case ReleaseDesk.Store.release(id) do
      %{status: "completed"} ->
        %{body: notes} = s3(ExAws.S3.get_object(@bucket, key(id)))
        %{notes: notes}

      _ ->
        nil
    end
  end

  def search(term) do
    case meili("/indexes/#{@index}/search", %{q: term, limit: 20}) do
      {:ok, %{status: 200, body: %{"hits" => hits}}} -> hits
      {:ok, %{status: 404}} -> []
      _ -> raise "Meilisearch query failed"
    end
  end

  defp key(id), do: "releases/#{id}.txt"

  defp s3(operation) do
    uri = URI.parse(System.fetch_env!("DEVLINE_SEAWEEDFS_URL"))

    case ExAws.request(operation,
           scheme: "#{uri.scheme}://",
           host: uri.host,
           port: uri.port,
           region: "us-east-1",
           access_key_id: "devline",
           secret_access_key: System.fetch_env!("DEVLINE_SEAWEEDFS_PASSWORD"),
           retries: [max_attempts: 2]
         ) do
      {:ok, result} -> result
      _ -> raise "SeaweedFS request failed"
    end
  end

  defp wait_for_index(_task, 0), do: raise("Meilisearch indexing timed out")

  defp wait_for_index(task, attempts) do
    case meili!("/tasks/#{task}") do
      %{"status" => "succeeded"} ->
        :ok

      %{"status" => status} when status in ["enqueued", "processing"] ->
        Process.sleep(200)
        wait_for_index(task, attempts - 1)

      _ ->
        raise "Meilisearch indexing failed"
    end
  end

  defp meili!(path, body \\ nil) do
    case meili(path, body) do
      {:ok, %{status: status, body: response}} when status in 200..299 -> response
      _ -> raise "Meilisearch request failed"
    end
  end

  defp meili(path, body) do
    url = System.fetch_env!("DEVLINE_MEILISEARCH_URL") <> path
    headers = [{"authorization", "Bearer " <> System.fetch_env!("DEVLINE_MEILISEARCH_PASSWORD")}]
    options = [url: url, headers: headers, retry: false, redirect: false, receive_timeout: 5_000]

    if body, do: Req.post(options, json: body), else: Req.get(options)
  end

  defp notify!(%{id: id, title: title}) do
    port = System.fetch_env!("DEVLINE_MAILPIT_WEB_PORT")

    case Req.post("http://127.0.0.1:#{port}/api/v1/send",
           json: %{
             "From" => %{"Email" => "release-desk@example.test"},
             "To" => [%{"Email" => "dev@example.test"}],
             "Subject" => "Release ready: #{title}",
             "Text" => "Release #{id} was indexed and its notes are ready to download."
           },
           retry: false,
           redirect: false,
           receive_timeout: 5_000
         ) do
      {:ok, %{status: status}} when status in 200..299 -> :ok
      _ -> raise "Mailpit notification failed"
    end
  end
end
