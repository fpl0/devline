import Config

config :devline_phoenix, DevlinePhoenix.Endpoint,
  adapter: Bandit.PhoenixAdapter,
  pubsub_server: DevlinePhoenix.PubSub,
  http: [ip: {127, 0, 0, 1}, port: 4000],
  server: false,
  url: [host: "127.0.0.1"],
  check_origin: ["http://127.0.0.1:4000", "http://localhost:4000"]

config :phoenix, :json_library, Jason
