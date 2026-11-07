# Glossary

> Definitions of key Meilisearch concepts and search terminology including full-text search, semantic search, hybrid search, and more.

This glossary defines key terms used throughout the Meilisearch documentation, as well as common search and information retrieval concepts.

## Meilisearch concepts

### Index

A collection of documents with shared settings. An index is the equivalent of a table in a relational database. Each index has a unique identifier (`uid`) and its own configuration for ranking, filtering, and other settings. [Learn more about indexes](https://www.meilisearch.com/docs/resources/internals/indexes).

### Document

A JSON object stored in an index. Documents are the basic unit of data in Meilisearch. Each document contains fields (key-value pairs) and must have a unique primary key. [Learn more about documents](https://www.meilisearch.com/docs/resources/internals/documents).

### Primary key

A unique identifier for each document in an index. Meilisearch uses the primary key to distinguish between documents. If you add a document with an existing primary key, the existing document is replaced. [Learn more about primary keys](https://www.meilisearch.com/docs/resources/internals/primary_key).

### Field

A key-value pair within a document. For example, `"title": "The Great Gatsby"` is a field where `title` is the attribute name and `"The Great Gatsby"` is the value. [Learn more about documents](https://www.meilisearch.com/docs/resources/internals/documents).

### API key

A token used to authenticate requests to a Meilisearch instance. Meilisearch uses three types of keys: a **master key** (used to create other keys), a **default admin API key** (full API access), and a **default search API key** (search-only access). [Learn more about API keys](https://www.meilisearch.com/docs/resources/self_hosting/security/basic_security).

### Master key

A secret key set at launch that protects your Meilisearch instance. The master key is used to generate the default admin and search API keys. It should never be exposed to end users. [Learn more about security](https://www.meilisearch.com/docs/resources/self_hosting/security/basic_security).

### Tenant token

A short-lived token generated from an API key that enforces search rules for multi-tenant applications. Tenant tokens allow you to restrict search results per user without creating separate indexes. [Learn more about tenant tokens](https://www.meilisearch.com/docs/capabilities/security/overview).

### Task

An asynchronous operation returned by Meilisearch when processing write requests (adding documents, updating settings, etc.). Tasks have statuses like `enqueued`, `processing`, `succeeded`, or `failed`. [Learn more about tasks](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/monitor_tasks).

### Batch

A group of tasks that Meilisearch processes together in a single operation. Meilisearch automatically groups compatible enqueued tasks into batches to improve indexing throughput. For example, multiple document addition tasks targeting the same index may be merged into one batch. [Learn more about batches](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations#task-batches).

### Ranking rules

An ordered list of criteria Meilisearch uses to sort search results by relevance. Default ranking rules include `words`, `typo`, `proximity`, `attribute`, `sort`, and `exactness`. [Learn more about ranking rules](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/ranking_rules).

### Filterable attributes

Document fields configured to support filtering. Only fields explicitly listed as filterable can be used in `filter` search parameters. [Learn more about filtering](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/getting_started).

### Sortable attributes

Document fields configured to support custom sorting. Only fields explicitly listed as sortable can be used in `sort` search parameters. [Learn more about sorting](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/sort_results).

### Searchable attributes

Document fields that Meilisearch scans when performing a search query. By default, all fields are searchable. Restricting searchable attributes improves relevancy and performance. [Learn more about searchable attributes](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_displayed_attributes).

### Displayed attributes

Document fields returned in search results. By default, all fields are displayed. Restricting displayed attributes lets you hide internal fields from search responses. [Learn more about displayed attributes](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_displayed_attributes).

### Distinct attribute

A field used to deduplicate search results. When set, Meilisearch returns only one document per unique value of the distinct attribute. [Learn more about distinct attribute](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_distinct_attribute).

### Synonyms

Words or phrases configured to be treated as equivalent during search. For example, you can define `"phone"` and `"mobile"` as synonyms so a search for either term returns results containing both. [Learn more about synonyms](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/synonyms).

### Stop words

Common words (such as "the", "a", "is") excluded from search queries to improve relevancy and performance. [API reference](https://www.meilisearch.com/docs/reference/api/settings/get-stopwords).

### Typo tolerance

Meilisearch's built-in ability to return relevant results even when the search query contains typos. You can configure the number of allowed typos and disable typo tolerance for specific attributes or words. [Learn more about typo tolerance](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/typo_tolerance_settings).

### Pagination

The mechanism for retrieving search results in smaller chunks. Meilisearch supports **offset/limit pagination** (for navigating pages with numbered buttons) and **estimated total hits**, which provides approximate result counts. [Learn more about pagination](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/paginate_search_results).

### Facets

Categorized counts of document attribute values returned alongside search results. Facets enable UI elements like filtering sidebars showing how many results exist for each category. [Learn more about facets](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/filter_with_facets).

### Geo search

The ability to filter and sort search results based on geographic coordinates (`_geo` field). Geo search supports filtering by radius (`_geoRadius`) or bounding box (`_geoBoundingBox`), and sorting by distance using `_geoPoint`. [Learn more about geo search](https://www.meilisearch.com/docs/capabilities/geo_search/getting_started).

### Multi-search

A single API request that performs multiple search queries across one or more indexes. Multi-search reduces network overhead and latency compared to sending individual search requests. [Learn more about multi-search](https://www.meilisearch.com/docs/capabilities/multi_search/overview).

### Federated search

A type of multi-search where results from multiple indexes are merged and ranked together into a single list, rather than returned as separate result sets. [Learn more about federated search](https://www.meilisearch.com/docs/capabilities/multi_search/getting_started/federated_search).

### Dump

A serialized export of your entire Meilisearch instance (documents, settings, API keys, and tasks). Dumps are portable across Meilisearch versions and can be used for migrations. [Learn more about dumps](https://www.meilisearch.com/docs/resources/self_hosting/data_backup/dumps).

### Snapshot

A binary copy of your Meilisearch database at a specific point in time. Snapshots are faster to create and restore than dumps but are not portable across versions. [Learn more about snapshots](https://www.meilisearch.com/docs/resources/self_hosting/data_backup/snapshots).

### Embedder

A model or service that generates vector representations (embeddings) of documents and queries for AI-powered search. Meilisearch supports built-in embedders (OpenAI, HuggingFace, Ollama) and custom REST embedders. [Learn more about embedders](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/choose_an_embedder).

### Auto-embeddings

Meilisearch's ability to automatically generate vector embeddings from your documents without any external pipeline. When you configure an embedder, Meilisearch generates embeddings at indexing time and at query time, so you never need to manage vectors yourself. [Learn more about AI-powered search](https://www.meilisearch.com/docs/capabilities/hybrid_search/getting_started).

### Document template

A template that controls how document fields are combined into a single text string before being sent to an embedder for auto-embedding. Templates let you choose which fields matter for semantic search and how they are formatted. For example, a template like `"{{ doc.title }}: {{ doc.description }}"` tells Meilisearch to embed the title and description together. [Learn more about document templates](https://www.meilisearch.com/docs/capabilities/hybrid_search/advanced/document_template_best_practices).

## Search and information retrieval

### Full-text search

A search technique that scans the entire content of indexed document fields to find matches for a query. Meilisearch's full-text search supports typo tolerance, prefix matching, and ranking by relevance. [Learn more about relevancy](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/relevancy).

### Keyword search

A search approach that matches documents based on the exact or near-exact presence of query terms. Traditional keyword search relies on term frequency and doesn't understand meaning. Meilisearch's full-text search is a form of keyword search enhanced with typo tolerance and ranking rules.

### Semantic search

A search technique that understands the meaning and intent behind a query, not just the keywords. Semantic search uses vector embeddings to find documents that are conceptually similar to the query, even if they don't share the same words. [Learn more about AI-powered search](https://www.meilisearch.com/docs/capabilities/hybrid_search/getting_started).

### Hybrid search

A search approach that combines full-text (keyword) search with semantic (vector) search. Meilisearch merges results from both techniques to provide results that are both keyword-accurate and semantically relevant. [Learn more about hybrid search](https://www.meilisearch.com/docs/capabilities/hybrid_search/getting_started).

### Vector search

A search technique that represents documents and queries as high-dimensional vectors (embeddings) and finds matches based on mathematical similarity. Vector search powers semantic search in Meilisearch. [Learn more about AI-powered search](https://www.meilisearch.com/docs/capabilities/hybrid_search/getting_started).

### Embeddings

Numerical vector representations of text (or images) generated by machine learning models. Embeddings capture semantic meaning, allowing similar concepts to have similar vector representations. [Learn more about embedders](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/choose_an_embedder).

### Conversational search

An AI-powered search experience where users interact with search results through natural language conversations. Meilisearch provides tooling for building conversational search interfaces using LLMs. [Learn more about conversational search](https://www.meilisearch.com/docs/capabilities/agentic_search/overview).

### RAG (Retrieval-Augmented Generation)

A technique that combines search (retrieval) with AI text generation. Instead of relying solely on the AI model's training data, RAG first retrieves relevant documents from a search engine and uses them as context for generating responses. [Learn more about agentic search](https://www.meilisearch.com/docs/capabilities/agentic_search/overview).

### Relevancy

How well search results match the user's intent. Meilisearch determines relevancy through ranking rules that consider factors like number of matching words, typo count, word proximity, and attribute importance. [Learn more about relevancy](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/relevancy).

### Tokenization

The process of breaking text into individual units (tokens) for indexing. Meilisearch tokenizes text based on word boundaries, with special handling for languages like Chinese, Japanese, and Korean. [Learn more about tokenization](https://www.meilisearch.com/docs/capabilities/indexing/advanced/tokenization).

### Prefix search

A search behavior where Meilisearch matches documents containing words that start with the query terms. For example, searching for "hel" matches "hello" and "help". By default, only the last word in a query uses prefix matching. [Learn more about prefix search](https://www.meilisearch.com/docs/resources/internals/prefix).

### Faceted search

A search interface pattern where users can refine results using categorized filters (facets). For example, an e-commerce site might let users filter by price range, brand, and color alongside their search query. [Learn more about faceted search](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/filter_with_facets).

### Geosearch

The ability to filter or sort search results based on geographic location. Users can find results within a specific radius or bounding box of a given point. [Learn more about geosearch](https://www.meilisearch.com/docs/capabilities/geo_search/getting_started).

### Autocomplete

A search UI pattern that suggests results or query completions as the user types, typically powered by prefix search.

### Search-as-you-type

An instant search experience where results update with every keystroke. Meilisearch is optimized for this pattern, with response times under 50ms.

### Typo tolerance (search concept)

The ability of a search engine to return relevant results despite misspellings in the query. Meilisearch calculates the edit distance (number of character changes) between query terms and indexed words. [Learn more about typo tolerance](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/typo_tolerance_settings).

### Multi-tenancy

An architecture where a single Meilisearch instance serves multiple users or organizations, with each tenant seeing only their own data. Meilisearch supports multi-tenancy through tenant tokens and filtered search. [Learn more about multi-tenancy](https://www.meilisearch.com/docs/capabilities/security/overview).

### Analytics

Data collected about search behavior (queries, clicks, conversions) used to understand how users interact with search and improve results over time. Meilisearch Cloud provides built-in analytics dashboards. [Learn more about analytics](https://www.meilisearch.com/docs/capabilities/analytics/getting_started).

## Infrastructure and scaling

### Sharding

Splitting a large dataset across multiple Meilisearch instances, each holding a subset of documents. Queries are sent to all shards in parallel using multi-search, and results are merged. Sharding allows horizontal scaling beyond what a single instance can handle. [Learn more about sharding](https://www.meilisearch.com/docs/resources/self_hosting/deployment/overview).

### Replication

Running multiple copies of the same Meilisearch instance so that read queries can be distributed across replicas. Replication improves availability and read throughput. If one replica goes down, others continue serving requests. [API reference](https://www.meilisearch.com/docs/reference/api/management/get-network-topology).

### Federation

Combining search results from multiple Meilisearch instances or indexes into a single ranked list. Federation can work locally (multiple indexes on one instance) or remotely (across multiple instances on a network). [Learn more about federation](https://www.meilisearch.com/docs/capabilities/multi_search/getting_started/federated_search).

### High availability

A deployment configuration where your search infrastructure continues to operate even if individual instances fail. Achieved through replication, load balancing, and automatic failover.

### Horizontal scaling

Adding more Meilisearch instances to handle larger datasets or higher query volumes, as opposed to vertical scaling (upgrading a single machine). Meilisearch supports horizontal scaling through sharding and replication. [Learn more about sharding](https://www.meilisearch.com/docs/resources/self_hosting/deployment/overview).

### Binary quantization

A compression technique that reduces vector embeddings to 1-bit representations. This dramatically reduces storage and improves search performance for large AI-powered search datasets, at the cost of some semantic precision. [API reference](https://www.meilisearch.com/docs/reference/api/settings/update-embedders).

### Memory mapping

A storage technique where Meilisearch maps database files directly into virtual memory, allowing the operating system to manage caching efficiently. This lets Meilisearch handle datasets larger than available RAM. [Learn more about storage](https://www.meilisearch.com/docs/resources/internals/storage).

### DiskANN

A disk-based approximate nearest neighbor algorithm used by Meilisearch for vector search. DiskANN enables fast similarity search on large vector datasets without requiring all vectors to fit in RAM.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.