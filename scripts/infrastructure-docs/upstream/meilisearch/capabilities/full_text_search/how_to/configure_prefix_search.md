# Configure prefix search

> Enable or disable prefix matching to control whether Meilisearch matches partial words as the user types.

Prefix search allows Meilisearch to match documents based on the beginning of the last word in a query. For example, typing `adv` matches "adventure", "adventure", and "advanced". This is the feature that powers the "search as you type" experience.

The `prefixSearch` index setting controls how Meilisearch handles prefix matching.

## Available modes

| Mode | Description |
| - | - |
| `indexingTime` | **Default.** Prefix data structures are built during indexing. This enables fast prefix search at query time but increases index size and indexing duration. |
| `disabled` | Prefix search is turned off. Only exact word matches are returned. This reduces index size and speeds up indexing, but users must type complete words to find results. |

## Check current prefix search setting

Retrieve the current `prefixSearch` setting for an index:

```bash
curl \
  -X GET 'MEILISEARCH_URL/indexes/INDEX_UID/settings/prefix-search' \
  -H 'Authorization: Bearer API_KEY'
```

```javascript
client.index('INDEX_NAME').getPrefixSearch();
```

```python
client.index('books').get_prefix_search()
```

```php
$client->index('INDEX_NAME')->getPrefixSearch();
```

```ruby
client.index('INDEX_UID').prefix_search
```

```go
client.Index("books").GetPrefixSearch()
```

```csharp
await client.Index("books").GetPrefixSearchAsync();
```

```rust
let prefix_search: PrefixSearchSettings = client
  .index(INDEX_UID)
  .get_prefix_search()
  .await
  .unwrap();
```

By default, the response is `"indexingTime"`.

## Disable prefix search

If your use case does not require search-as-you-type (for example, users submit complete queries via a search button), disabling prefix search can reduce index size and improve indexing performance:

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/INDEX_UID/settings/prefix-search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '"disabled"'
```

```javascript
client.index('INDEX_NAME').updatePrefixSearch('disabled');
```

```python
client.index('books').update_prefix_search(PrefixSearch.DISABLED)
```

```php
$client->index('INDEX_NAME')->updatePrefixSearch('disabled');
```

```ruby
client.index('INDEX_UID').update_prefix_search('disabled')
```

```go
client.Index("books").UpdatePrefixSearch("disabled")
```

```csharp
await client.Index("books").UpdatePrefixSearchAsync("disabled");
```

```rust
let task: TaskInfo = client
  .index(INDEX_UID)
  .set_prefix_search(PrefixSearchSettings::Disabled)
  .await
  .unwrap();
```

> [!WARNING]
>
> Updating the prefix search setting triggers a re-indexing of all documents in the index. This is an [asynchronous](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations) operation. Use the [task API](https://www.meilisearch.com/docs/reference/api/tasks/get-all-tasks) to monitor progress.

## Reset prefix search

Restore the default `indexingTime` behavior:

```bash
curl \
  -X DELETE 'MEILISEARCH_URL/indexes/INDEX_UID/settings/prefix-search' \
  -H 'Authorization: Bearer API_KEY'
```

```javascript
client.index('INDEX_NAME').resetPrefixSearch();
```

```python
client.index('books').reset_prefix_search()
```

```php
$client->index('INDEX_NAME')->resetPrefixSearch();
```

```ruby
client.index('INDEX_UID').reset_prefix_search
```

```go
client.Index("books").ResetPrefixSearch()
```

```csharp
await client.Index("books").ResetPrefixSearchAsync();
```

```rust
let task: TaskInfo = client
  .index(INDEX_UID)
  .reset_prefix_search()
  .await
  .unwrap();
```

## When to disable prefix search

* **Form-based search**: Users type a full query and press a search button rather than seeing results as they type
* **Large datasets with performance constraints**: Disabling prefix search reduces index size and speeds up both indexing and queries
* **Exact matching requirements**: When partial word matches would return too many irrelevant results

## When to keep prefix search enabled

* **Search-as-you-type interfaces**: Users expect results to update instantly as they type each character
* **Autocomplete experiences**: Prefix matching is essential for suggesting completions
* **Discovery-oriented search**: Partial matches help users explore content they might not find with exact queries

> [!NOTE]
>
> Prefix search only applies to the **last word** in a multi-word query. Earlier words in the query must match completely (or within [typo tolerance](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/typo_tolerance_settings)). For example, searching for `harry pot` matches "Harry Potter" because "harry" matches exactly and "pot" is a prefix match for "Potter".

> [!NOTE]
>
> For the full API reference, see [get prefix search](https://www.meilisearch.com/docs/reference/api/settings/get-prefixsearch).


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.