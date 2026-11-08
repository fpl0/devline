# __PROJECT_NAME__

A FastAPI API for triaging support tickets. Add and read tickets with PostgreSQL, then index them for search, mark them reviewed, export a copy, and send a notification through the local services configured by Devline.

Requires CPython 3.15 on Apple Silicon macOS. The wheelhouse includes Apple Silicon builds of the PostgreSQL and cryptography clients.

```sh
./run.sh
```

For background mode, use `./devline.sh start`. Devline starts the pinned services, passes their generated connection settings to the app, and blocks internet access. The API listens on http://127.0.0.1:7443.

## Triage a ticket

These commands use `jq` to extract IDs from responses. You can also copy them from the returned JSON.

```sh
base=http://127.0.0.1:7443
id=$(curl -fsS "$base/tickets" -H 'Content-Type: application/json' \
  -d '{"title":"Printer jam","details":"Paper is stuck in the rear tray."}' | jq -r .id)
ticket="$base/tickets/$id"

curl -fsS "$base/tickets"          # List up to 100 saved PostgreSQL tickets
curl -fsS "$ticket"                # Read the record and populate the Redis cache
curl -fsS "$ticket"                # Read the same immutable ticket from Redis
curl -fsS -X POST "$ticket/index"  # Send the record to Meilisearch
```

Copy the returned `task` number and check `GET /search/tasks/<task>`. When its status is `succeeded`, continue with the search and the other triage actions.

```sh
curl -fsS "$base/search?q=printer"                 # Search the Meilisearch index
curl -fsS -X POST "$ticket/activity"               # Publish a review event to NATS JetStream
curl -fsS "$ticket/activity"                       # Read the latest review event
curl -fsS -X POST "$ticket/export"                 # Save a JSON snapshot in SeaweedFS S3
curl -fsS "$ticket/export"                         # Download the saved snapshot
curl -fsS -X POST "$ticket/notify"                  # Confirm a durable RabbitMQ job
curl -fsS -X POST "$base/notifications/next"        # Process one job and send it to Mailpit
```

Open Mailpit from the project's Services panel to read the test email. The notification step acknowledges the RabbitMQ job after SMTP accepts it. A failure before acknowledgement can redeliver the job.

## Offline tests and storage

Run `./test-offline.sh` to install the pinned dependencies from the bundled wheelhouse and test the API and client setup without containers. When no PostgreSQL service is configured, the core ticket routes use `storage/support.sqlite3` locally. The live integration routes require their respective Devline services and report HTTP 503 if a service is unavailable.

Use `./devline.sh stop` to stop the background app while preserving its service data. Use `./devline.sh services down` to stop its services.
