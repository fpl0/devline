# Building recommendations with similar documents

> Use the /similar endpoint to recommend documents that are semantically close to a given item, powering "More like this" and "Related items" features.

The `/similar` endpoint finds documents that are semantically close to a reference document. Once you have configured an embedder, you can use it to build recommendation features such as "More like this", "Related items", or "You might also like".

> [!NOTE]
>
> This guide requires a configured embedder. If you haven't set one up yet, see the [hybrid search getting started](https://www.meilisearch.com/docs/capabilities/hybrid_search/getting_started) guide.

## Create an index with embeddings

Create an index called `movies` and add this [`movies.json`](https://www.meilisearch.com/docs/assets/datasets/movies.json) dataset to it. If necessary, consult the [getting started](https://www.meilisearch.com/docs/getting_started/first_project) for more instructions on index creation.

Then configure an OpenAI embedder:

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/movies/settings/embedders' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "movies-text": {
      "source": "openAi",
      "apiKey": "OPENAI_API_KEY",
      "model": "text-embedding-3-small",
      "documentTemplate": "A movie titled {{doc.title}} whose plot is: {{doc.overview}}"
    }
  }'
```

Replace `MEILISEARCH_URL`, `MEILISEARCH_KEY`, and `OPENAI_API_KEY` with the corresponding values in your application.

Meilisearch will start generating embeddings for all documents. Use the returned `taskUid` to [track the progress of this task](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations).

## Find a reference document

To recommend similar items, you first need a reference document. This is typically the item a user is currently viewing. For this example, search for "batman":

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/search' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "q": "batman",
    "hybrid": {
      "semanticRatio": 0.5,
      "embedder": "movies-text"
    }
  }'
```

The top result is the movie "Batman" with `id` 192. Use this as the reference document.

## Retrieve similar documents

Pass the reference document's `id` to the [`/similar` endpoint](https://www.meilisearch.com/docs/reference/api/similar-documents/get-similar-documents-with-post), specifying your embedder:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/similar' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "id": 192,
    "embedder": "movies-text"
  }'
```

Meilisearch returns the 20 documents most similar to the reference movie. Display these as recommendations to your users.

## Use cases

* **E-commerce**: "Customers also viewed" or "Similar products" on product detail pages
* **Content platforms**: "Related articles" or "More like this" alongside the current content
* **Media streaming**: "Because you watched X" recommendations based on the current title

## Next steps

> [card] [Choose an embedder](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/choose_an_embedder)
>
> Compare embedding providers and pick the right one for your use case.

> [card] [Personalized search](https://www.meilisearch.com/docs/capabilities/personalization/getting_started/personalized_search)
>
> Combine recommendations with personalized search for a tailored experience.

> [card] [Similar documents API reference](https://www.meilisearch.com/docs/reference/api/similar-documents/get-similar-documents-with-post)
>
> Full API reference for the /similar endpoint.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.