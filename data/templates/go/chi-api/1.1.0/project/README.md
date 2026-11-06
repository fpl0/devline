# __PROJECT_NAME__

A small HTTP API using Chi. Requires Go 1.27 or newer.

Run `./test-offline.sh` to compile and test using the included `vendor/` directory.
Run `GOTOOLCHAIN=local go run -mod=vendor .` to serve `GET /healthz` on port 8080.
The `GET /version` endpoint returns the configured service name.
