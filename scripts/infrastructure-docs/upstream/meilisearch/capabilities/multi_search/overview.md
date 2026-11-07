# Multi-search

> Query multiple indexes in a single API request, with options to receive separate result lists or a single merged (federated) result set.

Multi-search lets you query multiple indexes in one HTTP request. This is faster and more efficient than sending separate requests for each index.

## Two modes of multi-search

| Mode | Description | Use case |
| - | - | - |
| **Multi-index search** | Returns a separate result list for each queried index | Search bar with categorized sections (products, articles, users) |
| **Federated search** | Merges results from all indexes into one ranked list | Unified search across content types |

## How multi-search works

Send an array of search queries to the `/multi-search` endpoint. Each query can target a different index with its own [filters](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/getting_started), [sorting](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/sort_results), and parameters.

In federated mode, Meilisearch merges and re-ranks results from all indexes using configurable weights, giving you control over which index's results appear higher.

## Error handling

> [!WARNING]
>
> Multi-search requests fail fast. If Meilisearch encounters an error when handling any of the queries in a multi-search request, it immediately stops processing the request and returns an error message. The returned message only addresses the first error encountered, so earlier queries may succeed but no results are returned until every query is valid.

## Next steps

> [card] [Multi-index search](https://www.meilisearch.com/docs/capabilities/multi_search/getting_started/multi_search)
>
> Query multiple indexes with separate result lists

> [card] [Federated search](https://www.meilisearch.com/docs/capabilities/multi_search/getting_started/federated_search)
>
> Merge results from multiple indexes into one list

> [card] [Boost results](https://www.meilisearch.com/docs/capabilities/multi_search/how_to/boost_results_across_indexes)
>
> Use federation weights to prioritize one index over another

> [card] [Unified search bar](https://www.meilisearch.com/docs/capabilities/multi_search/how_to/build_unified_search_bar)
>
> Build a single search bar across content types

> [card] [Semantic multi-search](https://www.meilisearch.com/docs/capabilities/multi_search/how_to/combine_text_and_image_search)
>
> Combine text and image semantic search across indexes


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.