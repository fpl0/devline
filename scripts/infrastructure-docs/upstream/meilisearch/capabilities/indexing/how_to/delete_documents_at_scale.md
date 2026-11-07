# Delete documents at scale

> Remove large numbers of documents efficiently using batch deletion, filter-based deletion, and lifecycle management strategies.

When you need to remove thousands or millions of documents from an index, deleting them one at a time is impractical. Meilisearch provides batch deletion and filter-based deletion for removing documents efficiently.

## Delete by filter

Filter-based deletion removes all documents matching a [filter expression](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/advanced/filter_expression_syntax). This is the most efficient way to delete large sets of documents when they share a common attribute.

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/products/documents/delete' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "filter": "status = '\''archived'\''"
  }'
```

The filter expression supports the same syntax as [search filters](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/advanced/filter_expression_syntax), including `AND`, `OR`, and comparison operators.

> [!NOTE]
>
> Delete-by-filter tasks cannot be autobatched with other task types. Each delete-by-filter operation is processed as its own individual batch. If you are enqueuing many delete-by-filter tasks alongside other write operations, be aware that this may slow down overall task processing.

> [!WARNING]
>
> The attribute used in the filter must be listed in [`filterableAttributes`](https://www.meilisearch.com/docs/reference/api/settings/get-filterableattributes). If it is not, the request returns an error.

### Common filter patterns

**Delete by category:**

```json
{ "filter": "category = 'discontinued'" }
```

**Delete by date range:**

```json
{ "filter": "expires_at < 1704067200" }
```

**Delete with compound conditions:**

```json
{ "filter": "status = 'draft' AND updated_at < 1672531200" }
```

## Delete by batch of IDs

When you know the exact document IDs to remove, send them as an array:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/products/documents/delete-batch' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '["id1", "id2", "id3", "id4", "id5"]'
```

For very large ID lists, split them into batches. Each request creates a [task](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations), and tasks are processed sequentially:

```bash
# Split IDs into chunks and send each as a separate request
# Each batch processes as its own task
for batch_file in id_batch_*.json; do
  curl \
    -X POST 'MEILISEARCH_URL/indexes/products/documents/delete-batch' \
    -H 'Content-Type: application/json' \
    -H 'Authorization: Bearer MEILISEARCH_KEY' \
    --data-binary @"$batch_file"
done
```

## Monitor deletion progress

Deletion operations are asynchronous. The response returns a `taskUid` you can use to track progress:

```json
{
  "taskUid": 128,
  "indexUid": "products",
  "status": "enqueued",
  "type": "documentDeletion"
}
```

Check the task to see how many documents were deleted:

```bash
curl \
  -X GET 'MEILISEARCH_URL/tasks/128' \
  -H 'Authorization: Bearer MEILISEARCH_KEY'
```

The completed task includes the count of deleted documents:

```json
{
  "uid": 128,
  "status": "succeeded",
  "type": "documentDeletion",
  "details": {
    "providedIds": 0,
    "deletedDocuments": 15234,
    "originalFilter": "status = 'archived'"
  }
}
```

## Choose the right deletion strategy

| Strategy | Best for | Example |
| - | - | - |
| Delete by filter | Removing documents that share an attribute | Remove all expired listings, delete a product category |
| Delete by batch | Removing specific documents by ID | Remove items flagged by a moderation system |
| Delete all documents | Clearing an index for a full re-import | Nightly sync from a primary database |

### Delete all documents

To remove every document in an index while keeping the index settings:

```bash
curl \
  -X DELETE 'MEILISEARCH_URL/indexes/products/documents' \
  -H 'Authorization: Bearer MEILISEARCH_KEY'
```

This is useful when your data pipeline does full replacements. Delete all documents, then re-import the current dataset.

## Plan for regular cleanup

If your data has a natural lifecycle (listings expire, events pass, articles are archived), consider adding a timestamp or status field to your documents and making it filterable:

```json
{
  "id": "listing-42",
  "title": "Summer Sale",
  "status": "active",
  "expires_at": 1719792000
}
```

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/listings/settings' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "filterableAttributes": ["status", "expires_at"]
  }'
```

Then run periodic cleanup jobs:

```bash
# Remove expired listings
curl \
  -X POST 'MEILISEARCH_URL/indexes/listings/documents/delete' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary "{
    \"filter\": \"expires_at < $(date +%s)\"
  }"
```

## Next steps

> [card] [Add and update documents](https://www.meilisearch.com/docs/capabilities/indexing/how_to/add_and_update_documents)
>
> Learn about document add, update, and replace operations

> [card] [Filter expression reference](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/advanced/filter_expression_syntax)
>
> Full syntax reference for filter expressions

> [card] [Async operations](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations)
>
> Understand how tasks work in Meilisearch


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.