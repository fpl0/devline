# __PROJECT_NAME__

An offline FastAPI starting point with service clients already locked and cached. Prepare CPython 3.15 on Apple Silicon macOS first. The bundle contains native macOS wheels for PostgreSQL and cryptography. SQLite comes with CPython.

```sh
./setup-offline.sh
.venv/bin/python -m uvicorn app:app --host 127.0.0.1 --port 8000
curl -fsS http://127.0.0.1:8000/health
```

Use `./test-offline.sh` to install a fresh temporary environment and exercise the HTTP route, settings, logging, SQLite, and bundled clients. Network clients use local configuration, deferred connections, or stub responses in these tests.

| Use | Included |
| --- | --- |
| Web and HTTP | FastAPI, Uvicorn, HTTPX |
| Databases | Psycopg binary, PyMySQL, CPython SQLite |
| Cache and messaging | redis-py, Pika, nats-py |
| S3-compatible storage | Boto3 and Botocore |
| Authentication | PyJWT and cryptography |
| Configuration, logs, and tests | pydantic-settings, structlog, pytest |

Configure real connections using your project's local endpoints and credentials from Devline. The package inventory and SHA-256 hashes are in `requirements.txt`. Wheel files retain their upstream notices. The offline setup requires only the supported interpreter and the bundled wheels.
