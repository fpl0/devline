## Index and search a document

Use the HTTP endpoint and generated master key from the service details. Use the master key only in local administrative scripts. Give an app the narrower search key or scoped credentials it needs.

```sh
base=http://127.0.0.1:7700
curl -fsS "$base/health"
curl -fsS -X POST "$base/indexes/devline/documents?primaryKey=id" \
  -H "Authorization: Bearer $MEILI_MASTER_KEY" -H 'Content-Type: application/json' \
  -d '[{"id":1,"title":"Offline search","tags":["local","reference"]}]'
```

The indexing response includes a `taskUid`. Read `/tasks/<taskUid>` using the same credential until it reports `succeeded`, then search.

```sh
curl -fsS "$base/indexes/devline/search?q=offline" \
  -H "Authorization: Bearer $MEILI_MASTER_KEY"
```

Keyword search and local filtering run offline. An embedding setup that calls a remote provider needs that provider to be reachable.

## Snapshot and restore

Request a snapshot and wait for the returned task to succeed. Copy the resulting file from the service's `snapshots` directory inside `/meili_data`.

```sh
curl -fsS -X POST "$base/snapshots" -H "Authorization: Bearer $MEILI_MASTER_KEY"
docker exec "$CONTAINER" ls -l /meili_data/snapshots
```

Use a fresh service directory and the same version to start Meilisearch with `--import-snapshot /path/to/backup.snapshot`. For a version migration, use a Meilisearch dump and follow the destination version's import procedure. A cold copy of `/meili_data` is another way to move a local instance at the same version.

[Meilisearch API reference](https://www.meilisearch.com/docs/reference/api/overview) · [Meilisearch Docker](https://www.meilisearch.com/integrations/docker)
