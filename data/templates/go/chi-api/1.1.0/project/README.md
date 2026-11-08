# __PROJECT_NAME__

A small HTTP API using Chi. Requires Go 1.27 or newer.

Run `./test-offline.sh` to compile and test using the included `vendor/` directory.
Run `./devline.sh run` in a generated project and open [the API index](http://127.0.0.1:7402). It links to `GET /healthz`.
The `GET /version` endpoint returns the configured service name.
