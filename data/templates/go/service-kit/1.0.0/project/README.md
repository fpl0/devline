# Go Service Kit

Start a Chi HTTP service with SQLite and local clients for common databases, messaging and object storage. The Notes API uses SQLite immediately. Other clients are initialized only when your code requests them.

```sh
GOWORK=off GOTOOLCHAIN=local GOPROXY=off go run -mod=vendor .
curl http://127.0.0.1:8080/healthz
curl -H 'Content-Type: application/json' -d '{"text":"Offline notes"}' http://127.0.0.1:8080/notes
curl http://127.0.0.1:8080/notes
```

`SERVICE_LISTEN` changes the loopback address. `SERVICE_DB` changes the SQLite file path. Local service settings are passed to the constructors in `clients.go`. See `offline_test.go` for concrete PostgreSQL, MySQL, Redis, RabbitMQ, NATS, and S3 examples. Pass explicit local endpoints when working offline. Prepare and run their service images from Devline when an actual connection is needed.

`./devkit-test.sh` exercises the HTTP Notes flow on in-memory SQLite, constructs each client without dialing remote services, and signs and verifies JWTs. Go 1.27.3 and `CGO_ENABLED=0` are used for qualification. `vendor/modules.txt` lists the complete source closure and its retained licenses.

See `dependency-coverage.json` for the exact shipped package versions and size.
