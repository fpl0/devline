# Watchtower

A local HTTP uptime monitor with a server-rendered dashboard. It checks configured services concurrently, shows latency and sampled availability, retains the last 100 results per target across restarts, and exposes the same observations as JSON.

```sh
./devline.sh test
./devline.sh run
# Open http://127.0.0.1:8080
```

The default dashboard monitors its own healthy and deliberately failing demonstration endpoints. Within a second you can see a green service, an outage, HTTP statuses, latency, and history. The page refreshes every ten seconds. No containers or external network are needed.

## Monitor your own services

Copy `examples/monitors.json`, set meaningful names, URLs, expected HTTP statuses, an interval, and a timeout; then run:

```sh
WATCHTOWER_CONFIG=examples/monitors.json ./devline.sh run
curl -s http://127.0.0.1:8080/api/monitors
```

`WATCHTOWER_DATA` changes `.data/history.json`; `LISTEN_ADDR` changes the default `127.0.0.1:8080`. Devline supplies its selected listen address automatically. The default demo configuration follows that address; an explicit config uses its exact URLs. For remote monitoring outside Devline's offline sandbox, use `go run .` directly.

History keys include the target name and URL, so repointing a target does not mix old results into its availability. Stop the app before making a backup. Corrupt history fails startup. Use one process per data file. A persistence failure is logged; the dashboard retains observations in memory until a successful save. Availability is the fraction of successful checks in the retained window, not wall-clock uptime or an SLA.

## Design

`monitor.go` holds validation, a four-worker bounded polling round, HTTP timeouts, history and atomic snapshot persistence. `main.go` owns graceful shutdown and the HTML/JSON interfaces; `html/template` escapes monitor content. Redirects are recorded as their original HTTP status. Polls never overlap, slow targets are bounded by timeout, and cancellation stops work. Configure only services you are authorized to check.

Tests use ephemeral local HTTP servers to cover healthy/outage responses, timeouts, redirects, persistence, corrupt history, escaped HTML, and bounded retention. The application uses Go's standard library only.
