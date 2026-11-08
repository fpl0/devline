# __PROJECT_NAME__

A Spring Boot 4.1 task API. Create, list, read, replace, and delete tasks using JSON over HTTP. Tasks are kept in memory for the current process, so restarting the app starts with an empty list.

Use JDK 26 and Maven 3.10 or later. The bundled repository contains the dependencies and build plugins.

```sh
./devline.sh test
./devline.sh run
```

Use another terminal to exercise the API. The create response gives the new resource URL in its `Location` header. These commands use `1` for the first task on a fresh process.

```sh
base=http://127.0.0.1:8080
curl -fsS "$base/health"
curl -fsSi "$base/api/tasks" -H 'Content-Type: application/json' -d '{"title":"Read a chapter"}'
curl -fsS "$base/api/tasks"
curl -fsS "$base/api/tasks/1"
curl -fsS -X PUT "$base/api/tasks/1" -H 'Content-Type: application/json' -d '{"title":"Read two chapters","completed":true}'
curl -fsSi -X DELETE "$base/api/tasks/1"
```

A blank title produces HTTP 400 with a problem response. Fetching a missing or deleted task returns HTTP 404. To change the port for Devline runs, edit `run.url` in `.devline/config.json`. The packaged app also runs with `java -jar target/task-api.jar`.
