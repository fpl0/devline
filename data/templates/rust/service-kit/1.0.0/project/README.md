# Rust Service Kit

Start an Axum HTTP service with SQLite and reusable client constructors for databases, messaging and object storage. The Notes API runs with SQLite and needs no background services.

```sh
cargo run --frozen
curl http://127.0.0.1:8080/healthz
curl -H 'Content-Type: application/json' -d '{"text":"Offline notes"}' http://127.0.0.1:8080/notes
curl http://127.0.0.1:8080/notes
```

Use `SERVICE_LISTEN` to select another loopback address, `SERVICE_DB` to set the SQLite path, and `RUST_LOG` for tracing. `src/clients.rs` exposes lazy PostgreSQL/MySQL pools, Redis, RabbitMQ, NATS, S3 and HTTP clients plus JWT helpers. `tests/offline.rs` shows local service settings. Supply a local endpoint and explicit credentials for offline use. `devline infra` prepares and runs the matching services.

`./devkit-test.sh` runs HTTP and SQLite persistence tests, constructs the other clients without remote sockets, and verifies JWT signature handling. Builds use the checked-in Cargo lock, vendored crates, Rust 1.99.1, and the installed macOS command-line build tools for native SQLite and TLS code.

See `dependency-coverage.json` for the exact shipped package versions and size.
