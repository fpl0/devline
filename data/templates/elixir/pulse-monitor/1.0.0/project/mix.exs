defmodule Pulse.MixProject do
  use Mix.Project

  def project,
    do: [
      app: :pulse_monitor,
      version: "1.0.0",
      elixir: "~> 1.20",
      deps: [],
      escript: [main_module: Pulse.CLI]
    ]

  def application, do: [extra_applications: [:logger, :inets, :ssl]]
end
