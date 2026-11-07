# Boost results across indexes

> Use federation weights to control which index's results rank higher in federated multi-search.

When using [federated search](https://www.meilisearch.com/docs/capabilities/multi_search/getting_started/federated_search), all results from different indexes are merged into a single ranked list. By default, results from every index carry the same weight. You can change this by assigning different weights to each query, making results from one index rank higher than others.

## How weights work

Each query in a federated [multi-search](https://www.meilisearch.com/docs/capabilities/multi_search/overview) request can include a `federationOptions` object with a `weight` property. The weight is a floating-point number that multiplies the [ranking rules](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/ranking_rules) relevancy score of results from that query:

* The default weight is `1.0`
* A weight higher than `1.0` promotes results from that query
* A weight lower than `1.0` demotes results from that query
* A weight of `0.0` effectively excludes results from that query

## Boost results from a specific index

Suppose you have a CRM application with three indexes: `profiles`, `chats`, and `tickets`. When searching for a person's contact information, results from the `profiles` index are most likely to contain what you need. You can boost those results by giving the `profiles` query a higher weight.

```bash
curl \
  -X POST 'MEILISEARCH_URL/multi-search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "federation": {},
    "queries": [
      {
        "indexUid": "chats",
        "q": "rotondo"
      },
      {
        "indexUid": "profiles",
        "q": "rotondo",
        "federationOptions": { "weight": 1.2 }
      },
      {
        "indexUid": "tickets",
        "q": "rotondo"
      }
    ]
  }'
```

In this request, the `profiles` query has a weight of `1.2`, while the other queries use the default weight of `1.0`. This means matching profiles will rank higher in the merged result list.

The response returns all results in a single list, with profile matches promoted toward the top:

```json
{
  "hits": [
    {
      "id": 1,
      "name": "Riccardo Rotondo",
      "email": "riccardo.rotondo@example.com",
      "_federation": {
        "indexUid": "profiles",
        "queriesPosition": 1
      }
    },
    {
      "id": 5,
      "client_name": "Riccardo Rotondo",
      "message": "Please use riccardo@work.com for follow-ups",
      "_federation": {
        "indexUid": "chats",
        "queriesPosition": 0
      }
    }
  ],
  "processingTimeMs": 0,
  "limit": 20,
  "offset": 0,
  "estimatedTotalHits": 3
}
```

Each hit includes a `_federation` object showing which index and query position it came from.

## Practical example: products over blog posts

For an ecommerce site with both a `products` index and a `blog_posts` index, you likely want product listings to appear before blog content when users search:

```bash
curl \
  -X POST 'MEILISEARCH_URL/multi-search' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "federation": {},
    "queries": [
      {
        "indexUid": "products",
        "q": "wireless headphones",
        "federationOptions": { "weight": 1.5 }
      },
      {
        "indexUid": "blog_posts",
        "q": "wireless headphones",
        "federationOptions": { "weight": 0.8 }
      }
    ]
  }'
```

With a weight of `1.5` on products and `0.8` on blog posts, product results will consistently appear higher in the merged list unless a blog post has a significantly better relevancy match.

## Tips for choosing weights

* Start with small adjustments (for example, `1.2` for promoted indexes and `0.8` for demoted ones) and test with real queries
* Use larger differences (for example, `2.0` vs `0.5`) when you need a strong preference for one content type
* Remember that weights multiply the relevancy score, so a highly relevant result from a low-weight index can still outrank a weakly relevant result from a high-weight index

## Next steps

> [card] [Federated search](https://www.meilisearch.com/docs/capabilities/multi_search/getting_started/federated_search)
>
> Learn how to perform a basic federated search

> [card] [Build a unified search bar](https://www.meilisearch.com/docs/capabilities/multi_search/how_to/build_unified_search_bar)
>
> Combine federated search with a frontend UI


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.