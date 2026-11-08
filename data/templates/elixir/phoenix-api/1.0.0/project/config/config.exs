import Config

config :devline_phoenix, DevlinePhoenix.Endpoint,
  adapter: Bandit.PhoenixAdapter,
  pubsub_server: DevlinePhoenix.PubSub,

  http: [ip: {127, 0, 0, 1}, port: 4000],
  server: false,
  url: [host: "localhost"]

config :phoenix, :json_library, Jason
