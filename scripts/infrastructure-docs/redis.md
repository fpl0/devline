## Connect and test expiry

Use the Redis URL from the service details. The image's local `redis-cli` receives the generated password through `REDISCLI_AUTH`.

```sh
docker exec "$CONTAINER" redis-cli ping
docker exec "$CONTAINER" redis-cli SET devline:probe offline EX 60
docker exec "$CONTAINER" redis-cli GET devline:probe
docker exec "$CONTAINER" redis-cli TTL devline:probe
```

A TTL of `-2` means the key is absent. A TTL of `-1` means the key exists without an expiry. Store cache prefixes and expirations explicitly in your app so local tests behave like the deployed cache.

## Backup and restore

Devline enables the append-only file and mounts the `/data` volume. Inspect persistence before taking a copy.

```sh
docker exec "$CONTAINER" redis-cli INFO persistence
docker exec "$CONTAINER" redis-cli DBSIZE
```

Stop writes and use the [cold volume backup](https://github.com/fpl0/devline/blob/main/scripts/infrastructure-docs/portable-toolbox.md#back-up-a-named-data-volume) for a local clone. Keep the whole data directory, including the append-only manifest and segments. Restore it into a fresh volume with the same Redis version, then verify `PING`, key count, and a known key. Redis prefers the AOF on restart when both AOF and an RDB snapshot are present.

[Redis persistence](https://redis.io/docs/latest/operate/oss_and_stack/management/persistence/) · [Redis CLI](https://redis.io/docs/latest/develop/tools/cli/)
