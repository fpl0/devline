# __PROJECT_NAME__

A task API built with Go and Chi. It includes a concurrency-safe in-memory store, JSON input validation, and lifecycle tests. Data resets when the process exits.

Requires Go 1.27.3 or newer. Run `./test-offline.sh` to build and test using the bundled `vendor/` tree, with no registry access.

```sh
GOTOOLCHAIN=local go run -mod=vendor .
curl -s http://127.0.0.1:8080/healthz
curl -i http://127.0.0.1:8080/tasks \
  -H 'Content-Type: application/json' \
  -d '{"title":"Write a test"}'
curl -s http://127.0.0.1:8080/tasks/1
curl -s -X PUT http://127.0.0.1:8080/tasks/1 \
  -H 'Content-Type: application/json' \
  -d '{"title":"Write a test","completed":true}'
curl -i -X DELETE http://127.0.0.1:8080/tasks/1
```

`GET /tasks` lists tasks in ID order. `GET /version` returns the service name. Requests with a body accept one JSON object under 16 KiB. An error response includes an `error.code` and an `error.message`.

The default listen address is `127.0.0.1:8080`. Set `LISTEN_ADDR=127.0.0.1:9000` to use another local port.
