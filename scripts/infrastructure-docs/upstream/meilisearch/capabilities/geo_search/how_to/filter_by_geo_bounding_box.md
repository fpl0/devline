# Filter by geo bounding box

> Filter search results within a rectangular geographic area defined by two corner points.

The `_geoBoundingBox` filter returns documents located within a rectangle defined by its top-right and bottom-left coordinates. This is especially useful for map-based interfaces where you want to display results that fit within the current viewport.

## Syntax

```
_geoBoundingBox([topRightLat, topRightLng], [bottomLeftLat, bottomLeftLng])
```

| Parameter | Type | Description |
| - | - | - |
| `topRightLat` | Float | Latitude of the top-right corner (northern boundary) |
| `topRightLng` | Float | Longitude of the top-right corner (eastern boundary) |
| `bottomLeftLat` | Float | Latitude of the bottom-left corner (southern boundary) |
| `bottomLeftLng` | Float | Longitude of the bottom-left corner (western boundary) |

The first coordinate pair defines the **top-right** (northeast) corner of the rectangle, and the second defines the **bottom-left** (southwest) corner. This means:

* `topRightLat` should be greater than `bottomLeftLat`
* `topRightLng` should be greater than `bottomLeftLng`

## Filter by bounding box

The following example searches for restaurants within a bounding box covering central Milan:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/restaurants/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-type:application/json' \
  --data-binary '{ "filter": "_geoBoundingBox([45.494181, 9.214024], [45.449484, 9.179175])" }'
```

```javascript
client.index('restaurants').search('', {
  filter: ['_geoBoundingBox([45.494181, 9.214024], [45.449484, 9.179175])'],
})
```

```python
client.index('restaurants').search('Batman', {
  'filter': '_geoBoundingBox([45.494181, 9.214024], [45.449484, 9.179175])'
})
```

```php
$client->index('restaurants')->search('', [
  'filter' => '_geoBoundingBox([45.494181, 9.214024], [45.449484, 9.179175])'
]);
```

```java
SearchRequest searchRequest = SearchRequest.builder().q()("").filter(new String[] {
    "_geoBoundingBox([45.494181, 9.214024], [45.449484, 9.179175])"
  }).build();
client.index("restaurants").search(searchRequest);
```

```ruby
client.index('restaurants').search('', { filter: ['_geoBoundingBox([45.494181, 9.214024], [45.449484, 9.179175])'] })
```

```go
client.Index("restaurants").Search("", &meilisearch.SearchRequest{
  Filter: "_geoBoundingBox([45.494181, 9.214024], [45.449484, 9.179175])",
})
```

```csharp
SearchQuery filters = new SearchQuery()
{
    Filter = "_geoBoundingBox([45.494181, 9.214024], [45.449484, 9.179175])"
};
var restaurants = await client.Index("restaurants").SearchAsync<Restaurant>("restaurants", filters);
```

```rust
let results: SearchResults<Restaurant> = client
  .index("restaurants")
  .search()
  .with_filter("_geoBoundingBox([45.494181, 9.214024], [45.449484, 9.179175])")
  .execute()
  .await
  .unwrap();
```

```swift
let searchParameters = SearchParameters(
    filter: "_geoBoundingBox([45.494181, 9.214024], [45.449484, 9.179175])"
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
        filter:
            '_geoBoundingBox([45.494181, 9.214024], [45.449484, 9.179175])',
      ),
    );
```

Meilisearch returns all documents with a `_geo` location inside the specified rectangle:

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
      "_geoDistance": 0
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
      "_geoDistance": 0
    }
  ]
}
```

> [!NOTE]
>
> When using `_geoBoundingBox` without `_geoRadius` or `_geoPoint` sorting, the `_geoDistance` field is `0` because there is no reference point to calculate distance from.

## Use with map-based UIs

Bounding box filters work well with interactive maps. When a user pans or zooms the map, read the visible bounds from your map library and pass them directly to Meilisearch.

For example, with a JavaScript map library:

```javascript
// Get the current map bounds
const bounds = map.getBounds();
const ne = bounds.getNorthEast();
const sw = bounds.getSouthWest();

// Search for results in the visible area
const results = await client.index('restaurants').search('', {
  filter: `_geoBoundingBox([${ne.lat}, ${ne.lng}], [${sw.lat}, ${sw.lng}])`
});
```

## Combine with other filters

You can combine `_geoBoundingBox` with any other filter using `AND` and `OR` operators:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/restaurants/search' \
  -H 'Content-type:application/json' \
  --data-binary '{
    "filter": "_geoBoundingBox([45.494181, 9.214024], [45.449484, 9.179175]) AND type = pizza"
  }'
```

> [card] [Geo search overview](https://www.meilisearch.com/docs/capabilities/geo_search/getting_started)
>
> Learn about all geo search capabilities in Meilisearch.

> [card] [Search API reference](https://www.meilisearch.com/docs/reference/api/search/search-with-post)
>
> Full reference for the search endpoint and filter parameter.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.