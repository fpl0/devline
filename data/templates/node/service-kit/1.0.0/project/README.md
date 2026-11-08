# __PROJECT_NAME__

A Fastify Release Desk backed by eight local services, plus a standalone
SQLite task API. The npm packages and licenses are bundled for Node.js 26+
on macOS arm64.

## Release Desk

Use a running Docker engine and the pinned images for PostgreSQL, MySQL,
RabbitMQ, Redis, SeaweedFS S3, Meilisearch, Mailpit, and NATS. Devline prepares
missing archives when you create the starter while connected. If you create
it offline, prepare or import the archives first. Run `./devline.sh run`
from the generated project, or use **Run** in Devline. The app starts with the
selected local service ports and credentials.

Submit release notes.

```sh
base=http://127.0.0.1:7481
curl -fsS "$base/releases" -H 'Content-Type: application/json'   -d '{"title":"Release Atlas","notes":"The offline build is ready."}'
```

Copy the returned ID into `id` and follow the result.

```sh
id=PASTE_THE_RELEASE_ID
curl -fsS "$base/releases/$id"
```

Repeat the request until the status is `completed`. PostgreSQL stores the
submission and pending dispatch. RabbitMQ delivers it to a worker. The worker
stores the notes in SeaweedFS, indexes them in Meilisearch, writes a MySQL audit
row, and posts a notification to Mailpit, and publishes a NATS completion event. Refresh the status to see a
Redis cache hit.

```sh
curl -fsS "$base/releases/$id/artifact"
curl -fsS "$base/releases/$id/audit"
curl -fsS "$base/search?q=Atlas"
curl -fsS "$base/events"
```

In the project's **Services** view, open the Mailpit web inbox to read
**Release ready: Release Atlas**. Use `./devline.sh test` to verify the app
offline before starting the services. Use `./devline.sh services down` after
stopping the app to stop the containers and keep their data.

The worker acknowledges the job after processing. Artifact and index updates
and audit inserts can be repeated. If processing is interrupted just after
sending a notification or publishing an event, the retry can deliver it again.

## SQLite task API

`./devline.sh exec node src/server.mjs` starts the standalone API at
http://127.0.0.1:3000. POST `{"title":"Build offline"}` to `/tasks`,
GET `/tasks`, and DELETE `/tasks/1` to try the local workflow.
The tasks persist in `data/tasks.sqlite`. `.env.example` lists its settings.
The default launcher also uses SQLite if the full Release Desk stack is absent.

Build a new template release in Devline while connected to refresh the
dependency set. The release recipe verifies its offline tests before
publishing the new version.
