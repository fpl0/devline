# Add and update documents

> Add new documents, replace existing ones, or partially update specific fields using the documents API.

Meilisearch provides three document operations: add or replace, add or update, and delete. This guide explains the difference between each operation and when to use them.

## Add or replace documents

Use `POST /indexes/{index_uid}/documents` to add new documents or replace existing ones. If a document with the same [primary key](https://www.meilisearch.com/docs/resources/internals/primary_key) already exists, Meilisearch **replaces the entire document** with the new version.

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/documents' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    {
      "id": 287947,
      "title": "Shazam",
      "poster": "https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg",
      "overview": "A boy is given the ability to become an adult superhero in times of need with a single magic word.",
      "release_date": "2019-03-23"
    }
  ]'
```

```javascript
client.index('movies').addDocuments([{
    id: 287947,
    title: 'Shazam',
    poster: 'https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg',
    overview: 'A boy is given the ability to become an adult superhero in times of need with a single magic word.',
    release_date: '2019-03-23'
}], { skipCreation: true })
```

```python
client.index('movies').add_documents([{
  'id': 287947,
  'title': 'Shazam',
  'poster': 'https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg',
  'overview': 'A boy is given the ability to become an adult superhero in times of need with a single magic word.',
  'release_date': '2019-03-23'
}], skip_creation=True)
```

```php
$client->index('movies')->addDocuments([
  [
    'id' => 287947,
    'title' => 'Shazam',
    'poster' => 'https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg',
    'overview' => 'A boy is given the ability to become an adult superhero in times of need with a single magic word.',
    'release_date' => '2019-03-23'
  ]
]);
```

```java
client.index("movies").addDocuments("[{"
  + "\"id\": 287947,"
  + "\"title\": \"Shazam\","
  + "\"poster\": \"https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg\","
  + "\"overview\": \"A boy is given the ability to become an adult superhero in times of need with a single magic word.\","
  + "\"release_date\": \"2019-03-23\""
  + "}]"
);
```

```ruby
client.index('movies').add_documents([
  {
    id: 287947,
    title: 'Shazam',
    poster: 'https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg',
    overview: 'A boy is given the ability to become an adult superhero in times of need with a single magic word.',
    release_date: '2019-03-23'
  }
])
```

```go
documents := []map[string]interface{}{
  {
    "id":           287947,
    "title":        "Shazam",
    "poster":       "https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg",
    "overview":     "A boy is given the ability to become an adult superhero in times of need with a single magic word.",
    "release_date": "2019-03-23",
  },
}
options := &meilisearch.DocumentOptions{SkipCreation: false}
client.Index("movies").AddDocuments(documents, options)
```

```csharp
var movie = new[]
{
    new Movie
    {
          Id = "287947",
          Title = "Shazam",
          Poster = "https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg",
          Overview = "A boy is given the ability to become an adult superhero in times of need with a single magic word.",
          ReleaseDate = "2019-03-23"
    }
};
await index.AddDocumentsAsync(movie);
```

```rust
let task: TaskInfo = client
  .index("movies")
  .add_or_replace(&[
    Movie {
      id: 287947,
      title: "Shazam".to_string(),
      poster: "https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg".to_string(),
      overview: "A boy is given the ability to become an adult superhero in times of need with a single magic word.".to_string(),
      release_date: "2019-03-23".to_string(),
    }
  ], None)
  .await
  .unwrap();
```

```swift
let documentJsonString = """
[
  {
    "reference_number": 287947,
    "title": "Shazam",
    "poster": "https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg",
    "overview": "A boy is given the ability to become an adult superhero in times of need with a single magic word.",
    "release_date": "2019-03-23"
  }
]
"""
let documents: Data = documentJsonString.data(using: .utf8)!

client.index("movies").addDocuments(documents: documents) { (result) in
    switch result {
    case .success(let task):
        print(task)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('movies').addDocuments([
  {
    'id': 287947,
    'title': 'Shazam',
    'poster':
        'https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg',
    'overview':
        'A boy is given the ability to become an adult superhero in times of need with a single magic word.',
    'release_date': '2019-03-23'
  }
]);
```

This operation is best when you have complete document objects and want to ensure the stored version matches exactly what you send.

> [!WARNING]
>
> When replacing a document, any fields present in the old version but missing from the new version are removed. Always include all fields you want to keep.

### Example

Suppose your index contains this document:

```json
{
  "id": 287947,
  "title": "Shazam",
  "overview": "A boy becomes a superhero.",
  "genres": ["Action", "Comedy"]
}
```

If you send a POST request with:

```json
{
  "id": 287947,
  "title": "Shazam!",
  "overview": "A boy is given the ability to become an adult superhero."
}
```

The stored document becomes:

```json
{
  "id": 287947,
  "title": "Shazam!",
  "overview": "A boy is given the ability to become an adult superhero."
}
```

The `genres` field is gone because it was not included in the replacement.

## Add or update documents

Use `PUT /indexes/{index_uid}/documents` to add new documents or partially update existing ones. If a document with the same primary key already exists, Meilisearch **merges the new fields** into the existing document. Fields not included in the update remain unchanged. Partial updates apply only to top-level fields: updating an object field replaces the entire object, removing any omitted subfields.

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/movies/documents' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    {
      "id": 287947,
      "title": "Shazam ⚡️",
      "genres": "comedy"
    }
  ]'
```

```javascript
client.index('movies').updateDocuments([{
    id: 287947,
    title: 'Shazam ⚡️',
    genres: 'comedy'
}], { skipCreation: true })
```

```python
client.index('movies').update_documents([{
    'id': 287947,
    'title': 'Shazam ⚡️',
    'genres': 'comedy'
}], skip_creation=True)
```

```php
$client->index('movies')->updateDocuments([
  [
    'id' => 287947,
    'title' => 'Shazam ⚡️',
    'genres' => 'comedy'
  ]
]);
```

```java
client.index("movies").updateDocuments("[{
  + "\"id\": 287947,"
  + "\"title\": \"Shazam ⚡️\","
  + "\"genres\": \"comedy\""
  + "}]"
);
```

```ruby
client.index('movies').update_documents([
  {
    id: 287947,
    title: 'Shazam ⚡️',
    genres: 'comedy'
  }
])
```

```go
documents := []map[string]interface{}{
  {
    "id":     287947,
    "title":  "Shazam ⚡️",
    "genres": "comedy",
  },
}
options := &meilisearch.DocumentOptions{SkipCreation: true}
client.Index("movies").UpdateDocuments(documents, options)
```

```csharp
var movie = new[]
{
    new Movie { Id = "287947", Title = "Shazam ⚡️", Genres = "comedy" }
};
await index.UpdateDocumentsAsync(movie);
```

```rust
// Define the type of our documents
#[derive(Serialize, Deserialize)]
struct IncompleteMovie {
  id: usize,
  title: String,
  genres: String
}

let task: TaskInfo = client
  .index("movies")
  .add_or_update(&[
    IncompleteMovie {
      id: 287947,
      title: "Shazam ⚡️".to_string(),
      genres: "comedy".to_string()
    }
  ], None)
  .await
  .unwrap();
```

```swift
let documentJsonString = """
[
  {
    "reference_number": 287947,
    "title": "Shazam ⚡️",
    "genres": "comedy"
  }
]
"""
let documents: Data = documentJsonString.data(using: .utf8)!

client.index("movies").updateDocuments(documents: documents) { (result) in
    switch result {
    case .success(let task):
        print(task)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('movies').updateDocuments([
  {
    'id': 287947,
    'title': 'Shazam ⚡️',
    'genres': 'comedy',
  }
]);
```

This operation is ideal when you only need to change specific fields without resending the entire document.

### Example

Starting with the same document:

```json
{
  "id": 287947,
  "title": "Shazam",
  "overview": "A boy becomes a superhero.",
  "genres": ["Action", "Comedy"]
}
```

If you send a PUT request with:

```json
{
  "id": 287947,
  "title": "Shazam ⚡️",
  "genres": "comedy"
}
```

The stored document becomes:

```json
{
  "id": 287947,
  "title": "Shazam ⚡️",
  "overview": "A boy becomes a superhero.",
  "genres": "comedy"
}
```

The `overview` field is preserved because the update only touched `title` and `genres`.

## Delete documents

Use `DELETE /indexes/{index_uid}/documents/{document_id}` to remove a single document by its primary key:

```bash
curl \
  -X DELETE 'MEILISEARCH_URL/indexes/movies/documents/25684' \
  -H 'Authorization: Bearer API_KEY'
```

```javascript
client.index('movies').deleteDocument(25684)
```

```python
client.index('movies').delete_document(25684)
```

```php
$client->index('movies')->deleteDocument(25684);
```

```java
client.index("movies").deleteDocument("25684");
```

```ruby
client.index('movies').delete_document(25684)
```

```go
client.Index("movies").DeleteDocument("25684")
```

```csharp
await client.Index("movies").DeleteOneDocumentAsync("25684");
```

```rust
let task: TaskInfo = client
  .index("movies")
  .delete_document(25684)
  .await
  .unwrap();
```

```swift
client.index("movies").deleteDocument("25684") { (result) in
    switch result {
    case .success(let task):
        print(task)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('movies').deleteDocument(25684);
```

Meilisearch also supports batch deletion and deletion by filter:

* **Delete by batch**: send a `POST /indexes/{index_uid}/documents/delete-batch` request with an array of document IDs
* **Delete by filter**: send a `POST /indexes/{index_uid}/documents/delete` request with a [filter expression](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/advanced/filter_expression_syntax) to remove all matching documents

## Choosing the right operation

| Operation | HTTP method | Behavior | Use when |
| - | - | - | - |
| Add or replace | `POST` | Replaces entire document | You have complete documents and want exact control |
| Add or update | `PUT` | Merges fields into existing document | You only need to change specific fields |
| Delete | `DELETE` | Removes document entirely | You need to remove documents from the index |

## Batch operations

All three operations support sending multiple documents in a single request. Send an array of documents in the request body:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/documents' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    { "id": 1, "title": "Movie One" },
    { "id": 2, "title": "Movie Two" },
    { "id": 3, "title": "Movie Three" }
  ]'
```

Batch operations are processed as a single [task](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations). Meilisearch handles large batches efficiently, so prefer sending documents in bulk rather than one at a time.

## Update without creating new documents

By default, both `POST` and `PUT` document operations create new documents if no document with the given primary key exists. To change this behavior, add the `skipCreation=true` query parameter to your request. When enabled, Meilisearch silently ignores any documents whose primary key does not match an existing document in the index.

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/documents?skipCreation=true' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    { "id": 1, "title": "Updated Title" },
    { "id": 99999, "title": "This document does not exist" }
  ]'
```

In this example, only document `1` is updated. Document `99999` is ignored because it does not already exist in the index.

This is useful when you want to safely update fields for existing documents without accidentally creating incomplete records.

## Retrieve multiple documents by ID

Use `POST /indexes/{index_uid}/documents/fetch` to retrieve specific documents by their primary keys:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/documents/fetch' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "ids": ["id1", "id2", "id3"]
  }'
```

Meilisearch returns the matching documents in the `results` array. Note that documents are not returned in the order you queried them, and non-existent IDs are silently ignored.

> [!WARNING]
>
> Prefer `POST /indexes/{index_uid}/documents/fetch` over `GET /indexes/{index_uid}/documents`. The GET variant is discouraged unless you have a specific reason to use it (for example, to take advantage of HTTP caching at the proxy or CDN level). The GET route accepts fewer parameters and only supports string filter expressions, while the POST route accepts the richer JSON body used throughout this guide.

### Filter the documents you fetch

You can pass a `filter` expression to `POST /indexes/{index_uid}/documents/fetch` to retrieve only documents that match a condition:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/documents/fetch' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "filter": "genres = Action AND rating > 8"
  }'
```

> [!NOTE]
>
> Any attribute you reference in a document `filter` must first be declared in the index's [`filterableAttributes`](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/getting_started) setting. This rule is the same as for search filters and is specific to the documents endpoint when filtering the documents you retrieve or delete.

## Supported content types

By default, Meilisearch expects a JSON array in the request body and the `Content-Type: application/json` header. The documents endpoint also accepts NDJSON (`application/x-ndjson`) and CSV (`text/csv`) payloads when you set the matching header.

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/documents' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: text/csv' \
  --data-binary @movies.csv
```

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/documents' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/x-ndjson' \
  --data-binary @movies.ndjson
```

When uploading CSV data, you can override the default comma separator with the `csvDelimiter` query parameter (for example, `?csvDelimiter=;`).

> [!WARNING]
>
> `csvDelimiter` is only valid when the request uses `Content-Type: text/csv`. Passing it alongside a JSON or NDJSON payload returns an error.

## Next steps

> [card] [Documents API reference](https://www.meilisearch.com/docs/reference/api/documents/add-or-replace-documents)
>
> Full API reference for document operations

> [card] [Indexing overview](https://www.meilisearch.com/docs/capabilities/indexing/overview)
>
> Learn more about how indexing works in Meilisearch

> [card] [Monitor tasks](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/monitor_tasks)
>
> Track the status of your document operations


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.