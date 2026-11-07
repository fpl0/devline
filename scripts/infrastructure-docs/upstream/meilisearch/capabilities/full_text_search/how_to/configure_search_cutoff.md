# Configure search cutoff

> Set a maximum search time to ensure consistent response times for large datasets.

The search cutoff defines the maximum time in milliseconds that Meilisearch spends processing a single search query. When the cutoff is reached, Meilisearch stops searching and returns the best results found so far. This ensures predictable response times on large datasets where some queries might otherwise take too long.

## How it works

When a search query is processed, Meilisearch iterates through documents and [ranking rules](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/ranking_rules) to find and rank the best matches. On very large datasets (millions of documents) or with broad queries, this process can take significant time.

The search cutoff sets an upper bound on this processing time. If Meilisearch reaches the cutoff before finishing, it returns the results collected up to that point. These results are still ranked correctly according to the ranking rules, but the result set may not include every possible match.

By default, `searchCutoffMs` is `null`. When no explicit value is configured, Meilisearch interrupts searches after **1500 milliseconds**. Setting `searchCutoffMs` to an integer overrides this internal default with your chosen limit.

## Check current search cutoff

Retrieve the current `searchCutoffMs` setting for an index:

```bash
curl \
  -X GET 'MEILISEARCH_URL/indexes/movies/settings/search-cutoff-ms' \
  -H 'Authorization: Bearer API_KEY'
```

```javascript
client.index('movies').getSearchCutoffMs()
```

```python
client.index('movies').get_search_cutoff_ms()
```

```php
$client->index('movies')->getSearchCutoffMs();
```

```java
client.index("movies").getSearchCutoffMsSettings();
```

```ruby
client.index('movies').search_cutoff_ms
```

```go
client.Index("movies").GetSearchCutoffMs()
```

```csharp
var searchCutoff = await client.Index("movies").GetSearchCutoffMsAsync();
```

```rust
let search_cutoff_ms: String = client
  .index("movies")
  .get_search_cutoff_ms()
  .await
  .unwrap();
```

```swift
let precisionValue = try await self.client.index("books").getSearchCutoffMs()
```

The default response is `null`.

## Set a search cutoff

Configure a maximum search time of 150 milliseconds:

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/movies/settings/search-cutoff-ms' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '150'
```

```javascript
client.index('movies').updateSearchCutoffMs(150)
```

```python
client.index('movies').update_search_cutoff_ms(150)
```

```php
$client->index('movies')->updateSearchCutoffMs(150);
```

```java
client.index("movies").updateSearchCutoffMsSettings(150);
```

```ruby
client.index('movies').update_search_cutoff_ms(150)
```

```go
client.Index("movies").UpdateSearchCutoffMs(150)
```

```csharp
await client.Index("movies").UpdateSearchCutoffMsAsync(150);
```

```rust
let task: TaskInfo = client
  .index("movies")
  .set_search_cutoff_ms(Some(150))
  .await
  .unwrap();
```

```swift
let task = try await self.client.index("books").updateSearchCutoffMs(150)
```

With this configuration, any search query that takes longer than 150ms will be interrupted, and Meilisearch returns the best results found within that time.

> [!WARNING]
>
> Setting the cutoff too low may result in incomplete or empty result sets for broad queries. Start with a value between 100ms and 500ms and adjust based on your performance requirements.

## Reset search cutoff

Remove the explicit search cutoff and return to the default behavior (`searchCutoffMs: null`, with Meilisearch's internal 1500 ms interruption threshold):

```bash
curl \
  -X DELETE 'MEILISEARCH_URL/indexes/movies/settings/search-cutoff-ms' \
  -H 'Authorization: Bearer API_KEY'
```

```javascript
client.index('movies').resetSearchCutoffMs()
```

```python
client.index('movies').reset_search_cutoff_ms()
```

```php
$client->index('movies')->resetSearchCutoffMs();
```

```java
client.index("movies").resetSearchCutoffMsSettings();
```

```ruby
client.index('movies').reset_search_cutoff_ms
```

```go
client.Index("books").ResetSearchCutoffMs()
```

```csharp
await client.Index("movies").ResetSearchCutoffMsAsync();
```

```rust
let task: TaskInfo = client
  .index("movies")
  .reset_search_cutoff_ms()
  .await
  .unwrap();
```

```swift
let task = try await self.client.index("books").resetSearchCutoffMs()
```

## Choosing a cutoff value

The right cutoff value is a trade-off: lower values guarantee faster responses but increase the chance of returning incomplete results for broad queries. Higher values give Meilisearch more time to find all matches but allow occasional slow queries.

As a general recommendation, avoid setting the cutoff below **500ms**. This provides a good safety net against unusually long queries (including potential abuse from crafted search strings) while still giving Meilisearch enough time to return quality results for the vast majority of queries.

> [!TIP]
>
> The cutoff is most useful as a safety net, not as a performance tuning knob. If your searches are consistently slow, address the root cause with the optimizations below rather than lowering the cutoff.

## Search cutoff and remote embedders

If your index uses an embedder that calls an external API at search time, the cutoff also bounds that call. Meilisearch derives the embedding request timeout from `searchCutoffMs` rather than applying a fixed timeout of its own, so the cutoff has to cover the provider round trip on top of the search itself.

This matters when choosing a value:

* A cutoff that is comfortable for keyword search can be too short once a provider round trip is included. A 150 ms cutoff is fine for a purely lexical query and will usually not survive a call to a hosted embedding API.
* If the embedder does not answer in time, the query loses its semantic results. For a [pure semantic search](https://www.meilisearch.com/docs/capabilities/hybrid_search/advanced/semantic_vs_hybrid), where there are no keyword results to fall back on, the request can fail with an HTTP 500 instead.

So if semantic results go missing, or pure semantic searches start returning HTTP 500, raise `searchCutoffMs` before looking elsewhere. Embedders that run locally are not affected, since no network call is involved.

> [!TIP]
>
> Measure your provider's typical latency first, then set the cutoff above that plus your normal search time. [Choosing an embedder](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/choose_an_embedder) covers the trade-offs between hosted and local models.

## Search cutoff vs. other performance optimizations

The search cutoff is a reactive measure that limits query time after it becomes a problem. For proactive performance improvements, consider:

* [Configuring searchable attributes](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_searchable_attributes) to reduce the number of fields Meilisearch scans
* [Configuring stop words](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_stop_words) to eliminate common terms from indexing
* [Disabling prefix search](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_prefix_search) if search-as-you-type is not needed

These optimizations reduce the work Meilisearch does during each query, which may eliminate the need for a cutoff entirely.

> [!NOTE]
>
> For the full API reference, see [get search cutoff](https://www.meilisearch.com/docs/reference/api/settings/get-searchcutoffms).


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.