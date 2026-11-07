# Welcome to Meilisearch

> Meilisearch indexes your content and makes it accessible to both humans and AI through search, conversational interfaces, and APIs.

Meilisearch **indexes your content and makes it accessible to both humans and AI**. It stores your documents and embeddings, then exposes them through fast full-text search, semantic search, and conversational interfaces, all from a single API.

> [card] [Start with Cloud](https://www.meilisearch.com/docs/getting_started/first_project)
>
> Get started in minutes with Meilisearch Cloud

> [card] [Self-host](https://www.meilisearch.com/docs/resources/self_hosting/getting_started/quick_start)
>
> Deploy on your own infrastructure

## How it works

**Index once, access everywhere.** You push your content to Meilisearch. It stores the documents, builds the search indexes, and when configured with an embedder, generates vector embeddings automatically. Your data becomes accessible to both end users and AI systems.

> [card] For humans
>
> Search-as-you-type interfaces, faceted navigation, filtering, sorting, and personalized results, all in under 50ms.

> [card] For AI
>
> Semantic search, RAG-powered conversational interfaces, and similar document retrieval so LLMs can answer questions grounded in your data.

## What you can build

* **Search interfaces**: instant, typo-tolerant search bars for websites, apps, and documentation.
* **AI assistants**: connect LLMs to your content with built-in RAG. Users ask questions in natural language and get answers grounded in your data.
* **Recommendation systems**: find similar documents and personalize results based on user preferences.
* **Internal tools**: make company knowledge searchable across documents, databases, and APIs.

## Why Meilisearch?

Meilisearch is built on three pillars:

### Performance

Meilisearch is designed for speed at scale. Every query returns results in under 50 milliseconds, whether your index contains a thousand documents or tens of millions. The engine uses memory-mapped storage, multi-threaded indexing, and DiskANN-based vector search to maintain consistent performance as your data grows. Sharding and replication let you scale horizontally without sacrificing latency.

### Relevancy

Getting the right results means combining multiple signals. Meilisearch chains seven default ranking rules (words, typo, proximity, attributeRank, sort, wordPosition, and exactness) with support for custom rules tailored to your domain. Hybrid search merges keyword and semantic results so users find what they're looking for even when they don't use the exact right words. Conversational search goes further: RAG-powered responses are grounded in your indexed data, so AI answers are sourced and verifiable.

### Developer experience

Meilisearch is a single binary with a REST API. There is no cluster to configure, no schema to define, and no separate vector store to manage. Send your documents and Meilisearch handles tokenization, indexing, and vector generation through auto-embeddings. SDKs for 10+ languages, one-click deployment on Meilisearch Cloud, and sensible defaults mean you go from zero to production search in minutes, not weeks.

## See it in action

[![Search bar updating results](https://mintcdn.com/meilisearch-6b28dec2/AA65w-9bZrf-CgFA/assets/images/movies-demo-dark.gif?s=baae8cf6f1dbe895a8bf66fbd4a57c54)](https://where2watch.meilisearch.com/?utm_campaign=oss\&utm_source=docs\&utm_medium=overview)

Try our live demos:

* [E-commerce search](https://ecommerce.meilisearch.com/) - Browse millions of products
* [Where to Watch](https://where2watch.meilisearch.com/) - Search the TMDB movie database
* [SaaS search](https://saas.meilisearch.com/) - Multi-model search with Laravel

## Get started with Meilisearch Cloud

[Meilisearch Cloud](https://www.meilisearch.com/cloud?utm_campaign=oss\&utm_source=docs\&utm_medium=overview) gets you up and running in minutes with automatic scaling, updates, and maintenance. Start with a [14-day free trial](https://cloud.meilisearch.com).

## Next steps

> [card] [Explore features](https://www.meilisearch.com/docs/getting_started/features)
>
> See all Meilisearch capabilities

> [card] [Good practices](https://www.meilisearch.com/docs/getting_started/good_practices)
>
> Learn how to format, chunk, and index your data

> [card] [Choose your SDK](https://www.meilisearch.com/docs/getting_started/sdks/javascript)
>
> Get started with your preferred language

> [card] [Glossary](https://www.meilisearch.com/docs/getting_started/glossary)
>
> Key terms and concepts explained


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.