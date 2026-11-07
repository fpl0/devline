# Specific behaviors

> Learn how search rules interact with each other and with other Meilisearch features such as filters.

## Filters still apply

Pinned documents do not bypass filters. If the current search filters exclude a pinned document, Meilisearch drops it instead of forcing it into the response. This keeps pinned results consistent with the visibility, safety, and authorization rules your filters already enforce.

## Pinning wins over hiding

If a hide action and a pin action select the same document, Meilisearch pins the document instead of hiding it.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.