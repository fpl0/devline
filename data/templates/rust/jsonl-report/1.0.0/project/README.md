# Logbook

A streaming request-log analyzer for finding slow endpoints and failed requests after an incident. It groups HTTP traffic by method and route, reports 4xx/5xx counts, response bytes, average/max latency, approximate p50/p95, and can enforce an error-rate threshold in a script.

```sh
./devline.sh test
./devline.sh run
./devline.sh run examples/requests.jsonl --format json > report.json
./devline.sh run examples/requests.jsonl --fail-error-rate 5
cat examples/requests.jsonl | ./devline.sh run - --format json
```

The example intentionally includes one outage and one validation error. Exit **0** means analysis succeeded and any threshold passed; **1** means invalid input/I/O; **2** means the 5xx percentage exceeded the threshold. Malformed records fail with a line number. Use `--skip-invalid` explicitly to exclude and count them; the report retains the first five diagnostics.

## Input contract

One JSON object per line: `method` (uppercase HTTP method), `route` (prefer a template such as `/orders/{id}`), `status` (100–599), `duration_ms` (0–3600000), and optional nonnegative integer `response_bytes`. Extra fields are ignored. Blank lines are ignored. A line may be at most 1 MiB.

Memory is bounded by one line and `--max-routes` (default 1000, maximum 10000), independent of log length. Histogram percentiles are **upper bounds**, using fixed buckets from 1 ms to one hour; they are not exact quantiles. Average/max latency remain numeric measurements. Reporting no records gives null latency statistics and zero error rate; automation should inspect `summary.requests` when a nonempty sample is required.

`src/lib.rs` is the reusable analysis engine; `src/main.rs` owns CLI policy and rendering. Tests cover aggregation, malformed/oversized records, empty input, histogram semantics, route cardinality, and invalid durations. Clap, Serde and all transitive sources are vendored; no network or service is needed.
