# Full-text search

> Meilisearch's full-text search returns relevant results in milliseconds with built-in typo tolerance, prefix matching, and multi-criteria ranking.

Full-text search is the core capability of Meilisearch. When a user types a query, Meilisearch scans [indexed](https://www.meilisearch.com/docs/capabilities/indexing/overview) documents and returns results ranked by relevance using a multi-criteria sorting algorithm.

## Key features

* **[Typo tolerance](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/typo_tolerance_settings)**: automatically handles misspellings using Levenshtein distance
* **[Prefix search](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_prefix_search)**: returns results as the user types, matching partial words
* **Multi-criteria ranking**: combines multiple [ranking rules](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/ranking_rules) (typo, proximity, attribute, exactness, and more) to determine result order
* **Customizable relevancy**: configure ranking rules, [searchable attributes](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_searchable_attributes), [stop words](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_stop_words), [synonyms](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/synonyms), and more to fine-tune results for your use case

## When to use full-text search

Full-text search works best when users search with keywords or short phrases and expect results ranked by textual relevance. If your users search with natural language questions or need results based on meaning rather than exact terms, consider [hybrid search](https://www.meilisearch.com/docs/capabilities/hybrid_search/overview) or [agentic search](https://www.meilisearch.com/docs/capabilities/agentic_search/overview).

## Choosing a search endpoint

Meilisearch exposes two search routes: `POST /indexes/{index_uid}/search` and `GET /indexes/{index_uid}/search`. **Prefer `POST` in nearly all cases.** `POST` accepts the full range of search parameters, including structured (array) `filter` expressions, and is the endpoint every official SDK uses.

> [!WARNING]
>
> Use of the `GET /search` route is discouraged unless you have a specific reason to prefer it (for example, to leverage HTTP caching at a proxy layer). The `GET` route only accepts string filter expressions, so array-shaped filters must be serialized to strings before being passed as query parameters.

## Next steps

> [card] [Getting started](https://www.meilisearch.com/docs/capabilities/full_text_search/getting_started/placeholder_search)
>
> Try your first search query

> [card] [Configure relevancy](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/relevancy)
>
> Learn how ranking works and how to customize it

> [card] [Hybrid search](https://www.meilisearch.com/docs/capabilities/hybrid_search/overview)
>
> Combine full-text with semantic search

> [card] [How-to guides](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_searchable_attributes)
>
> Configure search behavior for your use case


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.