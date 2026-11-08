defmodule DevlinePhoenix.AwsHttp do
  @behaviour ExAws.Request.HttpClient

  @impl true
  def request(method, url, body, headers, _options) do
    case Req.request(method: method, url: url, body: body, headers: headers,
           decode_body: false, retry: false, redirect: false) do
      {:ok, response} ->
        {:ok, %{status_code: response.status, body: response.body, headers: response.headers}}

      {:error, reason} ->
        {:error, reason}
    end
  end
end
