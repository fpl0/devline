# Issue Desk · Node.js

A local issue tracker with a Fastify API, SQLite persistence, browser dashboard, and durable event stream. Capture issues, search/filter them, progress through **open → doing → done**, reopen work, and add comments to its history.

```sh
./devline.sh test
./devline.sh run
# Open http://127.0.0.1:7481
```

Create an issue in the right-hand form, select its title, edit it, move to doing/done, and add a comment. Open a second browser tab to see the list update through Server-Sent Events. An editor keeps the version it opened: a stale save returns a conflict instead of overwriting another tab. **Reload issue** fetches the latest version. Restart the application to confirm the database persists.

## HTTP workflow

```sh
curl -s http://127.0.0.1:7481/issues -H 'Content-Type: application/json' -d '{"title":"Fix offline sync","priority":"high"}'
curl -s 'http://127.0.0.1:7481/issues?status=open&q=sync&limit=20&offset=0'
curl -s http://127.0.0.1:7481/issues/1/comments -H 'Content-Type: application/json' -d '{"version":1,"body":"Reproduction confirmed"}'
curl -N 'http://127.0.0.1:7481/events/stream?after=0'
```

`PUT /issues/:id` replaces title, description, priority and status using the current `version`. `GET /events?after=ID` reads persisted events in batches of 100. The SSE endpoint polls those same events once per second; a disconnected client can reconnect with its last cursor in `after`. Comments and issue changes increment the version in one SQLite transaction. Completed issues remain searchable; there is no hard-delete endpoint.

## Persistence and structure

`ISSUE_DATA` defaults to `.data/issues.sqlite`. SQLite uses WAL, foreign keys, a schema version, prepared statements and immediate write transactions. For a backup, stop the app and copy the database and any `-wal`/`-shm` companions together. The built-in Node SQLite API needs Node 26+. No database service or network is needed.

`repository.mjs` owns SQL and workflow rules; `app.mjs` validates HTTP input with Zod and maps errors; `server.mjs` owns startup and signal shutdown. `web/index.html` is a dependency-free operator UI. HTTP bodies are bounded at 32 KiB; SSE clients are capped at 100 and slow clients disconnected. The database operations are synchronous and intentionally small: move to an asynchronous database or worker pool for heavy queries. This is a loopback-only single-user app without authentication.

Tests cover API and database workflows, stale edits, invalid transitions, validation, pagination/search escaping, restart persistence, rollback and event replay. The Go and Rust examples share the core Issue Desk workflow using their own ecosystem conventions.
