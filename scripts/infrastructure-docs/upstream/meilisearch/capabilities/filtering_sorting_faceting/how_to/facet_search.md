# Search within facet values

> Use the facet search endpoint to type-ahead through facet values, build auto-complete experiences, and narrow large filter lists. This page also covers the two key limitations of facet search.

Facet search is a dedicated endpoint for searching through the values of a single facet. It is typically used to power auto-complete and type-ahead interfaces on top of filter menus, especially when a facet has too many distinct values to display at once (for example, thousands of brands or authors).

If you are new to facets, start with [Search with facets](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/filter_with_facets) and [Build faceted navigation](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/build_faceted_navigation). This page focuses on the `/facet-search` endpoint itself and the edge cases you need to know about before relying on it.

## Prerequisites

Before you can search a facet's values, the attribute must be declared in [`filterableAttributes`](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/getting_started). Facet search is enabled by default on every index; you can disable it per index with the `facetSearch` setting if you do not need it and want to speed up indexing.

## Search a facet

Send a `POST` request to `/indexes/{index_uid}/facet-search` with the name of the facet you want to search and the partial string typed by the user:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/books/facet-search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "facetQuery": "c",
    "facetName": "genres"
}'
```

```javascript
client.index('books').searchForFacetValues({
  facetQuery: 'c',
  facetName: 'genres'
})
```

```python
client.index('books').facet_search('genres', 'c')
```

```php
$client->index('books')->facetSearch(
  (new FacetSearchQuery())
      ->setFacetQuery('c')
      ->setFacetName('genres')
);
```

```java
FacetSearchRequest fsr = FacetSearchRequest.builder().facetName("genres").facetQuery("c").build();
client.index("books").facetSearch(fsr);
```

```ruby
client.index('books').facet_search('genres', 'c')
```

```go
client.Index("books").FacetSearch(&meilisearch.FacetSearchRequest{
  FacetQuery: "c",
  FacetName: "genres",
  ExhaustiveFacetCount: true
})
```

```csharp
var query = new SearchFacetsQuery()
{
  FacetQuery = "c",
  ExhaustiveFacetCount: true
};
await client.Index("books").FacetSearchAsync("genres", query);
```

```rust
let res = client.index("books")
  .facet_search("genres")
  .with_facet_query("c")
  .execute()
  .await
  .unwrap();
```

```dart
await client.index('books').facetSearch(
      FacetSearchQuery(
        facetQuery: 'c',
        facetName: 'genres',
      ),
    );
```

The response contains a `facetHits` array with the matching values and the number of documents that carry each one:

```json
{
  "facetHits": [
    { "value": "Children's Literature", "count": 1 },
    { "value": "Classics", "count": 6 },
    { "value": "Comedy", "count": 2 },
    { "value": "Coming-of-Age", "count": 1 }
  ],
  "facetQuery": "c"
}
```

You can further scope the results by combining `facetQuery` with `q`, `filter`, and `matchingStrategy`. See the [facet search API reference](https://www.meilisearch.com/docs/reference/api/facet-search/search-for-facet-values) for the full list of parameters.

## Facet search only works on string fields

> [!WARNING]
>
> Meilisearch does not support facet search on numeric fields. If you need to type-ahead through a numeric facet, convert the values to strings before indexing them (for example, store `"rating": "4.5"` rather than `"rating": 4.5`).

Internally, Meilisearch stores numbers as `float64`. Floating-point values lack exact decimal precision and can be represented in more than one way, which makes prefix matching on their textual form unreliable. Only string facet values are indexed for facet search.

## Facet search only matches the first term of `facetQuery`

> [!WARNING]
>
> Meilisearch's facet search is single-term: only the first word in `facetQuery` is used to match facet values. Subsequent words are ignored.

For example, given a `author` facet that contains `"Jane Austen"`:

* `facetQuery: "Jane"` returns `Jane Austen`
* `facetQuery: "Austen"` does **not** return `Jane Austen`, because the word `Austen` is not the start of the stored value
* `facetQuery: "Jane Aus"` effectively behaves like `facetQuery: "Jane"` (the second word is dropped)

This is a deliberate trade-off that keeps the facet-search index compact and fast. If you need multi-word matching across facet values, perform a regular search with the `q` parameter instead and inspect the `facetDistribution` of the response.

## Get exact facet counts

By default, the counts returned by facet search are estimates, which is faster on large indexes. To force exact counts, set `exhaustiveFacetCount` to `true`:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/books/facet-search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "facetName": "genres",
    "facetQuery": "c",
    "exhaustiveFacetCount": true
  }'
```

Exact counts are slower to compute. Prefer them when accuracy matters (for example, a storefront's category counts) and the defaults when latency matters most.

## Next steps

> [card] [Search with facets](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/filter_with_facets)
>
> Configure facets and filter search results

> [card] [Build faceted navigation](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/build_faceted_navigation)
>
> Patterns for category and filter menus

> [card] [Handle large facet cardinality](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/handle_large_facet_cardinality)
>
> Strategies for facets with thousands of values

> [card] [Facet search API reference](https://www.meilisearch.com/docs/reference/api/facet-search/search-for-facet-values)
>
> Full list of parameters accepted by `/facet-search`


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.