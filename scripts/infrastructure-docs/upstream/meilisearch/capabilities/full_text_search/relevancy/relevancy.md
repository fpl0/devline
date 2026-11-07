# Relevancy

> Relevancy refers to the accuracy of search results. If search results tend to be appropriate for a given query, then they can be considered relevant.

**Relevancy** refers to the accuracy and effectiveness of search results. If search results are almost always appropriate, then they can be considered relevant, and vice versa.

Meilisearch has a number of features for fine-tuning the relevancy of search results. The most important tool among them is **ranking rules**. There are two types of ranking rules: [built-in ranking rules](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/ranking_rules) and custom ranking rules.

## Behavior

Each index possesses a list of ranking rules stored as an array in the [settings object](https://www.meilisearch.com/docs/reference/api/settings/list-all-settings). This array is fully customizable, meaning you can delete existing rules, add new ones, and reorder them as needed.

Meilisearch uses a [bucket sort](https://en.wikipedia.org/wiki/Bucket_sort) algorithm to rank documents whenever a search query is made. The first ranking rule applies to all documents, while each subsequent rule is only applied to documents considered equal under the previous rule as a tiebreaker.

**The order in which ranking rules are applied matters.** The first rule in the array has the most impact, and the last rule has the least. Our default configuration meets most standard needs, but [you can change it](https://www.meilisearch.com/docs/reference/api/settings/update-rankingrules).

Deleting a rule means that Meilisearch will no longer sort results based on that rule. For example, **if you delete the [typo ranking rule](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/ranking_rules#2-typo), documents with typos will still be considered during search**, but they will no longer be sorted by increasing number of typos.

## How ranking works

Meilisearch uses a [bucket sort](https://en.wikipedia.org/wiki/Bucket_sort) pipeline to determine which documents best match a query. The engine applies ranking rules sequentially: the first rule sorts all matching documents into broad groups (or "buckets"), and each subsequent rule acts as a tiebreaker within those groups. Because earlier rules have the greatest impact on final ordering, the position of each rule in the array matters significantly.

By default, Meilisearch ships with built-in ranking rules that handle word matching, typo tolerance, proximity, attribute weight, exactness, and more. You can also insert custom ranking rules at any position in the pipeline to sort by numeric or date fields specific to your dataset (for example, sorting by a popularity score or a release date).

## Chunking large documents

Meilisearch is optimized for paragraph-sized chunks of text. Documents with very large text fields (multiple pages of content) may lead to reduced search relevancy because ranking rules like proximity and word position work best on shorter text.

If your dataset contains large documents, split them into smaller chunks (one per paragraph or section) before indexing. Each chunk becomes its own document with a shared identifier linking it back to the original. Use Meilisearch's [distinct attribute](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_distinct_attribute) to prevent duplicates in search results.

For example, a book with 50 paragraphs becomes 50 documents, each containing one paragraph plus the book's metadata (title, author). The distinct attribute ensures only the best-matching paragraph is returned per book.

## Explore relevancy features

> [card] [Ranking rules](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/ranking_rules)
>
> Understand the built-in ranking rules and how they determine result order

> [card] [Custom ranking rules](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/custom_ranking_rules)
>
> Add your own ranking rules based on numeric or date attributes

> [card] [Ranking score](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/ranking_score)
>
> Inspect the relevancy score assigned to each search result

> [card] [Attribute ranking order](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/attribute_ranking_order)
>
> Control which document attributes carry the most weight in ranking

> [card] [Typo tolerance](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/typo_tolerance_settings)
>
> Configure how Meilisearch handles spelling mistakes

> [card] [Synonyms](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/synonyms)
>
> Define equivalent terms so users find results regardless of wording


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.