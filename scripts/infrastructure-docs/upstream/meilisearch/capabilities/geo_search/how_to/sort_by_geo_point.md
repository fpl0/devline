# Sort by geo point

> Sort search results by distance from a geographic reference point to show the closest results first.

The `_geoPoint` sort rule orders results by their distance from a specified latitude and longitude. Use this to show users the nearest matching results first, or to push nearby results to the end of the list.

## Syntax

```
_geoPoint(lat, lng):asc
_geoPoint(lat, lng):desc
```

| Parameter | Type | Description |
| - | - | - |
| `lat` | Float | Latitude of the reference point |
| `lng` | Float | Longitude of the reference point |

Use `:asc` to show the closest results first, or `:desc` to show the farthest results first.

## Sort by proximity

The following example sorts restaurants by their distance from the Eiffel Tower (latitude 48.8561446, longitude 2.2978204), with the closest results first:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/restaurants/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-type:application/json' \
  --data-binary '{ "sort": ["_geoPoint(48.8561446,2.2978204):asc"] }'
```

```javascript
client.index('restaurants').search('', {
  sort: ['_geoPoint(48.8561446, 2.2978204):asc'],
})
```

```python
client.index('restaurants').search('', {
  'sort': ['_geoPoint(48.8561446,2.2978204):asc']
})
```

```php
$client->index('restaurants')->search('', [
  'sort' => ['_geoPoint(48.8561446,2.2978204):asc']
]);
```

```java
SearchRequest searchRequest = SearchRequest.builder().q("").sort(new String[] {"_geoPoint(48.8561446,2.2978204):asc"}).build();
client.index("restaurants").search(searchRequest);
```

```ruby
client.index('restaurants').search('', { sort: ['_geoPoint(48.8561446, 2.2978204):asc'] })
```

```go
resp, err := client.Index("restaurants").Search("", &meilisearch.SearchRequest{
  Sort: []string{
    "_geoPoint(48.8561446,2.2978204):asc",
  },
})
```

```csharp
SearchQuery filters = new SearchQuery()
{
    Sort = new string[] { "_geoPoint(48.8561446,2.2978204):asc" }
};

var restaurants = await client.Index("restaurants").SearchAsync<Restaurant>("", filters);
```

```rust
let results: SearchResults<Restaurant> = client
  .index("restaurants")
  .search()
  .with_sort(&["_geoPoint(48.8561446, 2.2978204):asc"])
  .execute()
  .await
  .unwrap();
```

```swift
let searchParameters = SearchParameters(
    query: "",
    sort: ["_geoPoint(48.8561446, 2.2978204):asc"]
)
client.index("restaurants").search(searchParameters) { (result: Result<Searchable<Restaurant>, Swift.Error>) in
    switch result {
    case .success(let task):
      print(task)
    case .failure(let error):
      print(error)
    }
  }
```

```dart
await client.index('restaurants').search(
    '', SearchQuery(sort: ['_geoPoint(48.8561446, 2.2978204):asc']));
```

```json
{
  "hits": [
    {
      "id": 2,
      "name": "Bouillon Pigalle",
      "address": "22 Bd de Clichy, 75018 Paris, France",
      "type": "french",
      "rating": 8,
      "_geo": {
        "lat": 48.8826517,
        "lng": 2.3352748
      },
      "_geoDistance": 4156
    },
    {
      "id": 3,
      "name": "Artico Gelateria Tradizionale",
      "address": "Via Dogana, 1, 20123 Milan, Italy",
      "type": "ice cream",
      "rating": 10,
      "_geo": {
        "lat": 45.4632046,
        "lng": 9.1719421
      },
      "_geoDistance": 640728
    },
    {
      "id": 1,
      "name": "Nàpiz' Milano",
      "address": "Viale Vittorio Veneto, 30, 20124, Milan, Italy",
      "type": "pizza",
      "rating": 9,
      "_geo": {
        "lat": 45.4777599,
        "lng": 9.1967508
      },
      "_geoDistance": 640207
    }
  ]
}
```

### Understanding `_geoDistance`

When you use `_geoPoint` for sorting, Meilisearch automatically includes a `_geoDistance` field in each result. This value represents the distance in meters between the document's location and the reference point you specified.

> [!NOTE]
>
> `_geoDistance` is a computed field that only appears in search results. It is not stored in your documents and cannot be used as a filter or sort rule.

## Combine with other sort rules

`_geoPoint` works alongside other sort rules. You can sort by proximity first, then break ties with another attribute. The following example sorts restaurants by distance from the Eiffel Tower, then by rating in descending order:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/restaurants/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-type:application/json' \
  --data-binary '{
    "sort": [
      "_geoPoint(48.8561446,2.2978204):asc",
      "rating:desc"
    ]
  }'
```

```javascript
client.index('restaurants').search('', {
  sort: ['_geoPoint(48.8561446, 2.2978204):asc', 'rating:desc'],
})
```

```python
client.index('restaurants').search('', {
  'sort': ['_geoPoint(48.8561446,2.2978204):asc', 'rating:desc']
})
```

```php
$client->index('restaurants')->search('', [
  'sort' => ['_geoPoint(48.8561446,2.2978204):asc', 'rating:desc']
]);
```

```java
SearchRequest searchRequest = SearchRequest.builder().q()("").sort(new String[] {
    "_geoPoint(48.8561446,2.2978204):asc",
    "rating:desc",
  }).build();
client.index("restaurants").search(searchRequest);
```

```ruby
client.index('restaurants').search('', { sort: ['_geoPoint(48.8561446, 2.2978204):asc', 'rating:desc'] })
```

```go
resp, err := client.Index("restaurants").Search("", &meilisearch.SearchRequest{
  Sort: []string{
    "_geoPoint(48.8561446,2.2978204):asc",
    "rating:desc",
  },
})
```

```csharp
SearchQuery filters = new SearchQuery()
{
    Sort = new string[] {
          "_geoPoint(48.8561446,2.2978204):asc",
          "rating:desc"
    }
};

var restaurants = await client.Index("restaurants").SearchAsync<Restaurant>("restaurants", filters);
```

```rust
let results: SearchResults<Restaurant> = client
  .index("restaurants")
  .search()
  .with_sort(&["_geoPoint(48.8561446, 2.2978204):asc", "rating:desc"])
  .execute()
  .await
  .unwrap();
```

```swift
let searchParameters = SearchParameters(
    query: "",
    sort: ["_geoPoint(48.8561446, 2.2978204):asc", "rating:desc"]
)
client.index("restaurants").search(searchParameters) { (result: Result<Searchable<Restaurant>, Swift.Error>) in
    switch result {
    case .success(let searchResult):
        print(searchResult)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('restaurants').search(
    '',
    SearchQuery(
        sort: ['_geoPoint(48.8561446, 2.2978204):asc', 'rating:desc']));
```

```json
{
  "hits": [
    {
      "id": 2,
      "name": "Bouillon Pigalle",
      "address": "22 Bd de Clichy, 75018 Paris, France",
      "type": "french",
      "rating": 8,
      "_geo": {
        "lat": 48.8826517,
        "lng": 2.3352748
      },
      "_geoDistance": 4156
    },
    {
      "id": 3,
      "name": "Artico Gelateria Tradizionale",
      "address": "Via Dogana, 1, 20123 Milan, Italy",
      "type": "ice cream",
      "rating": 10,
      "_geo": {
        "lat": 45.4632046,
        "lng": 9.1719421
      },
      "_geoDistance": 640728
    },
    {
      "id": 1,
      "name": "Nàpiz' Milano",
      "address": "Viale Vittorio Veneto, 30, 20124, Milan, Italy",
      "type": "pizza",
      "rating": 9,
      "_geo": {
        "lat": 45.4777599,
        "lng": 9.1967508
      },
      "_geoDistance": 640207
    }
  ]
}
```

## Combine with geo filters

You can use `_geoPoint` sorting together with geo filters to both limit results to a geographic area and order them by proximity. For example, find restaurants within 5 km of central Milan, sorted by distance:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/restaurants/search' \
  -H 'Content-type:application/json' \
  --data-binary '{
    "filter": "_geoRadius(45.472735, 9.184019, 5000)",
    "sort": ["_geoPoint(45.472735, 9.184019):asc"]
  }'
```

This is useful when you want to both restrict results to a specific area and present them in order from nearest to farthest.

> [card] [Geo search overview](https://www.meilisearch.com/docs/capabilities/geo_search/getting_started)
>
> Learn about all geo search capabilities in Meilisearch.

> [card] [Search API reference](https://www.meilisearch.com/docs/reference/api/search/search-with-post)
>
> Full reference for the search endpoint and sort parameter.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.