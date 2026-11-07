# Configure searchable attributes

> Choose which document fields Meilisearch scans during a search query by setting the searchableAttributes index setting.

By default, Meilisearch searches through all document fields. Use the `searchableAttributes` setting to limit which fields are searchable and control their relative importance in the [attribute ranking order](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/attribute_ranking_order). This also affects [ranking rules](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/ranking_rules) that depend on attribute order.

## Why configure searchable attributes

There are two main reasons to customize searchable attributes:

1. **Improve relevancy**: Fields listed earlier in the `searchableAttributes` array have a greater impact on relevancy. If a match is found in the first attribute, that document ranks higher than one where the match appears in a later attribute.
2. **Improve performance**: Reducing the number of searchable fields means Meilisearch has less data to scan during each query, which can speed up search on large datasets.

For example, in a movies index with fields like `id`, `title`, `overview`, `genres`, and `release_date`, you probably want `title` to carry the most weight, followed by `overview` and `genres`. The `id` and `release_date` fields are not useful for text search and can be excluded.

## Check current searchable attributes

Retrieve the current `searchableAttributes` setting for an index:

```bash
curl \
  -X GET 'MEILISEARCH_URL/indexes/movies/settings/searchable-attributes' \
  -H 'Authorization: Bearer API_KEY'
```

```javascript
client.index('movies').getSearchableAttributes()
```

```python
client.index('movies').get_searchable_attributes()
```

```php
$client->index('movies')->getSearchableAttributes();
```

```java
client.index("movies").getSearchableAttributesSettings();
```

```ruby
client.index('movies').searchable_attributes
```

```go
client.Index("movies").GetSearchableAttributes()
```

```csharp
await client.Index("movies").GetSearchableAttributesAsync();
```

```rust
let searchable_attributes: Vec<String> = client
  .index("movies")
  .get_searchable_attributes()
  .await
  .unwrap();
```

```swift
client.index("movies").getSearchableAttributes { (result) in
    switch result {
    case .success(let searchableAttributes):
        print(searchableAttributes)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('movies').getSearchableAttributes();
```

By default, the response is `["*"]`, meaning all attributes are searchable in their order of appearance.

## Update searchable attributes

Set the `searchableAttributes` list to control which fields are searchable and their ranking order. Fields listed first have the highest impact on relevancy:

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/movies/settings/searchable-attributes' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    "title",
    "overview",
    "genres"
  ]'
```

```javascript
client.index('movies').updateSearchableAttributes([
  'title',
  'overview',
  'genres'
])
```

```python
client.index('movies').update_searchable_attributes([
    'title',
    'overview',
    'genres'
])
```

```php
$client->index('movies')->updateSearchableAttributes([
  'title',
  'overview',
  'genres'
]);
```

```java
client.index("movies").updateSearchableAttributesSettings(new String[]
{
  "title",
  "overview",
  "genres"
});
```

```ruby
client.index('movies').update_searchable_attributes([
  'title',
  'overview',
  'genres'
])
```

```go
searchableAttributes := []string{
  "title",
  "overview",
  "genres",
}
client.Index("movies").UpdateSearchableAttributes(&searchableAttributes)
```

```csharp
await client.Index("movies").UpdateSearchableAttributesAsync(new[] {"title", "overview", "genres"});
```

```rust
let searchable_attributes = [
  "title",
  "overview",
  "genres"
];

let task: TaskInfo = client
  .index("movies")
  .set_searchable_attributes(&searchable_attributes)
  .await
  .unwrap();
```

```swift
let searchableAttributes: [String] = ["title", "overview", "genres"]
client.index("movies").updateSearchableAttributes(searchableAttributes) { (result) in
    switch result {
    case .success(let task):
        print(task)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client
    .index('movies')
    .updateSearchableAttributes(['title', 'overview', 'genres']);
```

This configuration makes `title` the most important searchable field, followed by `overview`, then `genres`. Fields not in the list (such as `id` and `release_date`) are no longer searchable.

> [!WARNING]
>
> Updating `searchableAttributes` triggers a re-indexing of all documents in the index. This is an [asynchronous](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations) operation. Use the [task API](https://www.meilisearch.com/docs/reference/api/tasks/get-all-tasks) to monitor progress.

> [!WARNING]
>
> After manually updating `searchableAttributes`, new attributes found in subsequently indexed documents will not be automatically added to the list. You must either include them manually or [reset the setting](#reset-searchable-attributes).

> [!WARNING]
>
> **Known issue**: due to an implementation bug, manually updating `searchableAttributes` will change the displayed order of document fields in the JSON response. This behavior is inconsistent and will be fixed in a future release. If your application depends on a specific field order in responses, rely on explicit key access rather than the order returned by Meilisearch.

## Reset searchable attributes

Reset `searchableAttributes` to its default value (`["*"]`), making all fields searchable again:

```bash
curl \
  -X DELETE 'MEILISEARCH_URL/indexes/movies/settings/searchable-attributes' \
  -H 'Authorization: Bearer API_KEY'
```

```javascript
client.index('movies').resetSearchableAttributes()
```

```python
client.index('movies').reset_searchable_attributes()
```

```php
$client->index('movies')->resetSearchableAttributes();
```

```java
client.index("movies").resetSearchableAttributesSettings();
```

```ruby
client.index('movies').reset_searchable_attributes
```

```go
client.Index("movies").ResetSearchableAttributes()
```

```csharp
await client.Index("movies").ResetSearchableAttributesAsync();
```

```rust
let task: TaskInfo = client
  .index("movies")
  .reset_searchable_attributes()
  .await
  .unwrap();
```

```swift
client.index("movies").resetSearchableAttributes { (result) in
    switch result {
    case .success(let task):
        print(task)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('movies').resetSearchableAttributes();
```

After resetting, new attributes will once again be automatically added to the searchable attributes list as documents are indexed.

## Restrict searchable attributes at query time

If you need to limit which attributes are searched for a specific query without changing the index setting, use the `attributesToSearchOn` search parameter:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/search' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "adventure",
    "attributesToSearchOn": ["title"]
  }'
```

This searches only the `title` field for this request, regardless of the index-level `searchableAttributes` setting. The attributes specified must be a subset of the configured `searchableAttributes`.

> [!WARNING]
>
> `attributesToSearchOn` narrows the search to the fields you list, so documents that only match in other searchable fields are silently excluded from the response. For example, if your index has `searchableAttributes: ["title", "overview", "genre"]` and you set `attributesToSearchOn: ["overview"]`, a document whose only match is in `title` or `genre` will not appear in the results, even though those fields are otherwise searchable. No error is raised; the results are simply narrower than you may expect.

> [!NOTE]
>
> For more details on how searchable and displayed attributes work together, see [displayed and searchable attributes](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_displayed_attributes). For the full API reference, see [get searchable attributes](https://www.meilisearch.com/docs/reference/api/settings/get-searchableattributes).


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.