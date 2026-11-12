# Report Queue

A durable local job queue built with OTP supervisors, a GenServer, and supervised tasks. Enqueue category-report jobs, process up to two concurrently, retry failures with backoff, and inspect the result or recover failed work.

## Run the example

```sh
./test-offline.sh
./run.sh
```

The example enqueues the bundled sales dataset, processes it, and saves a report under `data/reports`. In a generated project, try the explicit lifecycle:

```sh
./devline.sh exec elixir -S mix run queue.exs enqueue weekly fixtures/sales.json data/weekly.json
./devline.sh exec elixir -S mix run queue.exs list
./devline.sh exec elixir -S mix run queue.exs work
```

A repeated key with identical input/output returns the original job. Reusing it for another request is rejected. Existing output files and output paths owned by another job are protected. To see a failure, enqueue a nonexistent input, run `work`, then inspect `list`. Correct the input and run `retry ID`, followed by `work` again.

The journal is `data/queue.json`; `REPORT_QUEUE_DATABASE` chooses another path. Use one queue process per journal and run these commands sequentially. The application models one local supervised node, not a distributed broker.

## Dataset and reports

Input is a JSON array of objects with a nonblank `category` and integer `amount_cents`. Negative amounts represent refunds. Reports contain per-category counts, totals, minimums, and maximums plus the total for the dataset. Inputs are limited to 10 MB and 100,000 rows. A failed report never publishes a partially written output.

## Reliability model

The GenServer serializes durable state changes. Jobs move through `queued`, `running`, `completed`, or `failed`; each attempt is persisted before its task starts. A task crash is monitored without crashing the queue. After three failed attempts the job needs an explicit retry. Backoff starts at 500 ms and doubles. Interrupted `running` work returns to the queue when the journal is reopened.

The application supervisor restarts the queue and its task supervisor together if the queue itself fails, preventing orphaned work from remaining under the old task supervisor. Delivery is at least once: a crash after writing the report but before saving completion may rerun the job. Reports use atomic replacement and deterministic contents to make that safe.

`lib/devline_worker/report.ex` is the pure report domain plus file boundary. `queue.ex` implements scheduling and persistence; `cli.ex` exposes commands. Tests cover idempotency, persistence, concurrent task supervision, repeated worker crashes, manual retry, interrupted-job recovery, invalid input, and atomic output. The package has no external dependencies.
