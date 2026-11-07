# Primary key

> The primary key is a special field that must be present in all documents indexed by Meilisearch.

## Primary field

An [index](https://www.meilisearch.com/docs/resources/internals/indexes) in Meilisearch is a collection of [documents](https://www.meilisearch.com/docs/resources/internals/documents). Documents are composed of fields, each field containing an attribute and a value.

The primary field is a special field that must be present in all documents. Its attribute is the **[primary key](#primary-key-1)** and its value is the **[document id](#document-id)**. It uniquely identifies each document in an index, ensuring that **it is impossible to have two exactly identical documents** present in the same index.

### Example

Suppose we have an index of books. Each document contains a number of fields with data on the book's `author`, `title`, and `price`. More importantly, each document contains a **primary field** consisting of the index's **primary key** `id` and a **unique id**.

```json
[
  {
    "id": 1,
    "title": "Diary of a Wimpy Kid: Rodrick Rules",
    "author": "Jeff Kinney",
    "genres": ["comedy","humor"],
    "price": 5.00
  },
  {
    "id": 2,
    "title": "Black Leopard, Red Wolf",
    "author": "Marlon James",
    "genres": ["fantasy","drama"],
    "price": 5.00
  }
]
```

Aside from the primary key, **documents in the same index are not required to share attributes**. A book in this dataset could be missing the `title` or `genre` attribute and still be successfully indexed by Meilisearch, provided it has the `id` attribute.

### Primary key

The primary key is the attribute of the primary field.

Every index has a primary key, an attribute that must be shared across all documents in that index. If you attempt to add documents to an index and even a single one is missing the primary key, **none of the documents will be stored.**

#### Example

```json
{
    "id": 1,
    "title": "Diary of a Wimpy Kid",
    "author": "Jeff Kinney",
    "genres": ["comedy","humor"],
    "price": 5.00
  }
```

Each document in the above index is identified by a primary field containing the primary key `id` and a unique document id value.

### Document id

The document id is the value associated with the primary key. It is part of the primary field and acts as a unique identifier for each document in a given index.

Two documents in an index can have the same values for all attributes except the primary key. If two documents in the same index have the same id, then they are treated as the same document and **the preceding document will be overwritten**.

Document addition requests in Meilisearch are atomic. This means that **if the primary field value of even a single document in a batch is incorrectly formatted, an error will occur, and Meilisearch will not index documents in that batch.**

#### Example

Good:

```json
"id": "_Aabc012_"
```

Bad:

```json
"id": "@BI+* ^5h2%"
```

#### Formatting the document id

The document id must be an integer or a string. If the id is a string, it can only contain alphanumeric characters (`a-z`, `A-Z`, `0-9`), hyphens (`-`), and underscores (`_`).

## Setting the primary key

You can set the primary key explicitly or let Meilisearch infer it from your dataset. Whatever your choice, an index can have only one primary key at a time, and the primary key cannot be changed while documents are present in the index.

### Setting the primary key on index creation

When creating an index manually, you can explicitly indicate the primary key you want that index to use.

The code below creates an index called `books` and sets `reference_number` as its primary key:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "uid": "books",
    "primaryKey": "reference_number"
  }'
```

```javascript
client.createIndex('books', { primaryKey: 'reference_number' })
```

```python
client.create_index('books', {'primaryKey': 'reference_number'})
```

```php
$client->createIndex('books', ['primaryKey' => 'reference_number']);
```

```java
client.createIndex("books", "reference_number");
```

```ruby
client.create_index('books', primary_key: 'reference_number')
```

```go
client.CreateIndex(&meilisearch.IndexConfig{
  Uid: "books",
  PrimaryKey: "reference_number",
})
```

```csharp
TaskInfo task = await client.CreateIndexAsync("books", "reference_number");
```

```rust
client
  .create_index("books", Some("reference_number"))
  .await
  .unwrap();
```

```swift
client.createIndex(uid: "books", primaryKey: "reference_number") { (result) in
  switch result {
    case .success(let task):
        print(task)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.createIndex('books', primaryKey: 'reference_number');
```

```json
{
  "taskUid": 1,
  "indexUid": "books",
  "status": "enqueued",
  "type": "indexCreation",
  "enqueuedAt": "2022-09-20T12:06:24.364352Z"
}
```

### Setting the primary key on document addition

When adding documents to an empty index, you can explicitly set the index's primary key as part of the document addition request.

The code below adds a document to the `books` index and sets `reference_number` as that index's primary key:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/books/documents?primaryKey=reference_number' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    {
      "reference_number": 287947,
      "title": "Diary of a Wimpy Kid",
      "author": "Jeff Kinney",
      "genres": [
        "comedy",
        "humor"
      ],
      "price": 5.00
    }
  ]'
```

```javascript
client.index('books').addDocuments([
  {
    reference_number: 287947,
    title: 'Diary of a Wimpy Kid',
    author: 'Jeff Kinney',
    genres: ['comedy','humor'],
    price: 5.00
  }
], { primaryKey: 'reference_number' })
```

```python
client.index('books').add_documents([{
  'reference_number': 287947,
  'title': 'Diary of a Wimpy Kid',
  'author': 'Jeff Kinney',
  'genres': ['comedy', 'humor'],
  'price': 5.00
}], 'reference_number')
```

```php
$client->index('books')->addDocuments([
  [
    'reference_number' => 287947,
    'title' => 'Diary of a Wimpy Kid',
    'author' => 'Jeff Kinney',
    'genres' => ['comedy', 'humor'],
    'price' => 5.00
  ]
], 'reference_number');
```

```java
client.index("books").addDocuments("[{"
  + "\"reference_number\": 2879,"
  + "\"title\": \"Diary of a Wimpy Kid\","
  + "\"author\": \"Jeff Kinney\","
  + "\"genres\": [\"comedy\", \"humor\"],"
  + "\"price\": 5.00"
  + "}]"
, "reference_number");
```

```ruby
client.index('books').add_documents([
  {
    reference_number: 287947,
    title: 'Diary of a Wimpy Kid',
    author: 'Jeff Kinney',
    genres: ['comedy', 'humor'],
    price: 5.00
  }
], 'reference_number')
```

```go
documents := []map[string]interface{}{
  {
    "reference_number": 287947,
    "title":            "Diary of a Wimpy Kid",
    "author":           "Jeff Kinney",
    "genres":           []string{"comedy", "humor"},
    "price":            5.00,
  },
}
refrenceNumber := "reference_number"
client.Index("books").AddDocuments(documents, &refrenceNumber)
```

```csharp
await index.AddDocumentsAsync(
  new[] {
      new Book {
          ReferenceNumber = 287947,
          Title = "Diary of a Wimpy Kid",
          Author = "Jeff Kinney",
          Genres = new string[] { "comedy", "humor" },
          Price = 5.00
      }
  },
  "reference_number");
```

```rust
#[derive(Serialize, Deserialize)]
struct Book {
  reference_number: String,
  title: String,
  author: String,
  genres: Vec<String>,
  price: f64
}

let task: TaskInfo = client
  .index("books")
  .add_documents(&[
    Book {
      reference_number: "287947".to_string(),
      title: "Diary of a Wimpy Kid".to_string(),
      author: "Jeff Kinney".to_string(),
      genres: vec!["comedy".to_string(),"humor".to_string()],
      price: 5.00
    }
  ], Some("reference_number"))
  .await
  .unwrap();
```

```swift
let documents: Data = """
[
  {
    "reference_number": 287947,
    "title": "Diary of a Wimpy Kid",
    "author": "Jeff Kinney",
    "genres": ["comedy", "humor"],
    "price": 5
  }
]
""".data(using: .utf8)!

client.index("books").addDocuments(documents: documents, primaryKey: "reference_number") { (result) in
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
    'reference_number': 287947,
    'title': 'Diary of a Wimpy Kid',
    'author': 'Jeff Kinney',
    'genres': ['comedy', 'humor'],
    'price': 5.00
  }
], primaryKey: 'reference_number');
```

**Response:**

```json
{
  "taskUid": 1,
  "indexUid": "books",
  "status": "enqueued",
  "type": "documentAdditionOrUpdate",
  "enqueuedAt": "2022-09-20T12:08:55.463926Z"
}
```

### Changing your primary key with the update index endpoint

The primary key cannot be changed while documents are present in the index. To change the primary key of an index that already contains documents, you must therefore [delete all documents](https://www.meilisearch.com/docs/reference/api/documents/delete-all-documents) from that index, [change the primary key](https://www.meilisearch.com/docs/reference/api/indexes/update-index), then [add them](https://www.meilisearch.com/docs/reference/api/documents/add-or-replace-documents) again.

The code below updates the primary key to `title`:

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/books' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{ "primaryKey": "title" }'
```

```javascript
client.updateIndex('books', {
  primaryKey: 'title'
})
```

```python
client.index('books').update(primary_key='title')
```

```php
$client->updateIndex('books', ['primaryKey' => 'title']);
```

```java
client.updateIndex("books", "title");
```

```ruby
client.index('books').update(primary_key: 'title')
```

```go
client.Index("books").UpdateIndex(&meilisearch.UpdateIndexRequestParams{
  PrimaryKey: "title",
})
```

```csharp
TaskInfo task = await client.UpdateIndexAsync("books", "title");
```

```rust
let task = IndexUpdater::new("books", &client)
  .with_primary_key("title")
  .execute()
  .await
  .unwrap();
```

```swift
client.updateIndex(uid: "movies", primaryKey: "title") { (result) in
  switch result {
    case .success(let task):
        print(task)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.updateIndex('books', 'title');
```

**Response:**

```json
{
  "taskUid": 1,
  "indexUid": "books",
  "status": "enqueued",
  "type": "indexUpdate",
  "enqueuedAt": "2022-09-20T12:10:06.444672Z"
}
```

### Meilisearch guesses your primary key

Suppose you add documents to an index without previously setting its primary key. In this case, Meilisearch will automatically look for an attribute ending with the string `id` in a case-insensitive manner (for example, `uid`, `BookId`, `ID`) in your first document and set it as the index's primary key.

If Meilisearch finds [multiple attributes ending with `id`](#index_primary_key_multiple_candidates_found) or [cannot find a suitable attribute](#index_primary_key_no_candidate_found), it will throw an error. In both cases, the document addition process will be interrupted and no documents will be added to your index.

## Primary key errors

This section covers some primary key errors and how to resolve them.

### `index_primary_key_multiple_candidates_found`

This error occurs when you add documents to an index for the first time and Meilisearch finds multiple attributes ending with `id`. It can be resolved by [manually setting the index's primary key](#setting-the-primary-key-on-document-addition).

```json
{
  "uid": 4,
  "indexUid": "books",
  "status": "failed",
  "type": "documentAdditionOrUpdate",
  "canceledBy": null,
  "details": {
    "receivedDocuments": 5,
    "indexedDocuments": 5
  },
  "error": {
    "message": "The primary key inference failed as the engine found 2 fields ending with `id` in their names: 'id' and 'author_id'. Please specify the primary key manually using the `primaryKey` query parameter.",
    "code": "index_primary_key_multiple_candidates_found",
    "type": "invalid_request",
    "link": "https://docs.meilisearch.com/errors#index-primary-key-multiple-candidates-found"
  },
  "duration": "PT0.006002S",
  "enqueuedAt": "2023-01-17T10:44:42.625574Z",
  "startedAt": "2023-01-17T10:44:42.626041Z",
  "finishedAt": "2023-01-17T10:44:42.632043Z"
}
```

### `index_primary_key_no_candidate_found`

This error occurs when you add documents to an index for the first time and none of them have an attribute ending with `id`. It can be resolved by [manually setting the index's primary key](#setting-the-primary-key-on-document-addition), or ensuring that all documents you add possess an `id` attribute.

```json
{
  "uid": 1,
  "indexUid": "books",
  "status": "failed",
  "type": "documentAdditionOrUpdate",
  "canceledBy": null,
  "details": {
    "receivedDocuments": 5,
    "indexedDocuments": null
  },
  "error": {
    "message": "The primary key inference failed as the engine did not find any field ending with `id` in its name. Please specify the primary key manually using the `primaryKey` query parameter.",
    "code": "index_primary_key_no_candidate_found",
    "type": "invalid_request",
    "link": "https://docs.meilisearch.com/errors#index-primary-key-no-candidate-found"
  },
  "duration": "PT0.006579S",
  "enqueuedAt": "2023-01-17T10:19:14.464858Z",
  "startedAt": "2023-01-17T10:19:14.465369Z",
  "finishedAt": "2023-01-17T10:19:14.471948Z"
}
```

### `invalid_document_id`

This happens when your document id does not have the correct [format](#formatting-the-document-id). The document id can only be of type integer or string, composed of alphanumeric characters `a-z A-Z 0-9`, hyphens `-`, and underscores `_`.

```json
{
    "uid": 1,
    "indexUid": "books",
    "status": "failed",
    "type": "documentAdditionOrUpdate",
    "canceledBy": null,
    "details": {
        "receivedDocuments": 5,
        "indexedDocuments": null
        },
    "error": {
        "message": "Document identifier `1@` is invalid. A document identifier can be of type integer or string, only composed of alphanumeric characters (a-z A-Z 0-9), hyphens (-) and underscores (_).",
        "code": "invalid_document_id",
        "type": "invalid_request",
        "link": "https://docs.meilisearch.com/errors#invalid_document_id"
        },
    "duration": "PT0.009738S",
    "enqueuedAt": "2021-12-30T11:28:59.075065Z",
    "startedAt": "2021-12-30T11:28:59.076144Z",
    "finishedAt": "2021-12-30T11:28:59.084803Z"
}
```

### `missing_document_id`

This error occurs when your index already has a primary key, but one of the documents you are trying to add is missing this attribute.

```json
{
    "uid": 1,
    "indexUid": "books",
    "status": "failed",
    "type": "documentAdditionOrUpdate",
    "canceledBy": null,
    "details": {
        "receivedDocuments": 1,
        "indexedDocuments": null
        },
    "error": {
        "message": "Document doesn't have a `id` attribute: `{\"title\":\"Solaris\",\"author\":\"Stanislaw Lem\",\"genres\":[\"science fiction\"],\"price\":5.0.",
        "code": "missing_document_id",
        "type": "invalid_request",
        "link": "https://docs.meilisearch.com/errors#missing_document_id"
        },
    "duration": "PT0.007899S",
    "enqueuedAt": "2021-12-30T11:23:52.304689Z",
    "startedAt": "2021-12-30T11:23:52.307632Z",
    "finishedAt": "2021-12-30T11:23:52.312588Z"
}
```


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.