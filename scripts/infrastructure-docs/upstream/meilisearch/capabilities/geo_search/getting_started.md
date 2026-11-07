# Geosearch

> Filter and sort search results based on their geographic location.

This guide walks you through indexing documents with geographic coordinates, then filtering and sorting results by location.

## Add `_geo` to your documents

Documents must contain a `_geo` field with `lat` and `lng` values:

```json
[
  {
    "id": 1,
    "name": "Nàpiz' Milano",
    "address": "Viale Vittorio Veneto, 30, 20124, Milan, Italy",
    "type": "pizza",
    "rating": 9,
    "_geo": {
      "lat": 45.4777599,
      "lng": 9.1967508
    }
  },
  {
    "id": 2,
    "name": "Bouillon Pigalle",
    "address": "22 Bd de Clichy, 75018 Paris, France",
    "type": "french",
    "rating": 8,
    "_geo": {
      "lat": 48.8826517,
      "lng": 2.3352748
    }
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
    }
  }
]
```

> [!WARNING]
>
> Trying to index a dataset with one or more documents containing badly formatted `_geo` values will cause Meilisearch to throw an [`invalid_document_geo_field`](https://www.meilisearch.com/docs/reference/errors/error_codes#invalid_document_geo_field) error. In this case, the update will fail and no documents will be added or modified.

> [!NOTE]
>
> Meilisearch also supports [GeoJSON](https://www.meilisearch.com/docs/capabilities/geo_search/how_to/use_geojson_format) for complex geometries like polygons and multi-polygons.

## Configure filterable and sortable attributes

To filter results by location, add `_geo` to `filterableAttributes`:

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/restaurants/settings/filterable-attributes' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-type:application/json' \
  --data-binary '["_geo"]'
```

```javascript
client.index('restaurants')
.updateFilterableAttributes([
  '_geo'
])
```

```python
client.index('restaurants').update_filterable_attributes([
  '_geo'
])
```

```php
$client->index('restaurants')->updateFilterableAttributes([
  '_geo'
]);
```

```java
Settings settings = new Settings();
settings.setFilterableAttributes(new String[] { "_geo" });
client.index("restaurants").updateSettings(settings);
```

```ruby
client.index('restaurants').update_filterable_attributes(['_geo'])
```

```go
filterableAttributes := []interface{}{
  "_geo",
}
client.Index("restaurants").UpdateFilterableAttributes(&filterableAttributes)
```

```csharp
List<string> attributes = new() { "_geo" };
TaskInfo result = await client.Index("movies").UpdateFilterableAttributesAsync(attributes);
```

```rust
let task: TaskInfo = client
  .index("restaurants")
  .set_filterable_attributes(&["_geo"])
  .await
  .unwrap();
```

```swift
client.index("restaurants").updateFilterableAttributes(["_geo"]) { (result) in
    switch result {
    case .success(let task):
        print(task)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('restaurants').updateFilterableAttributes(['_geo']);
```

To sort results by distance, add `_geo` to `sortableAttributes`:

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/restaurants/settings/sortable-attributes' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-type:application/json' \
  --data-binary '["_geo"]'
```

```javascript
client.index('restaurants').updateSortableAttributes([
  '_geo'
])
```

```python
client.index('restaurants').update_sortable_attributes([
  '_geo'
])
```

```php
$client->index('restaurants')->updateSortableAttributes([
  '_geo'
]);
```

```java
client.index("restaurants").updateSortableAttributesSettings(new String[] {"_geo"});
```

```ruby
client.index('restaurants').update_sortable_attributes(['_geo'])
```

```go
sortableAttributes := []string{
  "_geo",
}
client.Index("restaurants").UpdateSortableAttributes(&sortableAttributes)
```

```csharp
List<string> attributes = new() { "_geo" };
TaskInfo result = await client.Index("restaurants").UpdateSortableAttributesAsync(attributes);
```

```rust
let task: TaskInfo = client
  .index("restaurants")
  .set_sortable_attributes(&["_geo"])
  .await
  .unwrap();
```

```swift
client.index("restaurants").updateSortableAttributes(["_geo"]) { (result) in
  switch result {
  case .success(let task):
    print(task)
  case .failure(let error):
    print(error)
  }
}
```

```dart
await client.index('restaurants').updateSortableAttributes(['_geo']);
```

Meilisearch will rebuild your index whenever you update these settings. Depending on the size of your dataset, this might take a considerable amount of time.

## Filter results by location

Use the [`filter` search parameter](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-filter) with `_geoRadius` to find results within a given distance from a point. The following example searches for restaurants within 2km of central Milan:

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

```json
[
  {
    "id": 1,
    "name": "Nàpiz' Milano",
    "address": "Viale Vittorio Veneto, 30, 20124, Milan, Italy",
    "type": "pizza",
    "rating": 9,
    "_geo": {
      "lat": 45.4777599,
      "lng": 9.1967508
    }
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
    }
  }
]
```

## Sort results by distance

Use `_geoPoint` in the [`sort` search parameter](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-sort) to order results by proximity. The following example sorts restaurants by distance from the Eiffel Tower:

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
[
  {
    "id": 2,
    "name": "Bouillon Pigalle",
    "address": "22 Bd de Clichy, 75018 Paris, France",
    "type": "french",
    "rating": 8,
    "_geo": {
      "lat": 48.8826517,
      "lng": 2.3352748
    }
  },
  …
]
```

## Next steps

> [card] [Filter by radius](https://www.meilisearch.com/docs/capabilities/geo_search/how_to/filter_by_geo_radius)
>
> Find results within a circular area around a point

> [card] [Filter by bounding box](https://www.meilisearch.com/docs/capabilities/geo_search/how_to/filter_by_geo_bounding_box)
>
> Find results within a rectangular area

> [card] [Filter by polygon](https://www.meilisearch.com/docs/capabilities/geo_search/how_to/filter_by_geo_polygon)
>
> Find results within a custom polygon shape

> [card] [Sort by distance](https://www.meilisearch.com/docs/capabilities/geo_search/how_to/sort_by_geo_point)
>
> Rank results by proximity to a location

> [card] [Use GeoJSON format](https://www.meilisearch.com/docs/capabilities/geo_search/how_to/use_geojson_format)
>
> Index complex geometries with the GeoJSON standard


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.