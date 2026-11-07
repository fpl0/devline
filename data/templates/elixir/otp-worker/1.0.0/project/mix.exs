defmodule DevlineWorker.MixProject do
  use Mix.Project

  def project, do: [app: :devline_worker, version: "1.0.0", elixir: "~> 1.20", deps: []]

  def application do
    [extra_applications: [:logger], mod: {DevlineWorker.Application, []}]
  end
end
