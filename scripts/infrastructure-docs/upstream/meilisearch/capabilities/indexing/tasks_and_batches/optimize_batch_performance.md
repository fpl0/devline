# Optimize indexing performance with batch statistics

> Learn how to analyze the `progressTrace` to identify and resolve indexing bottlenecks in Meilisearch.

Indexing performance can vary significantly depending on your dataset, index settings, and hardware. The [batch object](https://www.meilisearch.com/docs/reference/api/batches/list-batches) provides information about the progress of asynchronous indexing operations.

The `progressTrace` field within the batch object offers a detailed breakdown of where time is spent during the indexing process. Use this data to identify bottlenecks and improve indexing speed.

## Understanding the `progressTrace`

`progressTrace` is a hierarchical trace showing each phase of indexing and how long it took.
Each entry follows the structure:

```json
"processing tasks > indexing > extracting word proximity": "33.71s"
```

This means:

* The step occurred during **indexing**.
* The subtask was **extracting word proximity**.
* It took **33.71 seconds**.

Focus on the **longest-running steps** and investigate which index settings or data characteristics influence them.

## Key phases and how to optimize them

### `computing document changes`and `extracting documents`

| Description | Optimization |
| - | - |
| Meilisearch compares incoming documents to existing ones. | No direct optimization possible. Process duration scales with the number and size of incoming documents. |

### `extracting facets` and `merging facet caches`

| Description | Optimization |
| - | - |
| Extracts and merges filterable attributes. | Keep the number of [**filterable attributes**](https://www.meilisearch.com/docs/reference/api/settings/get-filterableattributes) to a minimum. |

### `extracting words` and `merging word caches`

| Description | Optimization |
| - | - |
| Tokenizes text and builds the inverted index. | Ensure the [searchable attributes](https://www.meilisearch.com/docs/reference/api/settings/get-searchableattributes) list only includes the fields you want to be checked for query word matches. |

### `extracting word proximity` and `merging word proximity`

| Description | Optimization |
| - | - |
| Builds data structures for phrase and attribute ranking. | Lower the precision of this operation by setting [proximity precision](https://www.meilisearch.com/docs/reference/api/settings/update-proximityprecision) to `byAttribute` |

### `waiting for database writes`

| Description | Optimization |
| - | - |
| Time spent writing data to disk. | No direct optimization possible. Either the disk is too slow or you are writing too much data in a single operation. Avoid HDDs (Hard Disk Drives) |

### `waiting for extractors`

| Description | Optimization |
| - | - |
| Time spent waiting for CPU-bound extraction. | No direct optimization possible. Indicates a CPU bottleneck. Use more cores or scale horizontally with [sharding](https://www.meilisearch.com/docs/resources/self_hosting/deployment/overview). |

### `post processing facets > strings bulk` / `numbers bulk`

| Description | Optimization |
| - | - |
| Processes equality or comparison filters. | - Disable unused [**filter features**](https://www.meilisearch.com/docs/reference/api/settings/get-filterableattributes), such as comparison operators on string values.  - Reduce the number of [**sortable attributes**](https://www.meilisearch.com/docs/reference/api/settings/get-sortableattributes). |

### `post processing facets > facet search`

| Description | Optimization |
| - | - |
| Builds structures for the [facet search API](https://www.meilisearch.com/docs/reference/api/facet-search/search-for-facet-values). | If you don’t use the facet search API, [disable it](https://www.meilisearch.com/docs/reference/api/settings/update-facetsearch). |

### Embeddings

| Trace key | Description | Optimization |
| - | - | - |
| `writing embeddings to database` | Time spent saving vector embeddings. | Use embedding vectors with fewer dimensions. - Consider enabling [binary quantization](https://www.meilisearch.com/docs/reference/api/settings/update-embedders). |
| `extracting embeddings` | Time spent extracting embeddings from embedding providers' responses. | Reduce the amount of data sent to embeddings provider. - [Include fewer attributes in `documentTemplate`](https://www.meilisearch.com/docs/capabilities/hybrid_search/advanced/document_template_best_practices). - [Reduce maximum size of the document template](https://www.meilisearch.com/docs/reference/api/settings/update-embedders). - [Disabling embedding regeneration on document update](https://www.meilisearch.com/docs/reference/api/documents/add-or-update-documents). - If using a third-party service like OpenAI, upgrade your account to a higher tier. |

### `post processing words > word prefix *`

| Description | Optimization | |
| - | - | - |
| | Builds prefix data for autocomplete. Allows matching documents that begin with a specific query term, instead of only exact matches. | Disable [**prefix search**](https://www.meilisearch.com/docs/reference/api/settings/get-prefixsearch) (`prefixSearch: disabled`). *This can severely impact search result relevancy.* |

### `post processing words > word fst`

| Description | Optimization |
| - | - |
| Builds the word FST (finite state transducer). | No direct action possible, as FST size reflect the number of different words in the database. Using documents with fewer searchable words may improve operation speed. |

## Example analysis

If you see:

```json
"processing tasks > indexing > post processing facets > facet search": "1763.06s"
```

[Facet searching](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/filter_with_facets#searching-facet-values) is taking significant indexing time. If your application doesn’t use facets, disable the feature:

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/INDEX_UID/settings/facet-search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary 'false'
```

```javascript
client.index('INDEX_NAME').updateFacetSearch(false);
```

```python
client.index('books').update_facet_search_settings(False)
```

```php
$client->index('INDEX_NAME')->updateFacetSearch(false);
```

```ruby
client.index('INDEX_UID').update_facet_search_setting(false)
```

```go
client.Index("books").UpdateFacetSearch(false)
```

```csharp
await client.Index("books").UpdateFacetSearchAsync(false);
```

```rust
let task: TaskInfo = client
  .index(INDEX_UID)
  .set_facet_search(false)
  .await
  .unwrap();
```

## Learn more

* [Indexing best practices](https://www.meilisearch.com/docs/capabilities/indexing/advanced/indexing_best_practices)
* [Impact of RAM and multi-threading on indexing performance](https://www.meilisearch.com/docs/resources/self_hosting/performance/ram_multithreading)
* [Configuring index settings](https://www.meilisearch.com/docs/capabilities/indexing/overview)


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.