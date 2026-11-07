import Config

config :devkit_phoenix, DevKitPhoenix.Endpoint,
  adapter: Bandit.PhoenixAdapter,
  pubsub_server: DevKitPhoenix.PubSub,

  http: [ip: {127, 0, 0, 1}, port: 4000],
  server: System.get_env("PHX_SERVER") == "1",
  url: [host: "localhost"]

config :phoenix, :json_library, Jason

# Build the pinned SQLite source with the local C compiler. The dependency
# never looks for a precompiled NIF at build time.
config :exqlite, :force_build, true
config :ex_aws, :http_client, DevKitPhoenix.AwsHttp

# GNU Make sees relative build paths even when the project parent contains spaces.
config :exqlite, :make_env, fn ->
  %{"MIX_APP_PATH" => Path.relative_to(Mix.Project.app_path(), File.cwd!(), force: true)}
end
