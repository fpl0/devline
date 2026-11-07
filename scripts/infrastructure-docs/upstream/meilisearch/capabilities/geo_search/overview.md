# Geo search

> Filter and sort search results by geographic location using coordinates, bounding boxes, and polygons.

Geo search allows you to [filter](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/getting_started) and [sort](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/sort_results) documents based on their geographic location. Use it to build store locators, delivery zone finders, local service directories, and any application where physical proximity matters.

## Supported geo formats

Meilisearch supports two ways to store geographic data:

* **`_geo` field**: a simple object with `lat` and `lng` properties for point locations
* **GeoJSON**: a standardized format for complex geometries including points, polygons, and multi-polygons

## Geo operations

| Operation | Description |
| - | - |
| `_geoRadius` | Filter results within a circular area |
| `_geoBoundingBox` | Filter results within a rectangular area |
| `_geoPolygon` | Filter results within a custom polygon shape |
| `_geoPoint` | Sort results by distance from a reference point |

## Next steps

> [card] [Getting started](https://www.meilisearch.com/docs/capabilities/geo_search/getting_started)
>
> Index documents with coordinates and run your first geo search

> [card] [Filter by radius](https://www.meilisearch.com/docs/capabilities/geo_search/how_to/filter_by_geo_radius)
>
> Find results within a distance from a point

> [card] [Sort by distance](https://www.meilisearch.com/docs/capabilities/geo_search/how_to/sort_by_geo_point)
>
> Rank results by proximity to a location

> [card] [GeoJSON format](https://www.meilisearch.com/docs/capabilities/geo_search/how_to/use_geojson_format)
>
> Index complex geometries with GeoJSON


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.