## Keep endpoint settings in the app environment

Use the service details to copy the real mapped ports and credentials. These keys are examples your application can read from its environment. Store secrets in a local env file excluded from source control.

For a Devline-managed project, `devline infra credentials ./my-app` shows the resolved connection strings. The command intentionally reveals local credentials.

```dotenv
DATABASE_URL=postgresql://devline:<password>@127.0.0.1:5432/devline
AMQP_URL=amqp://devline:<password>@127.0.0.1:5672/devline
REDIS_URL=redis://:<password>@127.0.0.1:6379/0
SMTP_HOST=127.0.0.1
SMTP_PORT=1025
S3_ENDPOINT=http://127.0.0.1:8333
S3_BUCKET=devline
OIDC_ISSUER=http://127.0.0.1:8180/realms/devline
```

The variable names are application conventions. Match them to your framework or SDK. URL-encode characters in user info if you replace generated credentials with a password containing reserved characters. In a Compose application, use service names and internal ports. Match your configured OIDC issuer to the exact discovery URL used by your browser and application.

## Make the database do real work

Set `PG_CONTAINER` to the running PostgreSQL container and execute a transaction using the client already installed in its image.

```sh
docker exec -i "$PG_CONTAINER" psql -X -v ON_ERROR_STOP=1 -U devline -d devline <<'SQL'
BEGIN;
CREATE TABLE IF NOT EXISTS devline_probe (id integer PRIMARY KEY, note text NOT NULL);
INSERT INTO devline_probe VALUES (1, 'offline')
ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note;
SELECT id, note FROM devline_probe WHERE id = 1;
ROLLBACK;
SQL
```

Run the project's own migrations and integration tests next. They should use the same database URL the application receives.

## Exercise a durable queue

Set `RABBITMQ_PASSWORD` to the generated broker credential and replace `15672` if you changed the management port. The management API can create, publish to, and read a disposable queue without an extra CLI or client package.

```sh
base=http://127.0.0.1:15672/api
curl -fsS -u "devline:$RABBITMQ_PASSWORD" -X PUT \
  -H 'Content-Type: application/json' -d '{"durable":true}' \
  "$base/queues/devline/devline-probe"
curl -fsS -u "devline:$RABBITMQ_PASSWORD" -X POST \
  -H 'Content-Type: application/json' \
  -d '{"properties":{"delivery_mode":2},"routing_key":"devline-probe","payload":"offline","payload_encoding":"string"}' \
  "$base/exchanges/devline/amq.default/publish"
curl -fsS -u "devline:$RABBITMQ_PASSWORD" -X POST \
  -H 'Content-Type: application/json' \
  -d '{"count":1,"ackmode":"ack_requeue_false","encoding":"auto","truncate":1000}' \
  "$base/queues/devline/devline-probe/get"
```

Expect the publish response to contain `"routed":true` and the read response to contain the payload. Configure your app's AMQP library against the AMQP port and the same virtual host. The HTTP management endpoint is a useful probe. Application consumers should use AMQP acknowledgments and publishers should use confirms when delivery matters.

## Plan for transport and startup

- Keep initialization, migration, and application startup as separate observable steps.
- A listening TCP port is not sufficient for an authenticated query. Use the service's readiness check before launching tests.
- A host port and container port can differ. For remote development or non-Docker-Desktop engines, test the actual route before going offline.
- Keep local credentials and example messages disposable. Do not copy data or access tokens into the repository.

[RabbitMQ HTTP API](https://www.rabbitmq.com/docs/http-api-reference) · [PostgreSQL SQL commands](https://www.postgresql.org/docs/18/sql-commands.html) · [Docker Desktop networking](https://docs.docker.com/desktop/features/networking/)
