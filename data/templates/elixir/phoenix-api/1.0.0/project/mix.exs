defmodule DevKitPhoenix.MixProject do
  use Mix.Project

  def project do
    [
      app: :devkit_phoenix,
      version: "0.1.0",
      elixir: "~> 1.20",
      deps: dependencies()
    ]
  end

  defp dependencies do
    if File.exists?("vendor/dependencies.exs") do
      {dependencies, _} = Code.eval_file("vendor/dependencies.exs")
      dependencies
    else
      [{:phoenix, "~> 1.8.16"}, {:bandit, "~> 1.14"}, {:jason, "~> 1.4"}]
    end
  end

  def application do
    [extra_applications: [:logger], mod: {DevKitPhoenix.Application, []}]
  end
end
