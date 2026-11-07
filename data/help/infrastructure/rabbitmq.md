# Use RabbitMQ in Devline

**Image** `docker.io/library/rabbitmq:4.3.7-management-alpine` for Linux arm64. **Default host ports** `5672`, `15672`.

## Connect and inspect

Use the AMQP URI from the service details. The application user and virtual host are `devline`. Open the management console on its host port, normally 15672, and use the same credentials. AMQP clients use port 5672 by default.

```sh
docker exec "$CONTAINER" rabbitmq-diagnostics -q check_running
docker exec "$CONTAINER" rabbitmq-diagnostics -q check_port_listener 5672
docker exec "$CONTAINER" rabbitmqctl list_queues -p devline name durable messages
```

Use the [application connections guide](application-connections.md) to publish and consume a test message.

## Back up definitions

Definitions include topology, users, permissions, and policies. Treat the export as sensitive because it can contain password hashes.

```sh
docker exec "$CONTAINER" rabbitmqctl export_definitions /tmp/rabbitmq-definitions.json
docker cp "$CONTAINER:/tmp/rabbitmq-definitions.json" ./rabbitmq-definitions.json
docker exec "$CONTAINER" rm /tmp/rabbitmq-definitions.json
```

On an isolated restore instance, copy the file in and import it. Verify its expected queues, exchanges, and vhosts before connecting applications.

```sh
docker cp ./rabbitmq-definitions.json "$RESTORE_CONTAINER:/tmp/rabbitmq-definitions.json"
docker exec "$RESTORE_CONTAINER" rabbitmqctl import_definitions /tmp/rabbitmq-definitions.json
docker exec "$RESTORE_CONTAINER" rabbitmqctl list_vhosts
```

Definitions do not include messages. Stop the broker and take a cold copy of the entire `/var/lib/rabbitmq` volume for message persistence. Restore it with the same broker image and exactly the same Erlang node name.

[RabbitMQ definitions](https://www.rabbitmq.com/docs/definitions) · [RabbitMQ backup and restore](https://www.rabbitmq.com/docs/backup)
