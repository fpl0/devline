# __PROJECT_NAME__

A Phoenix and Bandit API with a Release Desk backed by seven local services.
The starter requires Elixir 1.20.5, Erlang/OTP 29, and the Apple Command Line
Tools. The latter build the pinned SQLite C code for the vendored package
tests.

## Release Desk

Use a running Docker engine and the pinned images for PostgreSQL, MySQL,
RabbitMQ, Redis, SeaweedFS S3, Meilisearch, and Mailpit. Devline prepares
missing archives when you create the starter while connected. If you create
it offline, prepare or import the archives first. Run `./devline.sh run`
from the generated project, or use **Run** in Devline. The app starts with the
selected local service ports and credentials.

Submit release notes.

```sh
base=http://127.0.0.1:7500
curl -fsS "$base/releases" -H 'Content-Type: application/json' \
  -d '{"title":"Release Atlas","notes":"The offline build is ready."}'
```

Copy the returned ID into `id` and follow the result.

```sh
id=PASTE_THE_RELEASE_ID
curl -fsS "$base/releases/$id"
```

Repeat the request until the status is `completed`. PostgreSQL stores the
submission and pending dispatch. RabbitMQ delivers it to a worker. The worker
stores the notes in SeaweedFS, indexes them in Meilisearch, writes a MySQL audit
row, and posts a notification to Mailpit. Refresh the status to see a
Redis cache hit.

```sh
curl -fsS "$base/releases/$id/artifact"
curl -fsS "$base/releases/$id/audit"
curl -fsS "$base/search?q=Atlas"
```

In the project's **Services** view, open the Mailpit web inbox to read
**Release ready: Release Atlas**. Use `./devline.sh test` to verify the app
offline before starting the services. Use `./devline.sh services down` after
stopping the app to stop the containers and keep their data.

The worker acknowledges the job after processing. Artifact and index updates
and audit inserts can be repeated. If processing is interrupted just after
sending a notification, the retry can deliver it again.

## Standalone Phoenix

Without the full service configuration, the default launcher still serves
`GET /healthz` using Phoenix and Bandit. The vendored Hex sources also include
Ecto SQL, SQLite, and Joken/JOSE for your own routes. Exact versions and Hex
checksums are in `vendor/upstream-lock.exs`.

Devline compiles the vendored Ranch Erlang source with Rebar and uses relative
SQLite build paths, including when the project folder contains spaces.
