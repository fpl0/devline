import Config

if System.get_env("PHX_SERVER") == "1" do
  uri = URI.parse(System.get_env("DEVLINE_APP_URL", "http://127.0.0.1:4000"))
  {:ok, ip} = :inet.parse_address(String.to_charlist(uri.host))

  config :devline_phoenix, DevlinePhoenix.Endpoint,
    server: true,
    http: [ip: ip, port: uri.port]
end
