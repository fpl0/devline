defmodule DevlinePhoenix.MixProject do
  use Mix.Project

  def project do
    [
      app: :devline_phoenix,
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
      [
        {:phoenix, "~> 1.8.16"},
        {:bandit, "~> 1.14"},
        {:jason, "~> 1.4"},
        {:ecto_sql, "~> 3.14"},
        {:postgrex, "~> 0.22"},
        {:myxql, "~> 0.9"},
        {:ecto_sqlite3, "~> 0.26"},
        {:redix, "~> 1.10"},
        {:amqp, "~> 4.3"},
        {:ex_aws, "~> 2.7"},
        {:ex_aws_s3, "~> 2.5"},
        {:sweet_xml, "~> 0.7"},
        {:req, "~> 0.7"},
        {:joken, "~> 2.7"}
      ]
    end
  end

  def application do
    [extra_applications: [:logger], mod: {DevlinePhoenix.Application, []}]
  end
end
