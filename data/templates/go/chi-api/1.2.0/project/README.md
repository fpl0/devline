# __PROJECT_NAME__

A task API built with Go and Chi. It includes a concurrency-safe in-memory store, JSON input validation, and lifecycle tests. Data resets when the process exits.

Requires Go 1.27.3 or newer. Run `./test-offline.sh` to build and test using the bundled `vendor/` tree, with no registry access.

```sh
./devline.sh start
curl -s http://127.0.0.1:7403/healthz
curl -i http://127.0.0.1:7403/tasks \
  -H 'Content-Type: application/json' \
  -d '{"title":"Write a test"}'
curl -s http://127.0.0.1:7403/tasks/1
curl -s -X PUT http://127.0.0.1:7403/tasks/1 \
  -H 'Content-Type: application/json' \
  -d '{"title":"Write a test","completed":true}'
curl -i -X DELETE http://127.0.0.1:7403/tasks/1
```

`GET /tasks` lists tasks in ID order. `GET /version` returns the service name. Requests with a body accept one JSON object under 16 KiB. An error response includes an `error.code` and an `error.message`.

Use `./devline.sh stop` to stop the app. Change `run.url` in `.devline/config.json` to select another loopback port.
