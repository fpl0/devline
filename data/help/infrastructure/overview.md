# Prepare local infrastructure

## Prepare before disconnecting

1. Start the Docker engine on this Mac. Confirm it can run Linux arm64 containers.
2. Open Infrastructure in Devline. Prepare only the services your project needs.
3. Start each prepared service and wait for its readiness check. Open the service details to copy its actual host port, connection string, and generated credential.
4. Prepare the project starter, SDK, and dependency cache separately. A database image cannot replace the application's compiler or packages.
5. Read the service's Devline guide and try its example while the engine is available. Where available, save the publisher's reference separately.

The listed compressed sizes describe the selected platform's layers. The engine may reuse shared layers, and unpacked images and persistent data need additional disk space.

## Set up a project in the terminal

Run these commands with a project directory that already exists. Prepare the pinned images while connected. The remaining commands use local files and the local Docker engine.

```sh
devline infra catalog
devline infra prepare postgres@18.6
devline infra prepare rabbitmq@4.3.7
devline infra configure ./my-app --service postgres@18.6 --service rabbitmq@4.3.7
devline infra run ./my-app
devline infra status --project ./my-app
```

Run `devline infra credentials ./my-app` in a private terminal when you need the generated passwords and connection strings. Run `devline infra logs ./my-app` to inspect startup. When finished, `devline infra stop ./my-app` stops the project services and keeps the named volumes.

## Use the service offline

Starting a prepared service uses its cached image. Keep Docker's local VM running. Stop unused services to release their CPU and memory. A stopped service keeps its named volume, so restart the same instance to resume work. If the default port is busy, use a different loopback port and update the application's environment.

Devline's port mappings are for this Mac. Use `127.0.0.1` from an application running directly on macOS. A separate container has its own loopback interface. From Docker Desktop, use `host.docker.internal` for host-published ports when that route is available. For direct container-to-container traffic, connect both containers to the same user-defined Docker network and use the service's network name and container port. See [application connections](application-connections.md).

## Know what you have

| Service | First task | Default host ports |
| --- | --- | --- |
| PostgreSQL | Test relational queries and transactions | 5432 |
| MySQL | Test MySQL schemas and migrations | 3306 |
| RabbitMQ | Publish and consume AMQP messages | 5672, 15672 |
| Redis | Cache values and test expiry | 6379 |
| MongoDB | Store JSON-like documents | 27017 |
| NATS | Publish subjects and persist JetStream messages | 4222, 8222 |
| Mailpit | Capture email without delivering it | 1025, 8025 |
| Meilisearch | Index and search a local catalog | 7700 |
| SeaweedFS S3 | Upload and fetch objects over S3 | 8333 |
| Keycloak | Exercise OAuth and OpenID Connect flows | 8180, 9100 |

## Move a toolbox to another Mac

An image archive contains image layers and tags. The service's user data lives in its named volume. Save images and back up data separately, verify both on the destination, and keep credentials with a secure local copy of your project configuration. Follow [image and data portability](portable-toolbox.md).

## Troubleshooting

- A missing image means the selected service revision was never prepared on this engine or its cache was pruned.
- A failed readiness check often means initialization is still running, a volume belongs to an incompatible server version, or the app is using the wrong port. Read the container logs before replacing data.
- A service that is healthy but rejects your app connection usually needs the generated password, the correct database or virtual host, or the host name appropriate for where the app runs.
- Features that send mail, fetch remote identity providers, or call remote embedding models require a connection to those providers. Use local substitutes or disable those integrations for an offline session.

[Docker volumes](https://docs.docker.com/engine/storage/volumes/) · [Docker port publishing](https://docs.docker.com/engine/network/port-publishing/)
