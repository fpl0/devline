# Use MySQL in Devline

**Image** `docker.io/library/mysql:8.4.11` for Linux arm64. **Default host ports** `3306`.

## Connect and query

Use the initial `devline` database and application user. The service has a separate root password for local administration. A cold first boot may take longer while MySQL initializes its data directory and creates the application grants.

```sh
docker exec "$CONTAINER" sh -c 'exec mysql --host=127.0.0.1 --user=devline --password="$MYSQL_PASSWORD" --database=devline --execute="SELECT DATABASE(), CURRENT_USER(), VERSION()"'
```

Use an up-to-date MySQL 8.4 client or connector for the default authentication method. Change credentials through SQL for an existing volume. The initialization variables apply to a fresh data directory.

## Dump and restore

For a consistent InnoDB snapshot, avoid schema changes during the dump. Use the root account inside the container and direct the dump to the Mac.

```sh
docker exec "$CONTAINER" sh -c 'exec mysqldump --user=root --password="$MYSQL_ROOT_PASSWORD" --single-transaction --routines --events --no-tablespaces devline' > devline.mysql.sql
docker exec "$CONTAINER" sh -c 'exec mysql --user=root --password="$MYSQL_ROOT_PASSWORD" --execute="CREATE DATABASE devline_restore"'
docker exec -i "$CONTAINER" sh -c 'exec mysql --user=root --password="$MYSQL_ROOT_PASSWORD" devline_restore' < devline.mysql.sql
docker exec "$CONTAINER" sh -c 'exec mysql --user=root --password="$MYSQL_ROOT_PASSWORD" devline_restore --execute="SHOW TABLES"'
```

The dump contains application data. Keep it private and verify the restored schemas and rows before switching the project. Preserve the `/var/lib/mysql` volume when recreating the same service.

[MySQL 8.4 Reference Manual](https://dev.mysql.com/doc/refman/8.4/en/) · [mysqldump](https://dev.mysql.com/doc/refman/8.4/en/mysqldump.html)
