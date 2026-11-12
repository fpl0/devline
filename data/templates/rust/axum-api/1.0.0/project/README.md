# Issue Desk · Rust

A persistent issue-tracking API using Axum, Tokio, and Serde. Create work, search/filter the backlog, progress through **open → doing → done**, reopen completed issues, and add comments to an immutable history. Version checks reject stale edits.

```sh
./devline.sh test
./devline.sh run
# In another terminal:
curl -s http://127.0.0.1:8082/issues -H 'Content-Type: application/json' -d '{"title":"Fix offline sync","priority":"high"}'
curl -s -X PUT http://127.0.0.1:8082/issues/1 -H 'Content-Type: application/json' -d '{"title":"Fix offline sync","priority":"high","status":"doing","version":1}'
curl -s http://127.0.0.1:8082/issues/1/comments -H 'Content-Type: application/json' -d '{"version":2,"body":"Reproduced on macOS"}'
curl -s 'http://127.0.0.1:8082/issues?status=doing&q=sync&limit=20&offset=0'
```

Restart the app and fetch `/issues/1` to confirm persistence. Use the newest returned version when editing or commenting. PUT replaces editable fields; omitted descriptions are cleared. Priorities are `low`, `normal`, `high`; new issues default to normal/open. Timestamps are UTC Unix milliseconds. Completed issues remain in the history rather than being deleted.

## Local storage

`ISSUE_DATA` defaults to `.data/issues.json`. Each update writes and syncs a new snapshot, atomically replaces the old file, then publishes the state in memory. A file lock rejects a second writer. Invalid stores fail startup. Stop the app before backing up its data. The local store caps itself at 10,000 issues and 1,000 events per issue, and a 128 MiB serialized snapshot; a database repository is the extension point for larger deployments.

`AXUM_LISTEN` defaults to `127.0.0.1:8082`; only loopback addresses are accepted. Devline supplies the local address. No external services are required. This local single-user example has no authentication; add it before a networked deployment.

## Architecture

`domain.rs` contains typed states, validation, and errors; `repository.rs` owns transactional snapshot persistence; `main.rs` adapts HTTP and uses Tokio's blocking pool for filesystem work, keeping it off the async reactor. Requests are limited to 32 KiB. Errors return JSON; stale updates return 409, missing IDs 404, invalid domain input 422.

Tests exercise actual loopback HTTP, state transitions, stale comments, filtering, restart persistence, exclusive writers, corrupt stores, and failed commits. All dependencies are vendored. The Go and Node examples share this product workflow while using their own runtime conventions.
