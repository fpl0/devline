# Document relations

> Automatically enrich search results with related data from other indexes using foreign keys.

Foreign keys let you link documents across indexes so that search results are automatically enriched with related data. Instead of duplicating information, you store it once in a dedicated index and reference it by ID.

For example, a `movies` index can reference actors by ID. When you search for movies, Meilisearch automatically replaces the actor IDs with full actor documents from the `actors` index. This approach also works with [multi-search](https://www.meilisearch.com/docs/capabilities/multi_search/overview) when querying across related indexes.

> [!WARNING]
>
> Foreign keys is an experimental feature. Its API and behavior may change in future releases. It is not supported in remote sharding environments.

## Step 1: Enable the experimental feature

Foreign keys must be activated through the [experimental features endpoint](https://www.meilisearch.com/docs/reference/api/experimental-features/list-experimental-features) before you can use them:

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/experimental-features' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "foreignKeys": true
  }'
```

## Step 2: Create your related index

Add documents to the index you want to reference. In this example, create an `actors` index with actor data:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/actors/documents' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    { "id": 1, "name": "Tom Hanks", "born": 1956 },
    { "id": 2, "name": "Robin Wright", "born": 1966 },
    { "id": 3, "name": "Gary Sinise", "born": 1955 }
  ]'
```

## Step 3: Configure foreign keys in the main index

Use the settings endpoint to define which fields contain foreign references and which index they point to:

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/movies/settings' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "foreignKeys": [
      {
        "fieldName": "actors",
        "foreignIndexUid": "actors"
      }
    ]
  }'
```

This tells Meilisearch that the `actors` field in the `movies` index contains IDs that reference documents in the `actors` index.

## Step 4: Add documents with foreign IDs

Add documents to your main index. Use arrays of IDs for the foreign key field:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/documents' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    { "id": 1, "title": "Forrest Gump", "actors": [1, 2, 3] },
    { "id": 2, "title": "Cast Away", "actors": [1] }
  ]'
```

## Step 5: Search and see hydrated results

When you search the `movies` index, Meilisearch automatically replaces foreign IDs with full documents from the referenced index:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/search' \
  -H 'Content-Type: application/json' \
  --data-binary '{ "q": "forrest" }'
```

Without foreign keys, a result would look like this:

```json
{
  "id": 1,
  "title": "Forrest Gump",
  "actors": [1, 2, 3]
}
```

With foreign keys configured, the same result is automatically hydrated:

```json
{
  "id": 1,
  "title": "Forrest Gump",
  "actors": [
    { "id": 1, "name": "Tom Hanks", "born": 1956 },
    { "id": 2, "name": "Robin Wright", "born": 1966 },
    { "id": 3, "name": "Gary Sinise", "born": 1955 }
  ]
}
```

## Limitations

* **Experimental**: This feature may change or be removed in future versions.
* **No remote [sharding](https://www.meilisearch.com/docs/resources/self_hosting/sharding/overview)**: Foreign keys are not supported in environments using remote sharding.
* **One direction**: Hydration works from the main index to the referenced index. The referenced index does not automatically link back.

## Next steps

> [card] [Foreign keys settings API](https://www.meilisearch.com/docs/reference/api/settings/get-foreignkeys)
>
> Full API reference for foreign key settings

> [card] [Experimental features API](https://www.meilisearch.com/docs/reference/api/experimental-features/list-experimental-features)
>
> Enable and manage experimental features

> [card] [Indexing overview](https://www.meilisearch.com/docs/capabilities/indexing/overview)
>
> Learn more about how indexing works in Meilisearch


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.