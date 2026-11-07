# Indexing

> Add, update, and manage documents in Meilisearch indexes, including task monitoring and batch operations.

Indexing is the process of adding documents to Meilisearch so they become searchable. All indexing operations are [asynchronous](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations), meaning they are added to a task queue and processed in order.

## Key concepts

| Concept | Description |
| - | - |
| Documents | JSON objects with a [primary key](https://www.meilisearch.com/docs/resources/internals/primary_key) that become searchable records |
| Primary key | A unique identifier for each document in an index |
| Tasks | Asynchronous operations that track the status of indexing requests |
| Batches | Groups of tasks processed together for efficiency |

## How indexing works

When you send documents to Meilisearch, the engine follows an asynchronous pipeline:

1. **Request**: Your application sends documents to the Meilisearch API.
2. **Task queue**: Meilisearch creates a task and places it in a FIFO queue. The API immediately returns a task ID so you can track progress.
3. **Processing**: The engine processes the task, parsing documents, building the inverted index and other internal data structures.
4. **Searchable**: Once processing completes, the documents are immediately available for search queries.

You can monitor the status of any task (enqueued, processing, succeeded, or failed) through the tasks API.

## Document formats

Meilisearch accepts documents in three formats:

* **JSON**: Arrays of objects. The most common format for API integrations.
* **NDJSON** (Newline-Delimited JSON): One JSON object per line. Ideal for streaming large datasets without loading everything into memory.
* **CSV**: Comma-separated values with a header row. Useful for importing data from spreadsheets or database exports.

All formats require that each document contains a primary key field to uniquely identify it within the index. Once indexed, documents are available for [full-text search](https://www.meilisearch.com/docs/capabilities/full_text_search/overview), [filtering](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/getting_started), and other search operations.

## Primary key

Every document in a Meilisearch index must have a unique **primary key** field. If you do not specify a primary key when creating an index, Meilisearch attempts to auto-detect it by looking for an attribute ending in `id` (such as `id`, `movieId`, or `product_id`). You can also set the primary key explicitly when adding documents or through the index settings.

## Cross-index relationships (experimental)

Foreign keys allow you to link documents across indexes. Instead of duplicating data, you store related information in a separate index and reference it by ID. At search time, Meilisearch automatically hydrates results with the full referenced documents. See [Link indexes](https://www.meilisearch.com/docs/capabilities/indexing/how_to/document_relations) for a step-by-step guide.

## Operational tools

Meilisearch includes several endpoints for managing indexes and migrating data:

* **Export**: Transfer data directly from one instance to another over the network, without creating intermediate files. See [Export data to another instance](https://www.meilisearch.com/docs/capabilities/indexing/how_to/export_data).
* **Compact**: Reclaim disk space by reorganizing an index's internal data structures after bulk updates or deletions. See [Compact an index](https://www.meilisearch.com/docs/capabilities/indexing/how_to/compact_an_index).

## Settings updates are atomic

When you update multiple settings in a single request, Meilisearch processes them as an atomic operation.

> [!WARNING]
>
> If Meilisearch encounters an error when updating any of the settings in a request, it immediately stops processing the request and returns an error message. In this case, the database settings remain unchanged. The returned error message will only address the first error encountered.

This means a partial update can never leave your index in an inconsistent state, but it also means you cannot rely on later settings in the request being applied if an earlier one fails. Inspect the task's error details to identify the failing setting, fix it, then resubmit the full settings payload.

## Next steps

> [card] [Getting started](https://www.meilisearch.com/docs/capabilities/indexing/getting_started)
>
> Add your first documents and verify they are indexed

> [card] [Monitor tasks](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/monitor_tasks)
>
> Track the status of indexing operations

> [card] [Best practices](https://www.meilisearch.com/docs/capabilities/indexing/advanced/indexing_best_practices)
>
> Optimize your indexing workflow for production

> [card] [Async operations](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations)
>
> Deep dive into the task lifecycle and queue


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.