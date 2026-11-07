# Exporting and using Snapshots

> Snapshots are exact copies of Meilisearch databases. They are often useful for periodical backups.

A [snapshot](https://www.meilisearch.com/docs/resources/self_hosting/data_backup/overview#when-to-use-snapshots) is an exact copy of the Meilisearch database. Snapshots are useful as quick backups, but cannot be used to migrate to a new Meilisearch release.

This tutorial shows you how to schedule snapshot creation to ensure you always have a recent backup of your instance ready to use. You will also see how to start Meilisearch from this snapshot.

> [!WARNING]
>
> Meilisearch Cloud does not support snapshots.

## Scheduling periodic snapshots

It is good practice to create regular backups of your Meilisearch data. This ensures that you can recover from critical failures quickly in case your Meilisearch instance becomes compromised.

Use the [`--schedule-snapshot` configuration option](https://www.meilisearch.com/docs/resources/self_hosting/configuration/reference#schedule-snapshot-creation) to create snapshots at regular time intervals:

```bash
meilisearch --schedule-snapshot
```

After launch, Meilisearch waits for the configured interval before creating the first snapshot. You will find it in the [snapshot directory](https://www.meilisearch.com/docs/resources/self_hosting/configuration/reference#snapshot-destination), `snapshots/`. Meilisearch will then create a new snapshot every 24 hours until you terminate your instance.

Meilisearch **automatically overwrites** old snapshots during snapshot creation. Only the most recent snapshot will be present in the folder at any given time.

In cases where your database is updated several times a day, it might be better to modify the interval between each new snapshot:

```bash
meilisearch --schedule-snapshot=3600
```

This instructs Meilisearch to create a new snapshot once every hour.

> [!TIP]
>
> If you need to generate a single snapshot without relaunching your instance, use [the `/snapshots` route](https://www.meilisearch.com/docs/reference/api/management/create-snapshot).

## Starting from a snapshot

To import snapshot data into your instance, launch Meilisearch using `--import-snapshot`:

```bash
meilisearch --import-snapshot mySnapShots/data.ms.snapshot
```

Because snapshots are exact copies of your database, starting a Meilisearch instance from a snapshot is much faster than adding documents manually or starting from a dump.

> [!WARNING]
>
> For security reasons, Meilisearch will never overwrite an existing database. By default, Meilisearch will throw an error when importing a snapshot if there is any data in your instance.
>
> You can change this behavior by specifying [`--ignore-snapshot-if-db-exists=true`](https://www.meilisearch.com/docs/resources/self_hosting/configuration/reference#ignore-snapshot-if-db-exists). This will cause Meilisearch to launch with the existing database and ignore the snapshot without throwing an error.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.