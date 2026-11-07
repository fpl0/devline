# Personalization and recommendations

> Personalize search results based on user profiles and recommend related items with the similar documents endpoint.

Meilisearch offers two ways to tailor results to individual users:

* **Personalized search** re-ranks search results at query time based on a user profile you provide, so each user sees the most relevant results for them.
* **Recommendations** use the [`/similar` endpoint](https://www.meilisearch.com/docs/capabilities/personalization/getting_started/recommendations) to find documents semantically close to a given item, powering features like "More like this" or "Related items".

Both features work alongside [full-text search](https://www.meilisearch.com/docs/capabilities/full_text_search/overview) and [hybrid search](https://www.meilisearch.com/docs/capabilities/hybrid_search/overview).

## Personalized search

Not everyone searches the same way. Personalized search lets you adapt relevance to each user's preferences, behavior, or intent.

For example, in an e-commerce site, someone who often shops for sportswear might see sneakers and activewear ranked higher when searching for "shoes". A user interested in luxury fashion might see designer heels or leather boots first instead.

### How it works

1. Generate a user profile: `"The user prefers genres like Documentary, Music, Drama"`
2. Submit the profile together with the search request
3. Meilisearch retrieves documents based on the query as usual
4. The re-ranking model reorders results based on the user profile, within the set of documents returned by the search

## Recommendations

Once you have configured an [embedder](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/choose_an_embedder), you can use the `/similar` endpoint to find documents that are semantically close to a reference document. This requires no additional configuration beyond the embedder itself.

Typical use cases include "Customers also viewed" on product pages, "Related articles" on content platforms, and "Because you watched X" on media streaming services.

## Use cases

* **E-commerce**: Surface products aligned with a shopper's purchase history, brand preferences, or browsing behavior. Recommend similar products on detail pages.
* **Content platforms**: Rank articles, videos, or podcasts based on the topics a user engages with most. Show related content alongside the current item. Combine with [analytics](https://www.meilisearch.com/docs/capabilities/analytics/overview) to measure impact.
* **Marketplace search**: Tailor listings to a buyer's location, budget range, or past interactions so the most relevant offers appear first.

## Next steps

> [card] [Personalized search](https://www.meilisearch.com/docs/capabilities/personalization/getting_started/personalized_search)
>
> Configure and perform your first personalized search

> [card] [Recommendations](https://www.meilisearch.com/docs/capabilities/personalization/getting_started/recommendations)
>
> Build a recommendation system with the similar documents endpoint

> [card] [Generate user context](https://www.meilisearch.com/docs/capabilities/personalization/how_to/generate_user_context)
>
> Build user profiles from behavior data

> [card] [Personalize e-commerce search](https://www.meilisearch.com/docs/capabilities/personalization/how_to/personalize_ecommerce_search)
>
> Step-by-step guide for personalizing product search results


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.