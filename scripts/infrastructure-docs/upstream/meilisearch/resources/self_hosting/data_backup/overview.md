# Backing up Meilisearch data

> Meilisearch offers two backup methods: snapshots for periodic safeguards and dumps for version migration. Learn when to use each.

Meilisearch offers two backup methods: **snapshots** and **dumps**. They serve different purposes and have different trade-offs.

## Snapshots vs dumps

| | Snapshots | Dumps |
| - | - | - |
| **What it is** | Exact copy of the database (`data.ms`) | Portable blueprint of all instance data |
| **Import speed** | Fast (data is already indexed) | Slow (requires full re-indexing) |
| **Version compatibility** | Same Meilisearch version only | Compatible across versions |
| **File size** | Larger | Smaller |
| **Best for** | Periodic backups, disaster recovery | Migrating to a new Meilisearch version |
| **Scheduling** | Can be [scheduled at launch](https://www.meilisearch.com/docs/resources/self_hosting/configuration/reference#schedule-snapshot-creation) | Created on demand via API |
| **Cloud support** | Self-hosted only | Cloud (via UI) and self-hosted (via API) |

## When to use snapshots

Use snapshots as a safeguard. If something goes wrong, you can recover and relaunch your database quickly. You can schedule periodic snapshot creation at launch.

[Learn how to create and import snapshots](https://www.meilisearch.com/docs/resources/self_hosting/data_backup/snapshots).

## When to use dumps

Use dumps when migrating data between Meilisearch versions. Dumps are not bound to a specific version, so they are ideal for upgrades.

> [!NOTE]
>
> You can import dumps from older Meilisearch versions into newer ones. Importing a dump from a newer version into an older one can lead to unexpected behavior.

[Learn how to create and import dumps](https://www.meilisearch.com/docs/resources/self_hosting/data_backup/dumps).

## Backup recommendations

* **Schedule snapshots** for regular backups. A daily snapshot (`--schedule-snapshot=86400`) is a good starting point.
* **Create a dump before upgrading** Meilisearch to a new version.
* **Test your restore process** periodically to make sure backups work.
* **Store backups off-server** using [S3 snapshot storage](https://www.meilisearch.com/docs/resources/self_hosting/configuration/reference#s3-options-enterprise) or by copying dump files to external storage.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.