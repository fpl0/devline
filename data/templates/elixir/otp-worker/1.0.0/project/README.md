# __PROJECT_NAME__

An OTP application with a supervised in-memory queue. Each job counts lines
and bytes in a local file, writes an atomic JSON report, and sends a result
message to the caller. The supervisor isolates task failures, so the next
job can continue. Use a durable queue for jobs that must survive restarts.

Run `./test-offline.sh` with Elixir 1.20 and Erlang/OTP 29. Start
`iex -S mix` and enqueue a sample job with
`DevlineWorker.Queue.enqueue("fixtures/example.txt", "reports/example.json")`.
Use `flush()` to see the result. The project uses the Erlang/OTP and Elixir
standard libraries, so there are no packages to fetch.
