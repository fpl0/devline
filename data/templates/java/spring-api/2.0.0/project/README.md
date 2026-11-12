# Issue Desk API

A persistent issue tracker for a small project. Create an issue, start work, resolve it and reopen it when a regression appears. Filter by status, search titles and descriptions, and page through the results. Each change records an audit event in the same database transaction.

```sh
./run.sh
./build-app.sh
./test-offline.sh
```

The run command prints its local address. Use that address in these examples (Java defaults to port 8080; .NET to 5080):

```sh
curl -H 'Content-Type: application/json' -d '{"title":"Offline search misses a page","description":"Reproduce with an empty cache","priority":"high"}' http://127.0.0.1:8080/api/issues
curl 'http://127.0.0.1:8080/api/issues?status=open&q=search&limit=20&offset=0'
curl -X PUT -H 'Content-Type: application/json' -d '{"title":"Offline search misses a page","description":"Investigating","priority":"high","status":"in_progress","version":1}' http://127.0.0.1:8080/api/issues/1
curl http://127.0.0.1:8080/api/issues/1/events
```

Priorities are `low`, `normal`, `high`, `urgent`. The workflow is `open → in_progress → resolved`; reopen a resolved issue before starting work again. PUT replaces editable fields and requires the version you last read. Concurrent or stale changes return 409; reload the resource before retrying. Invalid requests return 400, unknown issues return 404. There is no delete endpoint: the audit trail survives closure. No authentication is included; the app binds to loopback for local development.

Data lives in `data/` and survives restarts. Stop the process before backing up that directory. The integration test uses a temporary database, exercises actual HTTP requests and concurrent writers, then restarts the server to verify persistence. No external service or account is required. Source layers separate routing, domain rules and database operations; dependencies are bundled for offline builds.
