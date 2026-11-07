# Combine text and image search

> Use multi-search to run text-based and image-based semantic searches in a single request, leveraging multiple embedders for richer results.

Multi-search lets you query the same index with different embedders in a single request. This is useful when your index has both a text embedder and an image embedder configured, and you want to combine their results.

## Configure multiple embedders

Before running semantic multi-search queries, configure at least two embedders on your index. For example, a text embedder using OpenAI and an image embedder using a multimodal REST provider:

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/products/settings' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "embedders": {
      "text": {
        "source": "openAi",
        "apiKey": "OPEN_AI_API_KEY",
        "model": "text-embedding-3-small",
        "documentTemplate": "A product called {{doc.name}}: {{doc.description}}"
      },
      "image": {
        "source": "rest",
        "url": "https://api.voyageai.com/v1/multimodalembeddings",
        "apiKey": "VOYAGE_API_KEY",
        "request": {
          "inputs": [
            {
              "content": [
                { "type": "image_url", "image_url": "{{media.image}}" }
              ]
            }
          ],
          "model": "voyage-multimodal-3"
        },
        "response": {
          "data": [{ "embedding": "{{embedding}}" }]
        },
        "indexingFragments": {
          "image": { "value": "{{doc.image_url}}" }
        },
        "searchFragments": {
          "image": { "value": "{{media.image}}" }
        }
      }
    }
  }'
```

For more on embedder configuration, see [Configure an OpenAI embedder](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/configure_openai_embedder) and [Image search with a multimodal embedder](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/image_search_with_multimodal).

## Search with text and image in one request

Use federated multi-search to combine a text query and an image query, each targeting a different embedder on the same index:

```bash
curl \
  -X POST 'MEILISEARCH_URL/multi-search' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "federation": {},
    "queries": [
      {
        "indexUid": "products",
        "q": "comfortable running shoes",
        "hybrid": {
          "embedder": "text",
          "semanticRatio": 0.8
        }
      },
      {
        "indexUid": "products",
        "media": {
          "image": "https://example.com/red-sneaker.jpg"
        },
        "hybrid": {
          "embedder": "image",
          "semanticRatio": 1.0
        }
      }
    ]
  }'
```

Meilisearch runs both queries and merges the results into a single ranked list. Products matching both the text description and the image will rank higher.

## Control the balance between text and image results

Use `federationOptions.weight` to control how much each query contributes to the final ranking:

```bash
curl \
  -X POST 'MEILISEARCH_URL/multi-search' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "federation": {},
    "queries": [
      {
        "indexUid": "products",
        "q": "comfortable running shoes",
        "hybrid": {
          "embedder": "text",
          "semanticRatio": 0.8
        },
        "federationOptions": { "weight": 1.0 }
      },
      {
        "indexUid": "products",
        "media": {
          "image": "https://example.com/red-sneaker.jpg"
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

In this example, text results have twice the weight of image results. Adjust the weights to match your use case.

## Combine keyword, text semantic, and image search

You can go further and combine all three search modes in one request: keyword search, semantic text search, and image search.

```bash
curl \
  -X POST 'MEILISEARCH_URL/multi-search' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "federation": {},
    "queries": [
      {
        "indexUid": "products",
        "q": "red running shoes",
        "hybrid": {
          "embedder": "text",
          "semanticRatio": 0.0
        },
        "federationOptions": { "weight": 1.0 }
      },
      {
        "indexUid": "products",
        "q": "red running shoes",
        "hybrid": {
          "embedder": "text",
          "semanticRatio": 1.0
        },
        "federationOptions": { "weight": 0.8 }
      },
      {
        "indexUid": "products",
        "media": {
          "image": "https://example.com/red-sneaker.jpg"
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

This sends three queries to the same index:

1. **Keyword search** (`semanticRatio: 0.0`) for exact term matches
2. **Semantic text search** (`semanticRatio: 1.0`) for meaning-based matches
3. **Image search** for visually similar products

Meilisearch merges all results and ranks them using the configured weights.

## Search across multiple indexes with different embedders

You can also target different indexes, each with its own embedders. For example, searching a `products` index with a text embedder and an `inspiration` index with an image embedder:

```bash
curl \
  -X POST 'MEILISEARCH_URL/multi-search' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "federation": {},
    "queries": [
      {
        "indexUid": "products",
        "q": "summer dress",
        "hybrid": {
          "embedder": "text",
          "semanticRatio": 0.7
        },
        "federationOptions": { "weight": 1.0 }
      },
      {
        "indexUid": "inspiration",
        "media": {
          "image": "https://example.com/summer-outfit.jpg"
        },
        "hybrid": {
          "embedder": "image",
          "semanticRatio": 1.0
        },
        "federationOptions": { "weight": 0.6 }
      }
    ]
  }'
```

## Next steps

> [card] [Configure embedders](https://www.meilisearch.com/docs/capabilities/hybrid_search/getting_started)
>
> Set up text and multimodal embedders for semantic search.

> [card] [Image search](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/image_search_with_multimodal)
>
> Configure a multimodal embedder for image-based search.

> [card] [Multiple embedders](https://www.meilisearch.com/docs/capabilities/hybrid_search/advanced/multiple_embedders)
>
> Learn how to configure and use multiple embedders on the same index.

> [card] [Boost results](https://www.meilisearch.com/docs/capabilities/multi_search/how_to/boost_results_across_indexes)
>
> Fine-tune federation weights to control result ranking.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.