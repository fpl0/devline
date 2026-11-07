# Multiple embedders

> Configure multiple embedders on a single index to combine text search, image search, and semantic search with different models.

Meilisearch supports configuring multiple embedders on the same index. Each embedder generates its own set of vectors, and you can target a specific embedder at search time. This lets you combine different search strategies (text, image, semantic) with specialized models for each.

## Why use multiple embedders

A single embedder is a good fit when all your searches are the same type. But real applications often need different search modes:

* **Text + image search**: use a text-optimized embedder alongside a multimodal embedder, so users can search with keywords or with images
* **Precision vs speed**: use a large, high-quality model for precise searches and a smaller, faster model for search-as-you-type suggestions
* **Different quality levels**: use a small model at full precision for quick queries and a large model with [binary quantization](https://www.meilisearch.com/docs/capabilities/hybrid_search/advanced/binary_quantization) for deep searches
* **Multilingual**: use a language-specific model for your primary language and a multilingual model as a fallback
* **Federated search**: combine full-text, semantic, and image results in a single [federated search](https://www.meilisearch.com/docs/capabilities/multi_search/getting_started/federated_search) request, each powered by the best model for its task

## Configure multiple embedders

Add multiple keys to the `embedders` setting. Each key is a named embedder with its own configuration:

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/products/settings/embedders' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "text": {
      "source": "openAi",
      "apiKey": "OPEN_AI_API_KEY",
      "model": "text-embedding-3-small"
    },
    "image": {
      "source": "rest",
      "url": "https://api.voyageai.com/v1/multimodalembeddings",
      "apiKey": "VOYAGE_API_KEY",
      "indexingFragments": {
        "poster": {
          "value": {
            "content": [
              { "type": "image_url", "image_url": "{{doc.poster_url}}" }
            ]
          }
        }
      },
      "searchFragments": {
        "image": {
          "value": {
            "content": [
              { "type": "image_url", "image_url": "{{media.image}}" }
            ]
          }
        }
      },
      "request": {
        "inputs": ["{{fragment}}", "{{..}}"],
        "model": "voyage-multimodal-3"
      },
      "response": {
        "data": [{ "embedding": "{{embedding}}" }, "{{..}}"]
      }
    }
  }'
```

This configures two embedders: `text` for keyword-aware semantic search and `image` for visual similarity search.

## Search with a specific embedder

Specify which embedder to use with the `hybrid.embedder` parameter:

```bash
# Semantic text search
curl -X POST 'MEILISEARCH_URL/indexes/products/search' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "q": "comfortable running shoes",
    "hybrid": {
      "embedder": "text",
      "semanticRatio": 0.5
    }
  }'
```

```bash
# Image search
curl -X POST 'MEILISEARCH_URL/indexes/products/search' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "media": {
      "image": "https://example.com/shoe.jpg"
    },
    "hybrid": {
      "embedder": "image",
      "semanticRatio": 1.0
    }
  }'
```

## Combine embedders with federated search

The most powerful use case for multiple embedders is [federated search](https://www.meilisearch.com/docs/capabilities/multi_search/getting_started/federated_search). You can run full-text, semantic, and image searches in a single request and merge the results:

```bash
curl -X POST 'MEILISEARCH_URL/multi-search' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "federation": {},
    "queries": [
      {
        "indexUid": "products",
        "q": "running shoes",
        "hybrid": {
          "embedder": "text",
          "semanticRatio": 0.0
        },
        "federationOptions": { "weight": 1.0 }
      },
      {
        "indexUid": "products",
        "q": "running shoes",
        "hybrid": {
          "embedder": "text",
          "semanticRatio": 1.0
        },
        "federationOptions": { "weight": 0.8 }
      },
      {
        "indexUid": "products",
        "media": {
          "image": "https://example.com/shoe.jpg"
        },
        "hybrid": {
          "embedder": "image",
          "semanticRatio": 1.0
        },
        "federationOptions": { "weight": 0.5 }
      }
    ]
  }'
```

This single request combines:

1. **Full-text search** (`semanticRatio: 0.0`) with the highest weight for keyword-relevant results
2. **Semantic text search** (`semanticRatio: 1.0`) for meaning-based matches
3. **Image search** using a completely different model for visual similarity

Meilisearch merges all results into one ranked list using the [federation weights](https://www.meilisearch.com/docs/capabilities/multi_search/how_to/boost_results_across_indexes).

## Considerations

* Each embedder generates and stores its own vectors. More embedders means more disk usage and longer indexing times.
* You can use [binary quantization](https://www.meilisearch.com/docs/capabilities/hybrid_search/advanced/binary_quantization) on individual embedders to reduce storage (e.g., quantize the large model but keep the small one at full precision).
* [Composite embedders](https://www.meilisearch.com/docs/capabilities/hybrid_search/advanced/composite_embedders) can be combined with multiple embedders: use a fast local model for search and a cloud API for indexing, independently for each named embedder.

## Next steps

> [card] [Federated search](https://www.meilisearch.com/docs/capabilities/multi_search/getting_started/federated_search)
>
> Merge results from multiple queries into one ranked list

> [card] [Binary quantization](https://www.meilisearch.com/docs/capabilities/hybrid_search/advanced/binary_quantization)
>
> Reduce storage for high-dimensional embedders

> [card] [Image search](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/image_search_with_multimodal)
>
> Set up multimodal embedders for image search

> [card] [Choose an embedder](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/choose_an_embedder)
>
> Compare embedding providers for your use case


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.