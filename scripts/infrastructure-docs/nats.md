## Connect and monitor

Use the authenticated NATS URL from the service details. The client port is normally 4222. The monitoring port is normally 8222 and is intended for loopback inspection.

```sh
curl -fsS 'http://127.0.0.1:8222/healthz?js-enabled-only=true'
curl -fsS 'http://127.0.0.1:8222/varz'
curl -fsS 'http://127.0.0.1:8222/jsz'
```

Core NATS delivers to live subscribers. Use JetStream for durable messages, replay, and consumer acknowledgments. With the optional NATS CLI already installed, use two terminals to smoke-test a subject.

```sh
nats --server "$NATS_URL" sub devline.probe
nats --server "$NATS_URL" pub devline.probe 'offline'
```

## Persist and restore streams

JetStream uses the `/data` volume. The NATS CLI can take a stream snapshot and restore it to a compatible server. List the streams first so you can choose the right stream name.

```sh
nats --server "$NATS_URL" stream ls
nats --server "$NATS_URL" stream backup ORDERS ./orders-backup
nats --server "$RESTORE_NATS_URL" stream restore ./orders-backup
```

Use a separate restore instance for testing. If the optional CLI is unavailable, stop the single-node server and copy its entire named volume using the [cold volume backup](https://github.com/fpl0/devline/blob/main/scripts/infrastructure-docs/portable-toolbox.md). Restore using the same server version and check stream state and consumer behavior before moving application traffic.

[NATS monitoring endpoints](https://docs.nats.io/learn/monitoring/monitoring-endpoints) · [NATS CLI](https://github.com/nats-io/natscli)
