# Multi-index search

> Search multiple indexes in a single API request and receive separate result lists for each index.

Multi-index search lets you send several search queries in one HTTP request to the `/multi-search` endpoint. Each query targets a specific index and returns its own result list, making it ideal for search interfaces that display different content types in separate sections.

## Send a multi-search request

The `/multi-search` endpoint accepts an object with a `queries` array. Each element in the array is an independent search query with its own `indexUid`, search terms, and parameters.

```bash
curl \
  -X POST 'MEILISEARCH_URL/multi-search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "queries": [
      {
        "indexUid": "movies",
        "q": "pooh",
        "limit": 5
      },
      {
        "indexUid": "movies",
        "q": "nemo",
        "limit": 5
      },
      {
        "indexUid": "movie_ratings",
        "q": "us"
      }
    ]
  }'
```

```javascript
client.multiSearch({ queries: [
  {
    indexUid: 'movies',
    q: 'pooh',
    limit: 5,
  },
  {
    indexUid: 'movies',
    q: 'nemo',
    limit: 5,
  },
  {
    indexUid: 'movie_ratings',
    q: 'us',
  },
]})
```

```python
client.multi_search(
  [
    {'indexUid': 'movies', 'q': 'pooh', 'limit': 5},
    {'indexUid': 'movies', 'q': 'nemo', 'limit': 5},
    {'indexUid': 'movie_ratings', 'q': 'us'}
  ]
)
```

```php
$client->multiSearch([
    (new SearchQuery())
        ->setIndexUid('movies')
        ->setQuery('pooh')
        ->setLimit(5),
    (new SearchQuery())
        ->setIndexUid('movies')
        ->setQuery('nemo')
        ->setLimit(5),
    (new SearchQuery())
        ->setIndexUid('movie_ratings')
        ->setQuery('us')
  ]);
```

```java
MultiSearchRequest multiSearchRequest = new MultiSearchRequest();
multiIndexSearch.addQuery(new IndexSearchRequest("movies").setQuery("pooh").setLimit(5));
multiIndexSearch.addQuery(new IndexSearchRequest("movies").setQuery("nemo").setLimit(5));
multiIndexSearch.addQuery(new IndexSearchRequest("movie_ratings").setQuery("us"));

client.multiSearch(multiSearchRequest);
```

```ruby
client.multi_search([
  { index_uid: 'books', q: 'prince' },
  { index_uid: 'movies', q: 'pooh', limit: 5 }
  { index_uid: 'movies', q: 'nemo', limit: 5 }
  { index_uid: 'movie_ratings', q: 'us' }
])
```

```go
client.MultiSearch(&MultiSearchRequest{
  Queries: []SearchRequest{
    {
      IndexUID: "movies",
      Query:    "pooh",
      Limit:    5,
    },
    {
      IndexUID: "movies",
      Query:    "nemo",
      Limit:    5,
    },
    {
      IndexUID: "movie_ratings",
      Query:    "us",
    },
  },
})
```

```csharp
await client.MultiSearchAsync(new MultiSearchQuery()
{
    Queries = new System.Collections.Generic.List<SearchQuery>()
    {
        new SearchQuery() {
          IndexUid = "movies",
          Q = "booh",
          Limit = 5
        },
        new SearchQuery() {
          IndexUid = "movies",
          Q = "nemo",
          Limit = 5
        },
        new SearchQuery() {
          IndexUid = "movie_ratings",
          Q = "us",
        },
    }
});
```

```rust
let movie = client.index("movie");
let movie_ratings = client.index("movie_ratings");

let search_query_1 = SearchQuery::new(&movie)
    .with_query("pooh")
    .with_limit(5)
    .build();
let search_query_2 = SearchQuery::new(&movie)
    .with_query("nemo")
    .with_limit(5)
    .build();
let search_query_3 = SearchQuery::new(&movie_ratings)
    .with_query("us")
    .build();

let response = client
    .multi_search()
    .with_search_query(search_query_1)
    .with_search_query(search_query_2)
    .with_search_query(search_query_3)
    .execute::<Document>()
    .await
    .unwrap();
```

```dart
await client.multiSearch(MultiSearchQuery(queries: [
  IndexSearchQuery(query: 'pooh', indexUid: 'movies', limit: 5),
  IndexSearchQuery(query: 'nemo', indexUid: 'movies', limit: 5),
  IndexSearchQuery(query: 'us', indexUid: 'movies_ratings'),
]));
```

In this example, the request sends three queries: two targeting the `movies` index with different search terms and limits, and one targeting the `movie_ratings` index.

## Understand the response format

Meilisearch returns a `results` array with one entry per query, in the same order as the queries you sent:

```json
{
  "results": [
    {
      "indexUid": "movies",
      "hits": [
        { "id": 24, "title": "Winnie the Pooh" }
      ],
      "query": "pooh",
      "processingTimeMs": 0,
      "limit": 5,
      "offset": 0,
      "estimatedTotalHits": 2
    },
    {
      "indexUid": "movies",
      "hits": [
        { "id": 12, "title": "Finding Nemo" }
      ],
      "query": "nemo",
      "processingTimeMs": 0,
      "limit": 5,
      "offset": 0,
      "estimatedTotalHits": 1
    },
    {
      "indexUid": "movie_ratings",
      "hits": [
        { "id": 458723, "title": "Us", "director": "Jordan Peele" }
      ],
      "query": "us",
      "processingTimeMs": 0,
      "limit": 20,
      "offset": 0,
      "estimatedTotalHits": 1
    }
  ]
}
```

Each result set contains the same fields as a standard search response, including `hits`, `query`, `processingTimeMs`, and `estimatedTotalHits`.

## How queries work together

Each query in a multi-search request is fully independent. This means:

* **Different indexes**: each query can target a different index
* **Different parameters**: each query can have its own [`filter`](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/getting_started), [`sort`](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/sort_results), `limit`, `offset`, `attributesToRetrieve`, and other search parameters
* **Same index, different queries**: you can send multiple queries to the same index with different search terms or parameters
* **Single HTTP request**: all queries are bundled into one network call, reducing latency compared to sending individual requests

## When to use multi-index search

Multi-index search is best suited for interfaces that show results from different indexes in separate sections. For example, a search bar that displays matching products in one panel, blog posts in another, and user profiles in a third.

If you want to merge results from multiple indexes into a single ranked list instead, use [federated search](https://www.meilisearch.com/docs/capabilities/multi_search/getting_started/federated_search).

## Next steps

> [card] [Federated search](https://www.meilisearch.com/docs/capabilities/multi_search/getting_started/federated_search)
>
> Merge results from multiple indexes into one ranked list

> [card] [Multi-search overview](https://www.meilisearch.com/docs/capabilities/multi_search/overview)
>
> Learn about both modes of multi-search

> [card] [API reference](https://www.meilisearch.com/docs/reference/api/multi-search/perform-a-multi-search)
>
> Full endpoint documentation for multi-search

> [card] [Different filters per index](https://www.meilisearch.com/docs/capabilities/multi_search/how_to/search_with_different_filters)
>
> Apply different filters to each query


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.