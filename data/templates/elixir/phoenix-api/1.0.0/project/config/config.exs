import Config

config :devkit_phoenix, DevKitPhoenix.Endpoint,
  adapter: Bandit.PhoenixAdapter,
  pubsub_server: DevKitPhoenix.PubSub,

  http: [ip: {127, 0, 0, 1}, port: 4000],
  server: System.get_env("PHX_SERVER") == "1",
  url: [host: "localhost"]

config :phoenix, :json_library, Jason
