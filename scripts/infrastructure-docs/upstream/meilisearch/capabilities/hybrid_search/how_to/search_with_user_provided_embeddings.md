# Use AI-powered search with user-provided embeddings

> This guide shows how to perform AI-powered searches with user-generated embeddings instead of relying on a third-party tool.

## Configure a custom embedder

Configure the `embedder` index setting, setting its source to `userProvided`:

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/INDEX_NAME/settings' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "embedders": {
      "EMBEDDER_NAME": {
        "source": "userProvided",
        "dimensions": MODEL_DIMENSIONS
      }
    }
  }'
```

> [!WARNING]
>
> Embedders with `source: userProvided` are incompatible with `documentTemplate` and `documentTemplateMaxBytes`.

## Add documents to Meilisearch

Next, use [the `/documents` endpoint](https://www.meilisearch.com/docs/reference/api/documents/list-documents-with-get) to upload vectorized documents. Place vector data in your documents' `_vectors` field:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/INDEX_NAME/documents' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    { "id": 0, "_vectors": { "EMBEDDER_NAME": [0, 0.8, -0.2]}, "text": "frying pan" },
    { "id": 1, "_vectors": { "EMBEDDER_NAME": [1, -0.2, 0]}, "text": "baking dish" }
  ]'
```

## The `_vectors` field in detail

Meilisearch stores pre-computed embeddings under the reserved `_vectors` field of a document. The field is an object whose keys match the names of the embedders configured in your index settings.

### Full object form: `embeddings` + `regenerate`

Each embedder entry accepts two fields, `embeddings` and `regenerate`:

```json
{
  "id": 0,
  "title": "Kung Fu Panda",
  "_vectors": {
    "default": {
      "embeddings": [0.003, 0.1, 0.75],
      "regenerate": false
    }
  }
}
```

* `embeddings` is optional. It must be an array of numbers representing a single embedding for that document. It may also be an array of arrays of numbers, representing multiple embeddings for the same document. `embeddings` defaults to `null`.
* `regenerate` is mandatory and must be a boolean. If `regenerate` is `true`, Meilisearch automatically generates embeddings for that document immediately and every time the document is updated. If `regenerate` is `false`, Meilisearch keeps the last value of `embeddings` on document updates and never overwrites it.

> [!TIP]
>
> Use `embeddings` as an array of arrays when a single document should be represented by multiple vectors (for example, one embedding per paragraph, or one per language).

### Array shorthand

You may also use an array shorthand to add embeddings to a document:

```json
{
  "_vectors": {
    "default": [0.003, 0.1, 0.75]
  }
}
```

Vector embeddings added with the shorthand are not replaced when Meilisearch generates new embeddings. The example above is equivalent to:

```json
{
  "_vectors": {
    "default": {
      "embeddings": [0.003, 0.1, 0.75],
      "regenerate": false
    }
  }
}
```

### Null or empty embedder entries

If the key for an embedder inside `_vectors` is empty or `null`, Meilisearch treats the document as not having any embeddings for that embedder. This document is then returned last during AI-powered searches.

## Vector search with user-provided embeddings

When using a custom embedder, you must vectorize both your documents and user queries.

Once you have the query's vector, pass it to the `vector` search parameter to perform an AI-powered search:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/INDEX_NAME/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "vector": [0, 1, 2],
    "hybrid": {
      "embedder": "EMBEDDER_NAME"
    }
  }'
```

```python
client.index('books').search('',{
  "vector": [0, 1, 2],
  "hybrid": {
    "embedder": "EMBEDDER_NAME"
  }
})
```

```rust
let results = index
  .search()
  .with_vector(&[0.0, 1.0, 2.0])
  .with_hybrid("EMBEDDER_NAME", 1.0)
  .execute()
  .await
  .unwrap();
```

`vector` must be an array of numbers indicating the search vector. You must generate these yourself when using vector search with user-provided embeddings.

> [!NOTE]
>
> `vector` is mandatory when performing searches with `userProvided` embedders. You may also use `vector` with automatic embedders to override an embedder's automatic vector generation, for example to experiment with a custom-generated query vector without changing your embedder configuration.

`vector` can be used together with [other search parameters](https://www.meilisearch.com/docs/reference/api/search/search-with-post), including [`filter`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-filter) and [`sort`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-sort):

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/INDEX_NAME/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "vector": [0, 1, 2],
    "filter": "price < 10",
    "sort": ["price:asc"],
    "hybrid": {
      "embedder": "EMBEDDER_NAME"
    }
  }'
```

## Return stored vectors with `retrieveVectors`

Set `retrieveVectors: true` on a search request to return document and query embeddings alongside the search results. When enabled, Meilisearch displays vector data in each document's `_vectors` field.

> [!WARNING]
>
> `_vectors` must be included in the index's `displayedAttributes` list for it to be returned in the response. If `displayedAttributes` does not contain `_vectors` (or `*`), `retrieveVectors` has no visible effect on the response payload.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.