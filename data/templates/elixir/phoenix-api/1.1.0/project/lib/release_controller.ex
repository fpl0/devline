defmodule ReleaseDesk.Controller do
  use Phoenix.Controller, formats: [:json]

  alias ReleaseDesk.{Delivery, Store}

  def index(conn, _params) do
    if Store.enabled?() do
      json(conn, %{
        app: "Release Desk",
        submit: "POST /releases with {title, notes}",
        results: ["GET /releases/:id", "GET /releases/:id/artifact", "GET /releases/:id/audit"],
        discovery: ["GET /search?q=title"]
      })
    else
      json(conn, %{app: "Phoenix HTTP API", health: "/healthz"})
    end
  end

  def submit(conn, params) do
    with :ok <- enabled(),
         {:ok, release} <- validate(params) do
      id = Store.submit(release)

      conn
      |> put_resp_header("location", "/releases/#{id}")
      |> put_status(:accepted)
      |> json(%{id: id, status: "queued"})
    else
      :disabled ->
        error(conn, :service_unavailable, "Start the Release Desk services through Devline.")

      :invalid ->
        error(conn, :bad_request, "Provide a title and release notes.")
    end
  end

  def show(conn, %{"id" => id}), do: by_id(conn, id, &Store.status/1)
  def audit(conn, %{"id" => id}), do: by_id(conn, id, &Store.audit/1)
  def artifact(conn, %{"id" => id}), do: by_id(conn, id, &Delivery.artifact/1)

  def search(conn, params) do
    with :ok <- enabled(),
         q when is_binary(q) <- params["q"],
         term <- String.trim(q),
         true <- String.length(term) in 1..120 do
      json(conn, %{hits: Delivery.search(term)})
    else
      :disabled ->
        error(conn, :service_unavailable, "Start the Release Desk services through Devline.")

      _invalid ->
        error(conn, :bad_request, "Provide a search term of 1–120 characters.")
    end
  end

  def validate(%{"title" => title, "notes" => notes})
      when is_binary(title) and is_binary(notes) do
    title = String.trim(title)

    if String.length(title) in 1..120 and
         not String.contains?(title, ["\r", "\n"]) and String.length(notes) in 1..16_000 do
      {:ok, %{title: title, notes: notes}}
    else
      :invalid
    end
  end

  def validate(_params), do: :invalid

  defp by_id(conn, id, action) do
    with :ok <- enabled(),
         true <- byte_size(id) == 36,
         {:ok, uuid} <- Ecto.UUID.cast(id),
         result when not is_nil(result) <- action.(uuid) do
      json(conn, result)
    else
      :disabled ->
        error(conn, :service_unavailable, "Start the Release Desk services through Devline.")

      _missing ->
        error(conn, :not_found, "Release result is unavailable.")
    end
  end

  defp enabled, do: if(Store.enabled?(), do: :ok, else: :disabled)
  defp error(conn, status, message), do: conn |> put_status(status) |> json(%{error: message})
end
