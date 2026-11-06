# __PROJECT_NAME__

A FastAPI task API with a thread-safe in-memory store, validated request models, and local Swagger UI. Data resets when the process exits.

Requires CPython 3.15.2 or a compatible 3.15 patch release on Apple Silicon macOS. Run `./test-offline.sh` to restore pinned wheels locally and run the API tests.

```sh
.venv/bin/python -m uvicorn app:app --host 127.0.0.1 --port 8000
curl -s http://127.0.0.1:8000/healthz
curl -i http://127.0.0.1:8000/tasks \
  -H 'Content-Type: application/json' \
  -d '{"title":"Write a test"}'
curl -s http://127.0.0.1:8000/tasks/1
curl -s -X PUT http://127.0.0.1:8000/tasks/1 \
  -H 'Content-Type: application/json' \
  -d '{"title":"Write a test","completed":true}'
curl -i -X DELETE http://127.0.0.1:8000/tasks/1
```

Use `GET /tasks` to list tasks. Open http://127.0.0.1:8000/docs for the interactive API reference. All Swagger UI assets are bundled. Validation failures and missing tasks return JSON with an `error.code` and `error.message`.
