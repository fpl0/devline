# Features

> Explore all Meilisearch capabilities - index your content and make it accessible to humans and AI through search, conversational interfaces, and APIs.

Meilisearch indexes your content and makes it accessible to both humans and AI. Here's everything it can do.

## Search capabilities

### Full-text search

Lightning-fast keyword search with typo tolerance and customizable relevancy.

| Feature | Description |
| - | - |
| Sub-50ms responses | Fast search regardless of dataset size |
| [Typo tolerance](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/typo_tolerance_settings) | Get relevant results even with spelling mistakes |
| [Ranking rules](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/ranking_rules) | 6 default rules plus custom ranking |
| [Ranking score](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/ranking_score) | Relevancy scores with optional detailed breakdown |
| [Synonyms](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/synonyms) | Define equivalent terms for better recall |
| [Stop words](https://www.meilisearch.com/docs/reference/api/settings/get-stopwords) | Ignore common words like "the" or "and" |
| [Distinct attribute](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_distinct_attribute) | Deduplicate results by a specific field |
| [Prefix search](https://www.meilisearch.com/docs/resources/internals/prefix) | Results update as users type |
| [Matching strategy](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-matching-strategy) | Control how query terms are matched: `last`, `all`, or `frequency` |
| [Phrase search](https://www.meilisearch.com/docs/reference/api/search/search-with-post) | Use double quotes to search for an exact phrase |
| [Negative search](https://www.meilisearch.com/docs/reference/api/search/search-with-post) | Exclude terms from results using the minus operator |
| [Placeholder search](https://www.meilisearch.com/docs/reference/api/search/search-with-post) | Return all results when the query is empty |

### Querying and result formatting

| Feature | Description |
| - | - |
| [Highlighting](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-highlight-pre-tag) | Emphasize query matches in results with customizable tags |
| [Cropping](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-attributes-to-crop) | Return only the relevant portion of long text fields |
| [Matches position](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-show-matches-position) | Get byte positions and lengths of matched terms |
| [Search cutoff](https://www.meilisearch.com/docs/reference/api/settings/get-searchcutoffms) | Set a maximum time limit for search queries |
| [Tokenization](https://www.meilisearch.com/docs/capabilities/indexing/advanced/tokenization) | Customize how queries are broken into tokens, with custom separators and dictionaries |

### AI-powered search

Semantic and hybrid search using vector embeddings for meaning-based results.

| Feature | Description |
| - | - |
| [Hybrid search](https://www.meilisearch.com/docs/capabilities/hybrid_search/getting_started) | Combine keyword and semantic search |
| [Embedders](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/choose_an_embedder) | OpenAI, Hugging Face, Cohere, Mistral, Voyage, Gemini, Cloudflare, Ollama, and custom REST |
| [Similar documents](https://www.meilisearch.com/docs/capabilities/personalization/getting_started/recommendations) | Find related content automatically |
| [Image search](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/image_search_with_multimodal) | Search images with multimodal embeddings |
| [Multi-embedder](https://www.meilisearch.com/docs/reference/api/settings/get-embedders) | Multiple embedding models on the same document |
| [User-provided vectors](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/search_with_user_provided_embeddings) | Bring your own pre-generated embeddings |

### Agentic search

Let AI agents and chat interfaces use your search data, with LLM integration.

| Feature | Description |
| - | - |
| [AI SDK](https://www.meilisearch.com/docs/getting_started/integrations/ai_sdk) | Give agents a Meilisearch search tool with the Vercel AI SDK |
| [Chats API](https://www.meilisearch.com/docs/capabilities/agentic_search/advanced/chats_api) | Engine-managed conversational search via the direct `/chats` API |
| [LLM providers](https://www.meilisearch.com/docs/capabilities/agentic_search/advanced/chats_api#workspace-settings) | OpenAI, Azure OpenAI, Mistral, and vLLM |
| [Streaming responses](https://www.meilisearch.com/docs/reference/api/chats/request-a-chat-completion) | Stream chat responses in real time |
| [Chat workspaces](https://www.meilisearch.com/docs/reference/api/chats/get-settings-of-a-chat-workspace) | Configure chat settings per index |

### Personalization

| Feature | Description |
| - | - |
| [User preference ranking](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-personalize-one-of-1) | Re-rank results based on individual user preferences using AI embeddings |
| Real-time adaptation | Adjust results as user preferences evolve |

### Filtering and faceting

Refine search results with powerful filters and build faceted navigation.

| Feature | Description |
| - | - |
| [Filters](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/getting_started) | Filter by any attribute with complex expressions (AND/OR) |
| [Facets](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/filter_with_facets) | Build faceted navigation interfaces |
| [Facet types](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/overview) | AND/OR operators, numeric, boolean, and date facets |
| [Sorting](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/sort_results) | Sort results by any field |
| [Geo search](https://www.meilisearch.com/docs/capabilities/geo_search/getting_started) | Geo radius, bounding box, geo sorting, and distance calculation |

### Multi-search and federation

Query multiple indexes in a single request for complex search scenarios.

| Feature | Description |
| - | - |
| [Multi-search](https://www.meilisearch.com/docs/capabilities/multi_search/overview) | Query multiple indexes at once |
| [Federated search](https://www.meilisearch.com/docs/capabilities/multi_search/getting_started/federated_search) | Merge results from multiple sources with configurable weights |

## Scaling

Scale Meilisearch horizontally across multiple instances or optimize resource usage on a single node.

| Feature | Description |
| - | - |
| [Sharding](https://www.meilisearch.com/docs/resources/self_hosting/deployment/overview) | Distribute documents across multiple instances for horizontal scaling |
| [Replication](https://www.meilisearch.com/docs/reference/api/management/get-network-topology) | Replicate data across multiple instances for high availability |
| [Remote federation](https://www.meilisearch.com/docs/reference/api/management/get-network-topology) | Federate search across multiple Meilisearch instances |
| Memory mapping | Efficient memory usage through memory-mapped storage |

## Database

### Document database

| Feature | Description |
| - | - |
| [Schemaless](https://www.meilisearch.com/docs/resources/internals/documents) | Index documents without a predefined schema |
| [Documents](https://www.meilisearch.com/docs/resources/internals/documents) | Add, replace, update, and delete documents |
| [Delete by filter](https://www.meilisearch.com/docs/reference/api/documents/delete-documents-by-filter) | Delete documents matching a filter expression |
| [Update by function](https://www.meilisearch.com/docs/reference/api/documents/edit-documents-by-function) | Partial updates to documents using functions |
| [Searchable attributes](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_displayed_attributes) | Configure which fields are searchable and their priority |
| [Displayed attributes](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_displayed_attributes) | Control which fields are returned in results |
| [Filterable attributes](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/getting_started) | Define which fields can be used in filters |
| [Sortable attributes](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/sort_results) | Define which fields can be used for sorting |
| [Index swap](https://www.meilisearch.com/docs/reference/api/indexes/swap-indexes) | Swap indexes to perform updates without downtime |

### Vector database

| Feature | Description |
| - | - |
| [Similar documents](https://www.meilisearch.com/docs/capabilities/personalization/getting_started/recommendations) | Find semantically similar documents using vector embeddings |
| [Binary quantization](https://www.meilisearch.com/docs/reference/api/settings/get-embedders) | Compress vectors to save storage |
| DiskANN | Disk-based approximate nearest neighbors for large datasets |
| [Auto embedding](https://www.meilisearch.com/docs/reference/api/settings/get-embedders) | Automatically generate embeddings without manual input |

## Platform features

### Security

Protect your data with API keys and multi-tenant access control.

| Feature | Description |
| - | - |
| [API keys](https://www.meilisearch.com/docs/resources/self_hosting/security/basic_security) | Admin, search, and chat key types for different access levels |
| [Tenant tokens](https://www.meilisearch.com/docs/capabilities/security/overview) | Secure multi-tenant applications with document-level access control |
| [Search rules](https://www.meilisearch.com/docs/capabilities/security/advanced/tenant_token_payload) | Restrict which documents users can access |

### Tasks and monitoring

Monitor indexing operations and receive notifications.

| Feature | Description |
| - | - |
| [Task management](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/monitor_tasks) | Track and manage async operations |
| [Batches](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations#task-batches) | Automatic task batching for efficient processing |
| [Webhooks](https://www.meilisearch.com/docs/resources/self_hosting/webhooks) | Get notified when tasks complete |
| Diff indexing | Only index differences between datasets |

### Analytics (Cloud)

Track search behavior and optimize relevancy with built-in analytics.

| Feature | Description |
| - | - |
| [Search analytics](https://www.meilisearch.com/docs/capabilities/analytics/getting_started) | Monitor search patterns, no-result rates, and top queries |
| [Click tracking](https://www.meilisearch.com/docs/capabilities/analytics/how_to/bind_events_to_user) | Track which results users engage with |
| [Metrics reference](https://www.meilisearch.com/docs/capabilities/analytics/advanced/analytics_metrics) | Click-through rate, conversion rate, average click position |
| Monitoring | Search latency, indexing latency, bandwidth, and API health |

### Pagination

| Feature | Description |
| - | - |
| [Pagination](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/paginate_search_results) | Offset/limit and cursor-based pagination |

## Language support

Meilisearch provides optimized support for many languages:

* **Latin-based languages**: English, French, Spanish, German, Italian, Portuguese, etc.
* **CJK**: Chinese, Japanese, Korean with specialized tokenization
* **RTL languages**: Hebrew, Arabic
* **Others**: Thai, Greek, and more

[See full language support →](https://www.meilisearch.com/docs/resources/help/language)

## Cloud features

These features are available exclusively on Meilisearch Cloud.

| Feature | Description |
| - | - |
| Crawler | Crawl web pages with JS rendering, DocSearch mode, and schema extraction |
| [Search preview](https://www.meilisearch.com/docs/resources/self_hosting/getting_started/search_preview) | Visual search interface with filtering, sorting, and document CRUD |
| [Teams](https://www.meilisearch.com/docs/capabilities/platform/teams/overview) | Organize projects and members into team workspaces |
| [Enterprise SSO/SCIM](https://www.meilisearch.com/docs/resources/self_hosting/enterprise_edition) | SAML 2.0 SSO and automated user provisioning |
| Autoscale disk | Automatically scale storage as data grows |
| Automatic backups | Scheduled backups for data safety |
| One-click upgrade | Upgrade Meilisearch version with a single click |
| Up to 99.999% SLA | Industry-leading service level agreement |

## Self-hosting

| Feature | Description |
| - | - |
| [Configuration](https://www.meilisearch.com/docs/resources/self_hosting/configuration/overview) | CLI flags and environment variables |
| [Snapshots](https://www.meilisearch.com/docs/resources/self_hosting/data_backup/snapshots) | Full binary copies for fast restoration |
| [Dumps](https://www.meilisearch.com/docs/resources/self_hosting/data_backup/dumps) | Portable JSON exports for migration |
| [Master key](https://www.meilisearch.com/docs/resources/self_hosting/security/basic_security) | Secure your instance with a master key |

## Integration options

| Option | Description |
| - | - |
| [REST API](https://www.meilisearch.com/docs/reference/api/openapi) | Direct HTTP integration |
| [Official SDKs](https://www.meilisearch.com/docs/resources/help/sdks) | 10+ language SDKs |
| [Frameworks](https://www.meilisearch.com/docs/resources/help/sdks#framework-sdks) | Laravel, Rails, Strapi, Symfony |


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.