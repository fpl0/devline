# Pulse Monitor

An Elixir HTTP monitoring CLI. Check several endpoints concurrently, compare their status codes with expectations, record failures and latency, and summarize recent history. The default example starts a real temporary loopback HTTP server, so it works offline without another application.

## Run

```sh
./test-offline.sh
./run.sh
```

Edit `monitors.json` to point at your local applications. In a generated project:

```sh
./devline.sh exec elixir -S mix run pulse.exs check --config monitors.json --concurrency 4 --timeout 3000
./devline.sh exec elixir -S mix run pulse.exs history
```

A check exits **0** when every endpoint is healthy, **2** when any check fails, and **1** for configuration or storage errors. Each result contains the name, URL, expected and actual status, elapsed milliseconds, and any connection error. Redirects are not followed: a redirect is itself the observed status.

```json
[{"name":"Order Ledger","url":"http://127.0.0.1:7441/healthz","expected_status":200}]
```

Names must be unique. Only HTTP(S) URLs without embedded credentials are accepted. HTTPS verifies certificates and hostnames. Concurrency is bounded to 1–32 tasks; timeouts are 100–60,000 ms. `--history PATH` selects another history file; the default is `data/history.json`.

## Design and tests

`Pulse.Monitor` validates targets and uses `Task.async_stream` with bounded concurrency, ordered results, and task timeouts. HTTP is supplied by OTP's `:httpc`; failed requests remain report rows instead of terminating the whole run. `Pulse.History` atomically stores the latest 500 runs and computes per-monitor check counts, healthy counts, and nearest-rank p95 latency. The p95 includes failed checks, so it describes attempts rather than successful-response latency alone.

Run one monitor process per history file. The history lock coordinates writes inside a node, not between independent operating-system processes. Invalid history is reported and preserved; choose a new history path after inspecting it rather than silently resetting it.

Tests exercise real loopback HTTP failures, refused connections, task timeouts, concurrency limits, ordering, URL validation, persistence, summaries, and corrupt-history preservation. No package download is required. A natural extension is a supervised periodic runner with configurable intervals; keep the finite checker reusable for tests and CI.
