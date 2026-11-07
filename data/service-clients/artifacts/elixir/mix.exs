defmodule DevlineClients.MixProject do
  use Mix.Project
  def project do
    [app: :devline_clients, version: "1.0.0", elixir: "~> 1.20", deps: [
      {:postgrex, "== 0.22.4"},
      {:jason, "== 1.4.5"},
      {:myxql, "== 0.9.0"},
      {:amqp, "== 4.3.0"},
      {:redix, "== 1.10.0"},
      {:mongodb_driver, "== 1.6.4"},
      {:gnat, "== 1.19.1"},
      {:gen_smtp, "== 1.3.0"},
      {:req, "== 0.7.5"},
      {:ex_aws_s3, "== 2.5.9"},
      {:hackney, "== 4.9.1"},
      {:sweet_xml, "== 0.7.5"},
      {:oidcc, "== 3.9.2"}
    ]]
  end
  def application, do: [extra_applications: [:logger]]
end
