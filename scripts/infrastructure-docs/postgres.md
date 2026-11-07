## Connect and query

Use the PostgreSQL URI from Devline's service details. The initial role and database are both `devline`. Set `CONTAINER` to the running container name. The image provides `psql`, `pg_isready`, `pg_dump`, and `pg_restore`.

```sh
docker exec "$CONTAINER" pg_isready -h 127.0.0.1 -U devline -d devline
docker exec "$CONTAINER" psql -X -v ON_ERROR_STOP=1 -U devline -d devline \
  -c 'SELECT current_database(), current_user, version();'
```

If a host application cannot log in, compare its password and port with the service details. The direct container commands above use the local Unix socket for SQL. Check the application's authenticated TCP connection separately.

## Persistence

The PostgreSQL 18 image stores its data under `/var/lib/postgresql/18/docker`. Devline mounts the named volume at `/var/lib/postgresql`. The bootstrap database, user, and password settings apply when the data directory is initialized. Changing the container environment alone does not update roles in an existing database.

## Dump and restore

Create a custom-format dump while the service is running. Restore into a separate, empty database so you can inspect it before switching clients.

```sh
docker exec "$CONTAINER" pg_dump -Fc -U devline -d devline > devline.pgdump
docker exec "$CONTAINER" createdb -U devline -T template0 devline_restore
docker exec -i "$CONTAINER" pg_restore --exit-on-error --no-owner \
  -U devline -d devline_restore < devline.pgdump
docker exec "$CONTAINER" psql -X -U devline -d devline_restore -c '\dt'
```

A single-database dump excludes cluster roles and tablespaces. Use `pg_dumpall --globals-only` when those objects matter. Test restoration with the same or a compatible newer server version before replacing a project database.

[Offline SQL dump reference](https://www.postgresql.org/docs/18/backup-dump.html) · [pg_dump](https://www.postgresql.org/docs/18/app-pgdump.html) · [pg_restore](https://www.postgresql.org/docs/18/app-pgrestore.html)
