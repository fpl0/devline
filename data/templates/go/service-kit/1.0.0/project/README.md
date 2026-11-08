# Release Rehearsal

Run a local rehearsal with the generated Go service and eight Devline infrastructure services. PostgreSQL records the release, SeaweedFS stores its notes as an artifact, RabbitMQ queues a delivery, and NATS broadcasts a release event. MySQL records an audit row, Meilisearch indexes the notes, and Mailpit captures the notification. Redis caches completed receipts.

## Run locally

Create the project from Devline with its default services. You need a running Docker engine and the eight pinned images in your Devline library. Prepare them while connected or import their offline archives. Devline starts the stack, passes the local service settings, and runs the application from its vendored dependencies.

```sh
./devline.sh start
./devline.sh status
```

Open [the API index](http://127.0.0.1:7406). Pick a new release ID for each rehearsal. Use lowercase letters, digits, and hyphens.

```sh
url=http://127.0.0.1:7406
id=demo-$(date +%s)
curl -fsS "$url/releases/$id" -H 'Content-Type: application/json' \
  -d '{"notes":"Verify the lighthouse release locally"}'
curl -fsi "$url/releases/$id"
curl -fsi "$url/releases/$id"
curl -fsS "$url/releases/$id/artifact"
curl -fsS "$url/search?q=lighthouse"
```

The POST returns a completed receipt after the two brokers deliver their messages, the search index commits, and the inbox accepts the notification. The first receipt read reports `X-Release-Source: postgres`. A second read within 30 seconds reports `redis`. The artifact returns the original notes. Follow `mail_inbox` in the API index to see the captured message.

The rehearsal claims a release ID before contacting the other services. A failure returns the failed stage and leaves an incomplete receipt. Use a fresh ID after checking the services. A duplicate request for the same ID returns 409 to avoid repeating the notification. The local inbox uses example addresses and retains the message for inspection.

The original SQLite notes workflow is available at `GET /notes` and `POST /notes` with a JSON `text` field. It also runs when the app starts without the managed service environment. `./devline.sh test` exercises this path with the bundled dependencies and no services running.

Use `./devline.sh stop` to stop the app and `./devline.sh services down` to stop its containers while retaining their data. To select another local app port, edit `run.url` in `.devline/config.json`.
