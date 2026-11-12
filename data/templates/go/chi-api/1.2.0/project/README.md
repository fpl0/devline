# Issue Desk · Go

A persistent issue-tracking service built with Chi. Capture work, search and filter the backlog, move issues through **open → doing → done**, reopen completed work, and keep a timestamped comment/change history. A version number prevents one editor from silently overwriting another.

## Run and use

```sh
./devline.sh test
./devline.sh run
# In a second terminal:
curl -s http://127.0.0.1:8080/issues -H 'Content-Type: application/json' -d '{"title":"Fix offline sync","priority":"high"}'
curl -s -X PUT http://127.0.0.1:8080/issues/1 -H 'Content-Type: application/json' -d '{"title":"Fix offline sync","description":"Reproduce on a train","priority":"high","status":"doing","version":1}'
curl -s http://127.0.0.1:8080/issues/1/comments -H 'Content-Type: application/json' -d '{"version":2,"body":"Reproduction confirmed"}'
curl -s 'http://127.0.0.1:8080/issues?status=doing&q=sync&limit=20&offset=0'
```

Use the latest returned version for each update or comment. PUT replaces the editable fields; omission of `description` clears it. New issues default to normal priority and open status. The root endpoint describes the API. There is no delete endpoint: completed issues remain searchable with their history.

## Data and operations

Data lives in `.data/issues.json` (`ISSUE_DATA` overrides it). Each successful change saves an atomic snapshot before returning; restart the server to see the same state. A process lock rejects a second writer to the same file. Invalid/corrupt stores fail startup rather than resetting data. Stop the app before copying the store as a backup. This deliberately small single-process local service caps the backlog at 10,000 issues, 1,000 history events per issue, and a 128 MiB serialized snapshot; migrate the repository to a database for a multi-instance deployment.

`LISTEN_ADDR` defaults to loopback port 8080; Devline supplies its selected local address. Requests have size/time limits, unknown JSON fields are rejected, and shutdown drains active requests. Errors use `{ "error": { "status": 409, "message": "..." } }`. This is a local single-user app; add authentication and authorization before exposing it to other users.

## Design and verification

- `domain.go`: issue validation and workflow rules.
- `store.go`: locking, optimistic concurrency and commit-before-publish persistence.
- `main.go`: HTTP representation, routing, pagination and graceful shutdown.
- Tests cover HTTP workflows, stale writes, invalid transitions, restart persistence, corrupt files, simultaneous creates, and failed writes leaving memory unchanged.

All dependencies are vendored. Tests and builds need no network or containers. The Rust and Node Issue Desk examples implement the same core workflow using their own ecosystem conventions.
