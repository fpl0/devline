# Search with facets

> Faceted search interfaces provide users with a quick way to narrow down search results by selecting categories relevant to their query.

In Meilisearch, facets are a specialized type of filter. This guide shows you how to configure facets and use them when searching a database of books. It also gives you instruction on how to get facet value distributions and to search for specific facet values.

## Configure facet index settings

First, create a new index using this [books dataset](https://www.meilisearch.com/docs/assets/datasets/books.json). Documents in this dataset have the following fields:

```json
{
  "id": 5,
  "title": "Hard Times",
  "genres": ["Classics","Fiction", "Victorian", "Literature"],
  "publisher": "Penguin Classics",
  "language": "English",
  "author": "Charles Dickens",
  "description": "Hard Times is a novel of social […] ",
  "format": "Hardcover",
  "rating": 3
}
```

Next, add `genres`, `language`, and `rating` to the list of `filterableAttributes`:

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/books/settings/filterable-attributes' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    "genres", "rating", "language"
  ]'
```

```javascript
client.index('movie_ratings').updateFilterableAttributes(['genres', 'rating', 'language'])
```

```python
client.index('movie_ratings').update_filterable_attributes([
  'genres',
  'director',
  'language'
])
```

```php
$client->index('movie_ratings')->updateFilterableAttributes(['genres', 'rating', 'language']);
```

```java
client.index("movie_ratings").updateFilterableAttributesSettings(new String[] { "genres", "director", "language" });
```

```ruby
client.index('movie_ratings').update_filterable_attributes(['genres', 'rating', 'language'])
```

```go
filterableAttributes := []interface{}{
  "genres",
  "rating",
  "language",
}
client.Index("movie_ratings").UpdateFilterableAttributes(&filterableAttributes)
```

```csharp
List<string> attributes = new() { "genres", "rating", "language" };
TaskInfo result = await client.Index("movie_ratings").UpdateFilterableAttributesAsync(attributes);
```

```rust
let task: TaskInfo = client
  .index("movie_ratings")
  .set_filterable_attributes(&["genres", "rating", "language"])
  .await
  .unwrap();
```

```dart
await client
    .index('movie_ratings')
    .updateFilterableAttributes(['genres', 'rating', 'language']);
```

You have now configured your index to use these attributes as filters.

## Use facets in a search query

Make a search query setting the `facets` search parameter:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/books/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "classic",
    "facets": [
    "genres", "rating", "language"
  ]
}'
```

```javascript
client.index('books').search('classic', { facets: ['genres', 'rating', 'language'] })
```

```python
client.index('books').search('classic', {
  'facets': ['genres', 'rating', 'language']
})
```

```php
$client->index('books')->search('classic', [
  'facets' => ['genres', 'rating', 'language']
]);
```

```java
SearchRequest searchRequest = SearchRequest.builder().q("classic").facets(new String[]
{
  "genres",
  "rating",
  "language"
}).build();
client.index("books").search(searchRequest);
```

```ruby
client.index('books').search('classic', {
  facets: ['genres', 'rating', 'language']
})
```

```go
resp, err := client.Index("books").Search("classic", &meilisearch.SearchRequest{
  Facets: []string{
    "genres",
    "rating",
    "language",
  },
})
```

```csharp
var sq = new SearchQuery
{
    Facets = new string[] { "genres", "rating", "language" }
};
await client.Index("books").SearchAsync<Book>("classic", sq);
```

```rust
let books = client.index("books");

let results: SearchResults<Book> = SearchQuery::new(&books)
  .with_query("classic")
  .with_facets(Selectors::Some(&["genres", "rating", "language"]))
  .execute()
  .await
  .unwrap();
```

```dart
await client
    .index('books')
    .search('', SearchQuery(facets: ['genres', 'rating', 'language']));
```

The response returns all books matching the query. It also returns two fields you can use to create a faceted search interface, `facetDistribution` and `facetStats`:

```json
{
  "hits": [
    …
  ],
  …
  "facetDistribution": {
    "genres": {
      "Classics": 6,
      …
    },
    "language": {
      "English": 6,
      "French": 1,
      "Spanish": 1
    },
    "rating": {
      "2.5": 1,
      …
    }
  },
  "facetStats": {
    "rating": {
      "min": 2.5,
      "max": 4.7
    }
  }
}
```

`facetDistribution` lists all facets present in your search results, along with the number of documents returned for each facet.

`facetStats` contains the highest and lowest values for all facets containing numeric values.

### Sorting facet values

By default, all facet values are sorted in ascending alphanumeric order. You can change this using the `sortFacetValuesBy` property of the [`faceting` index settings](https://www.meilisearch.com/docs/reference/api/settings/get-faceting):

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/books/settings/faceting' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "sortFacetValuesBy": {
      "genres": "count"
  }
}'
```

```javascript
client.index('books').updateFaceting({
  sortFacetValuesBy: {
    genres: 'count'
  }
})
```

```python
client.index('books').update_faceting_settings({ 'sortFacetValuesBy': { 'genres': 'count' } })
```

```php
$client->index('books')->updateFaceting(['sortFacetValuesBy' => ['genres' => 'count']]);
```

```java
Faceting newFaceting = new Faceting();
HashMap<String, FacetSortValue> facetSortValues = new HashMap<>();
facetSortValues.put("genres", FacetSortValue.COUNT);
newFaceting.setSortFacetValuesBy(facetSortValues);
client.index("books").updateFacetingSettings(newFaceting);
```

```ruby
client.index('books').update_faceting(
  sort_facet_values_by: {
    genres: 'count'
  }
)
```

```go
client.Index("books").UpdateFaceting(&meilisearch.Faceting{
    SortFacetValuesBy: {
       "genres": SortFacetTypeCount,
    }
})
```

```csharp
var newFaceting = new Faceting
{
    SortFacetValuesBy = new Dictionary<string, SortFacetValuesByType>
    {
        ["genres"] = SortFacetValuesByType.Count
    }
};
await client.Index("books").UpdateFacetingAsync(newFaceting);
```

```rust
let mut facet_sort_setting = BTreeMap::new();
facet_sort_setting.insert("genres".to_string(), FacetSortValue::Count);
let faceting = FacetingSettings {
  max_values_per_facet: 100,
  sort_facet_values_by: Some(facet_sort_setting),
};

let res = client.index("books")
  .set_faceting(&faceting)
  .await
  .unwrap();
```

```dart
await client.index('books').updateFaceting(
      Faceting(
        sortFacetValuesBy: {
          'genres': FacetingSortTypes.count,
        },
      ),
    );
```

The above code sample sorts the `genres` facet by descending value count.

Repeating the previous query using the new settings will result in a different order in `facetsDistribution`:

```json
{
  …
  "facetDistribution": {
    "genres": {
      "Fiction": 8,
      "Literature": 7,
      "Classics": 6,
      "Novel": 2,
      "Horror": 2,
      "Fantasy": 2,
      "Victorian": 2,
      "Vampires": 1,
      "Tragedy": 1,
      "Satire": 1,
      "Romance": 1,
      "Historical Fiction": 1,
      "Coming-of-Age": 1,
      "Comedy": 1
    },
   …
   }
}
```

## Searching facet values

You can also search for facet values with the [facet search endpoint](https://www.meilisearch.com/docs/reference/api/facet-search/search-for-facet-values):

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

The following code sample searches the `genres` facet for values starting with `c`:

The response contains a `facetHits` array listing all matching facets, together with the total number of documents that include that facet:

```json
{
  …
  "facetHits": [
    {
      "value": "Children's Literature",
      "count": 1
    },
    {
      "value": "Classics",
      "count": 6
    },
    {
      "value": "Comedy",
      "count": 2
    },
    {
      "value": "Coming-of-Age",
      "count": 1
    }
  ],
  "facetQuery": "c",
  …
}
```

You can further refine results using the `q`, `filter`, and `matchingStrategy` parameters. [Learn more about them in the API reference.](https://www.meilisearch.com/docs/reference/api/facet-search/search-for-facet-values)

## Toggle facet search globally

By default, the facet search endpoint is enabled for all indexes. If you do not need facet search and want to speed up indexing, you can disable it with the `facetSearch` index setting:

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/books/settings/facet-search' \
  -H 'Content-Type: application/json' \
  --data-binary 'false'
```

Setting `facetSearch` to `false` disables the `/indexes/{index_uid}/facet-search` endpoint for this index. Documents are still indexed for regular facet distribution, but Meilisearch skips the additional processing needed for facet search, resulting in faster indexing. To re-enable facet search, send the same request with `true`.

## Get exact facet counts

By default, the facet counts returned by the facet search endpoint are estimates. This is faster but may not be perfectly accurate for large datasets.

To get exact facet counts, set the `exhaustiveFacetCount` parameter to `true` in your facet search request:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/books/facet-search' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "facetName": "genres",
    "facetQuery": "c",
    "exhaustiveFacetCount": true
  }'
```

Exact counts are slower to compute, especially on large indexes. Use this option when precision matters more than speed, for example when displaying category counts in a storefront.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.