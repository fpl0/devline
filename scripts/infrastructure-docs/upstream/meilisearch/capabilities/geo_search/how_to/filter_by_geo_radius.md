# Filter by geo radius

> Filter search results to only include documents within a specified distance from a geographic point.

The `_geoRadius` filter returns documents located within a circular area defined by a center point and a radius. This is the most common geo filter, useful for "find nearby" features like store locators, restaurant finders, or service area lookups.

## Syntax

```
_geoRadius(lat, lng, distanceInMeters)
```

| Parameter | Type | Description |
| - | - | - |
| `lat` | Float | Latitude of the center point |
| `lng` | Float | Longitude of the center point |
| `distanceInMeters` | Integer | Radius of the search area in meters |

The distance is always expressed in **meters**. For example, use `2000` for a 2 km radius or `500` for 500 meters.

## Filter by radius

The following example searches for restaurants within 2 km of central Milan (latitude 45.472735, longitude 9.184019):

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/restaurants/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-type:application/json' \
  --data-binary '{ "filter": "_geoRadius(45.472735, 9.184019, 2000)" }'
```

```javascript
client.index('restaurants').search('', {
  filter: ['_geoRadius(45.472735, 9.184019, 2000)'],
})
```

```python
client.index('restaurants').search('', {
  'filter': '_geoRadius(45.472735, 9.184019, 2000)'
})
```

```php
$client->index('restaurants')->search('', [
  'filter' => '_geoRadius(45.472735, 9.184019, 2000)'
]);
```

```java
SearchRequest searchRequest = SearchRequest.builder().q("").filter(new String[] {"_geoRadius(45.472735, 9.184019, 2000)"}).build();
client.index("restaurants").search(searchRequest);
```

```ruby
client.index('restaurants').search('', { filter: '_geoRadius(45.472735, 9.184019, 2000)' })
```

```go
resp, err := client.Index("restaurants").Search("", &meilisearch.SearchRequest{
  Filter: "_geoRadius(45.472735, 9.184019, 2000)",
})
```

```csharp
SearchQuery filters = new SearchQuery() { Filter = "_geoRadius(45.472735, 9.184019, 2000)" };
var restaurants = await client.Index("restaurants").SearchAsync<Restaurant>("", filters);
```

```rust
let results: SearchResults<Restaurant> = client
  .index("restaurants")
  .search()
  .with_filter("_geoRadius(45.472735, 9.184019, 2000)")
  .execute()
  .await
  .unwrap();
```

```swift
let searchParameters = SearchParameters(
    filter: "_geoRadius(45.472735, 9.184019, 2000)"
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
        filterExpression: Meili.geoRadius(
          (lat: 45.472735, lng: 9.184019),
          2000,
        ),
      ),
    );
```

Meilisearch returns all documents with a `_geo` location inside the specified circle:

```json
{
  "hits": [
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
      "_geoDistance": 1532
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
      "_geoDistance": 1343
    }
  ]
}
```

### Understanding `_geoDistance`

When you use `_geoRadius`, Meilisearch automatically includes a `_geoDistance` field in each result. This value represents the distance in meters between the document's location and the center point of your radius filter.

> [!NOTE]
>
> `_geoDistance` is a computed field that only appears in search results. It is not stored in your documents and cannot be used as a filter.

## Combine with other filters

You can combine `_geoRadius` with any other filter using `AND` and `OR` operators. The following example finds only pizzerias within 2 km of central Milan:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/restaurants/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-type:application/json' \
  --data-binary '{ "filter": "_geoRadius(45.472735, 9.184019, 2000) AND type = pizza" }'
```

```javascript
client.index('restaurants').search('', {
  filter: ['_geoRadius(45.472735, 9.184019, 2000) AND type = pizza'],
})
```

```python
client.index('restaurants').search('', {
  'filter': '_geoRadius(45.472735, 9.184019, 2000) AND type = pizza'
})
```

```php
$client->index('restaurants')->search('', [
  'filter' => '_geoRadius(45.472735, 9.184019, 2000) AND type = pizza'
]);
```

```java
SearchRequest searchRequest = SearchRequest.builder().q("").filter(new String[] {"_geoRadius(45.472735, 9.184019, 2000) AND type = pizza"}).build();
client.index("restaurants").search(searchRequest);
```

```ruby
client.index('restaurants').search('', { filter: '_geoRadius(45.472735, 9.184019, 2000) AND type = pizza' })
```

```go
resp, err := client.Index("restaurants").Search("", &meilisearch.SearchRequest{
  Filter: "_geoRadius(45.472735, 9.184019, 2000) AND type = pizza",
})
```

```csharp
SearchQuery filters = new SearchQuery()
{
    Filter = new string[] { "_geoRadius(45.472735, 9.184019, 2000) AND type = pizza" }
};

var restaurants = await client.Index("restaurants").SearchAsync<Restaurant>("restaurants", filters);
```

```rust
let results: SearchResults<Restaurant> = client
  .index("restaurants")
  .search()
  .with_filter("_geoRadius(45.472735, 9.184019, 2000) AND type = pizza")
  .execute()
  .await
  .unwrap();
```

```swift
let searchParameters = SearchParameters(
    filter: "_geoRadius(45.472735, 9.184019, 2000) AND type = pizza"
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
        filterExpression: Meili.and([
          Meili.geoRadius(
            (lat: 45.472735, lng: 9.184019),
            2000,
          ),
          Meili.attr('type').eq('pizza'.toMeiliValue())
        ]),
      ),
    );
```

```json
{
  "hits": [
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
      "_geoDistance": 1532
    }
  ]
}
```

## Common radius values

| Use case | Radius |
| - | - |
| Walking distance | `1000` (1 km) |
| Short drive | `5000` (5 km) |
| City-wide | `15000` (15 km) |
| Regional | `50000` (50 km) |

> [card] [Geo search overview](https://www.meilisearch.com/docs/capabilities/geo_search/getting_started)
>
> Learn about all geo search capabilities in Meilisearch.

> [card] [Search API reference](https://www.meilisearch.com/docs/reference/api/search/search-with-post)
>
> Full reference for the search endpoint and filter parameter.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.