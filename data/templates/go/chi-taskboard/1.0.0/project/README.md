# Taskboard

A local task board built with Go, Chi, PostgreSQL, and Redis. Add, edit, complete, and delete tasks in a browser. PostgreSQL stores the tasks and Redis caches reads for 30 seconds. The footer identifies where a read came from. Cache errors fall back to PostgreSQL.

## Run locally

Prepare the exact service releases once while connected. Devline keeps independent archives for both images.

```sh
devline infra prepare postgres@18.6
devline infra prepare redis@8.10.1
```

In the generated project folder, configure services if the desktop app did not already do that for you. Start them offline and run the vendored Go app.

```sh
devline infra configure . --service postgres@18.6 --service redis@8.10.1
devline --offline infra run .
GOWORK=off GOTOOLCHAIN=local GOPROXY=off go run -mod=vendor .
```

Open `http://127.0.0.1:8080`. The app reads the allocated ports and passwords from `.devline-infra/credentials.env`. You can set `TASKBOARD_LISTEN` to another loopback address or supply `TASKBOARD_DATABASE_URL` and `TASKBOARD_REDIS_URL` for your own loopback services. Keep the credentials file local.

Use `./devkit-test.sh` for the generated project's offline test, or `./test-offline.sh` in the starter source. With both services running, `./smoke-infra.sh` checks CRUD, PostgreSQL persistence, and Redis hits and invalidation. `devline infra stop .` stops services while retaining the database volume.
