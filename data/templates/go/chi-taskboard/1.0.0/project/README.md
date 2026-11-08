# Taskboard

A local task board built with Go, Chi, PostgreSQL, and Redis. Add, edit, complete, and delete tasks in a browser. PostgreSQL stores the tasks and Redis caches reads for 30 seconds. The footer identifies where a read came from. Cache errors fall back to PostgreSQL.

## Run locally

Prepare the exact service releases once while connected. Devline keeps independent archives for both images.

```sh
devline infra prepare postgres@18.6
devline infra prepare redis@8.10.1
```

In the generated project folder, run the board. Devline starts both configured services and runs the vendored Go app offline.

```sh
./devline.sh run
```

Open [the task board](http://127.0.0.1:7404). Add a task and reload to see Redis serve the list. Toggle the task to invalidate the cache. Change `run.url` in `.devline/config.json` to select another loopback port. Devline supplies the service URLs with the generated ports and credentials.

Run `./devline.sh test` for the generated project's offline tests. With both services running, `./devline.sh smoke` checks CRUD, PostgreSQL persistence, and Redis hits and invalidation. `./devline.sh services down` stops the services and retains their data.
