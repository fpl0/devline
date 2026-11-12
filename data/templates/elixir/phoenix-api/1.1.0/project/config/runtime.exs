import Config

if System.get_env("PHX_SERVER") == "1" do
  uri = URI.parse(System.get_env("DEVLINE_APP_URL", "http://127.0.0.1:4000"))

  config :devline_phoenix, DevlinePhoenix.Endpoint,
    server: true,
    http: [ip: {127, 0, 0, 1}, port: uri.port],
    url: [host: "127.0.0.1", port: uri.port],
    check_origin: ["http://127.0.0.1:#{uri.port}", "http://localhost:#{uri.port}"]
end
