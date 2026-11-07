# Get embedders

> Returns the current value of the `embedders` setting for the index.



## OpenAPI

<pre><code class="language-yaml">openapi: 3.1.0
info:
  title: meilisearch
  description: Meilisearch HTTP server
  contact:
    name: Quentin de Quelen
    email: quentin@dequelen.me
  license:
    name: MIT
    identifier: MIT
  version: 1.56.1
servers:
  - url: http://localhost:7700
    description: Local server.
security: []
tags:
  - name: Stats
    description: &gt;-
      Stats gives extended information and metrics about indexes and the
      Meilisearch database.
  - name: Health
    description: &gt;-
      The health check endpoint enables you to periodically test the health of
      your Meilisearch instance.
  - name: Version
    description: Returns the version of the running Meilisearch instance.
  - name: Backups
    description: &gt;-
      Meilisearch offers two types of backups: snapshots and dumps. Snapshots
      are mainly intended as a safeguard, while dumps are useful when migrating
      Meilisearch.
  - name: Export
    description: &gt;-
      Export documents and settings from this instance to a remote Meilisearch
      server.
  - name: Async task management
    description: &gt;-
      Routes for listing and managing batches and tasks (asynchronous
      operations).
  - name: Chats
    description: The `/chats` route allows you to manage chat workspaces.
  - name: MCP
    description: &gt;-
      Model Context Protocol (MCP) is an open protocol that enables seamless
      integration between LLM applications and external data sources and tools.
  - name: Indexes
    description: &gt;-
      An index is an entity that gathers a set of
      [documents](https://www.meilisearch.com/docs/learn/getting_started/documents)
      with its own
      [settings](https://www.meilisearch.com/docs/reference/api/settings). Learn
      more about indexes.
  - name: Documents
    description: &gt;-
      Documents are objects composed of fields that can store any type of data.
      Each field contains an attribute and its associated value. Documents are
      stored inside
      [indexes](https://www.meilisearch.com/docs/learn/getting_started/indexes).
  - name: Facet Search
    description: &gt;-
      The `/facet-search` route allows you to search for facet values within a
      given facet attribute.


      Facet search supports prefix search and typo tolerance. Results are sorted
      lexicographically in ascending order by default. You can configure sort
      order using the `sortFacetValuesBy` property of the faceting index
      settings.


      **Note:** Facet search does not support multi-word queries. Only the first
      word of `facetQuery` is considered. For example, searching for `Jane`
      returns `Jane Austen`, but searching for `Austen` does not.


      **Note:** Meilisearch does not support facet search on numeric values.
      Convert numeric facets to strings to make them searchable.


      **Prerequisite:** The `facetName` attribute must be in the index's
      `filterableAttributes` list before facet search can be used.
  - name: Similar documents
    description: &gt;-
      The /similar route uses AI-powered search to return a number of documents
      similar to a target document.


      Meilisearch exposes two routes for retrieving similar documents: POST and
      GET. In the majority of cases, POST will offer better performance and ease
      of use.
  - name: Settings
    description: &gt;-
      Configure search and index behavior. Update all settings at once via PATCH
      /indexes/{indexUid}/settings, or use a sub-route to get, update, or reset
      a single setting.
  - name: Search
    description: &gt;-
      Meilisearch exposes two routes to perform searches:


      - A POST route: this is the preferred route when using API authentication,
      as it allows [preflight
      request](https://developer.mozilla.org/en-US/docs/Glossary/Preflight_request)
      caching and better performance.

      - A GET route: the usage of this route is discouraged, unless you have
      good reason to do otherwise (specific caching abilities for example)
    externalDocs:
      url: https://www.meilisearch.com/docs/reference/api/search
      description: Search API reference
  - name: Keys
    description: &gt;-
      Manage API `keys` for a Meilisearch instance. Each key has a given set of
      permissions.

      You must have the master key or the default admin key to access the keys
      route. More information about the keys and their rights.

      Accessing any route under `/keys` without having set a master key will
      result in an error.
  - name: Logs
    description: &gt;-
      Everything about retrieving or customizing logs.

      Currently
      [experimental](https://www.meilisearch.com/docs/learn/experimental/overview).
  - name: Multi-search
    description: &gt;-
      The `/multi-search` route allows you to perform multiple search queries on
      one or more indexes by bundling them into a single HTTP request.
      Multi-search is also known as federated search.
  - name: Experimental features
    description: &gt;-
      The `/experimental-features` route allows you to activate or deactivate
      some of Meilisearch's experimental features.


      This route is **synchronous**. This means that no task object will be
      returned, and any activated or deactivated features will be made available
      or unavailable immediately.
  - name: Webhooks
    description: &gt;-
      The `/webhooks` route allows you to register endpoints to be called once
      tasks are processed.
  - name: Search rules
    description: The `/dynamic-search-rules` route allows you to configure search rules.
  - name: Render templates
    description: The /render-template route allows rendering templates used by Meilisearch.
    externalDocs:
      url: https://www.meilisearch.com/docs/reference/api/render-template
      description: Render template API reference
paths:
  /indexes/{index_uid}/settings/embedders:
    get:
      tags:
        - Settings
      summary: Get embedders
      description: Returns the current value of the `embedders` setting for the index.
      operationId: getembedders
      parameters:
        - name: index_uid
<span id="parameter-index-uid"></span>          in: path
          description: Unique identifier of the index.
          required: true
          schema:
            type: string
          example: movies
      responses:
        '200':
          description: Returns the current value of the `embedders` setting.
          content:
            application/json:
              schema:
                type: object
<span id="response-additional-properties"></span>                additionalProperties:
                  $ref: '#/components/schemas/SettingEmbeddingSettings'
                propertyNames:
                  type: string
              example: {}
        '401':
          description: The authorization header is missing.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ResponseError'
              example:
                message: &gt;-
                  The Authorization header is missing. It must use the bearer
                  authorization method.
                code: missing_authorization_header
                type: auth
                link: &gt;-
                  https://docs.meilisearch.com/errors#missing_authorization_header
        '404':
          description: Index not found.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ResponseError'
              example:
                message: Index `movies` not found.
                code: index_not_found
                type: invalid_request
                link: https://docs.meilisearch.com/errors#index_not_found
      security:
        - Bearer:
            - settings.get
            - settings.*
            - '*'
      x-codeSamples:
        - lang: cURL
          source: |-
            curl \
              -X GET 'MEILISEARCH_URL/indexes/INDEX_NAME/settings/embedders' \
              -H 'Authorization: Bearer API_KEY'
        - lang: Python
          source: client.index('INDEX_NAME').get_embedders()
        - lang: Ruby
          source: client.index('INDEX_NAME').embedders
        - lang: Rust
          source: let embedders = index.get_embedders().await.unwrap();
components:
  schemas:
    SettingEmbeddingSettings:
      type: object
      description: &gt;-
        Configuration for one
        [embedder](https://www.meilisearch.com/docs/learn/ai_powered_search/getting_started_with_ai_search)
        used for semantic and hybrid search.


        Set `source` (`openAi`, `huggingFace`, `ollama`, `rest`,
        `userProvided`), then the options that apply: `model`, `apiKey`,
        `documentTemplate`, `dimensions`, `url`, etc.
      properties:
<span id="response-additional-properties-inner"></span><span id="response-inner"></span>        inner:
          oneOf:
<span id="response-additional-properties-inner-one-of-0"></span><span id="response-inner-one-of-0"></span>            - type: 'null'
<span id="response-additional-properties-inner-one-of-1"></span><span id="response-inner-one-of-1"></span>            - type: object
              description: &gt;-
                Embedder configuration for [AI-powered / hybrid
                search](https://www.meilisearch.com/docs/learn/ai_powered_search/getting_started_with_ai_search).
              properties:
<span id="response-additional-properties-inner-one-of-1-source"></span><span id="response-additional-properties-inner-source"></span><span id="response-inner-one-of-1-source"></span><span id="response-inner-source"></span>                source:
                  oneOf:
<span id="response-additional-properties-inner-one-of-1-source-one-of-0"></span><span id="response-additional-properties-inner-source-one-of-0"></span><span id="response-inner-one-of-1-source-one-of-0"></span><span id="response-inner-source-one-of-0"></span>                    - type: 'null'
<span id="response-additional-properties-inner-one-of-1-source-one-of-1"></span><span id="response-additional-properties-inner-source-one-of-1"></span><span id="response-inner-one-of-1-source-one-of-1"></span><span id="response-inner-source-one-of-1"></span>                    - $ref: '#/components/schemas/EmbedderSource'
                      description: &gt;-
                        Embedding provider.


                        - `openAi`: integration with OpenAI. Use `rest` for
                        OpenAI-like APIs.

                        - `huggingFace`: download a model from [Hugging
                        Face](https://huggingface.co/), then compute embeddings
                        on the local CPU/GPU.

                        - `ollama`: integration with
                        [ollama](https://ollama.com/).

                        - `rest`: configurable integration with a remote
                        embedder via its REST API.

                        - `composite`: combine two embedding providers at
                        indexing and search time.
                          The generated embeddings must be **exactly the same** regardless of the embedding provider.
                        - `userProvided`: manually provide embeddings in
                        documents.


                        - Determines which other fields are required or allowed.

                        - 🏗️ Changing the value of this parameter always
                        regenerates embeddings.
                  default: openAi
<span id="response-additional-properties-inner-model"></span><span id="response-additional-properties-inner-one-of-1-model"></span><span id="response-inner-model"></span><span id="response-inner-one-of-1-model"></span>                model:
                  type:
                    - string
                    - 'null'
                  description: &gt;-
                    Model name.


                    - Available for `openAi`, `huggingFace`, `ollama`.

                    - Mandatory for `ollama`.

                    - For `openAi`/`huggingFace` optional with defaults.

                    - 🏗️ Changing the value of this parameter always
                    regenerates embeddings.
                  example: text-embedding-3-small
<span id="response-additional-properties-inner-one-of-1-revision"></span><span id="response-additional-properties-inner-revision"></span><span id="response-inner-one-of-1-revision"></span><span id="response-inner-revision"></span>                revision:
                  type:
                    - string
                    - 'null'
                  description: &gt;-
                    Model revision (e.g. Hugging Face commit SHA).


                    - Only available for `huggingFace`.

                    - If unset, latest is used.

                    - 🏗️ Changing the value of this parameter always
                    regenerates embeddings
<span id="response-additional-properties-inner-one-of-1-pooling"></span><span id="response-additional-properties-inner-pooling"></span><span id="response-inner-one-of-1-pooling"></span><span id="response-inner-pooling"></span>                pooling:
                  oneOf:
<span id="response-additional-properties-inner-one-of-1-pooling-one-of-0"></span><span id="response-additional-properties-inner-pooling-one-of-0"></span><span id="response-inner-one-of-1-pooling-one-of-0"></span><span id="response-inner-pooling-one-of-0"></span>                    - type: 'null'
<span id="response-additional-properties-inner-one-of-1-pooling-one-of-1"></span><span id="response-additional-properties-inner-pooling-one-of-1"></span><span id="response-inner-one-of-1-pooling-one-of-1"></span><span id="response-inner-pooling-one-of-1"></span>                    - $ref: '#/components/schemas/OverridePooling'
                      description: &gt;-
                        Pooling method when computing embedding locally.


                        - Only available for `huggingFace`.

                        - `useModel`: automatically determine the pooling method
                        depending on the chosen model (recommended).

                        - `forceMean`: always use mean pooling.

                        - `forceCls`: always use CLS pooling.

                        - 🏗️ Changing the value of this parameter always
                        regenerates embeddings
                  default: useModel
<span id="response-additional-properties-inner-api-key"></span><span id="response-additional-properties-inner-apikey"></span><span id="response-additional-properties-inner-one-of-1-api-key"></span><span id="response-additional-properties-inner-one-of-1-apikey"></span><span id="response-inner-api-key"></span><span id="response-inner-apikey"></span><span id="response-inner-one-of-1-api-key"></span><span id="response-inner-one-of-1-apikey"></span>                apiKey:
                  type:
                    - string
                    - 'null'
                  description: &gt;-
                    API key sent to the embedder.


                    - If not set for source `openAi`, the key is read from the
                    `OPENAI_API_KEY` then `MEILI_OPENAI_API_KEY` environment
                    variables.

                    - If not set for other sources, no bearer token is sent.

                    - This setting is partially hidden when returned by the
                    settings

                    - 🌱 Changing the value of this parameter never regenerates
                    embeddings
<span id="response-additional-properties-inner-dimensions"></span><span id="response-additional-properties-inner-one-of-1-dimensions"></span><span id="response-inner-dimensions"></span><span id="response-inner-one-of-1-dimensions"></span>                dimensions:
                  type:
                    - integer
                    - 'null'
                  description: &gt;-
                    Number of dimensions of the embedding vectors.


                    - Available for `userProvided`, `openAi`, `ollama`, `rest`.

                    - Mandatory for `userProvided`.

                    - For `openAi`/`ollama`/`rest` inferred if omitted.

                    - 🏗️ For `openAi`, changing the value of this parameter
                    always regenerates embeddings

                    - 🌱 For other sources, changing the value of this parameter
                    never regenerates embeddings
                  example: 1536
                  minimum: 0
<span id="response-additional-properties-inner-binary-quantized"></span><span id="response-additional-properties-inner-binaryquantized"></span><span id="response-additional-properties-inner-one-of-1-binary-quantized"></span><span id="response-additional-properties-inner-one-of-1-binaryquantized"></span><span id="response-inner-binary-quantized"></span><span id="response-inner-binaryquantized"></span><span id="response-inner-one-of-1-binary-quantized"></span><span id="response-inner-one-of-1-binaryquantized"></span>                binaryQuantized:
                  type:
                    - boolean
                    - 'null'
                  description: &gt;-
                    When true, vectors are stored as 1-bit (smaller, faster,
                    less precise).


                    - Available for all sources.

                    - Irreversible once enabled.

                    - 🏗️ When set to true, embeddings are not regenerated, but
                    they are binary quantized, which takes time.
<span id="response-additional-properties-inner-document-template"></span><span id="response-additional-properties-inner-documenttemplate"></span><span id="response-additional-properties-inner-one-of-1-document-template"></span><span id="response-additional-properties-inner-one-of-1-documenttemplate"></span><span id="response-inner-document-template"></span><span id="response-inner-documenttemplate"></span><span id="response-inner-one-of-1-document-template"></span><span id="response-inner-one-of-1-documenttemplate"></span>                documentTemplate:
                  type:
                    - string
                    - 'null'
                  description: &gt;-
                    [Liquid template](https://shopify.github.io/liquid/) to
                    build the text sent to the embedder for each document.


                    - Available for `openAi`, `huggingFace`, `ollama`, `rest`.

                    - 🏗️ When modified, embeddings are regenerated for
                    documents whose rendering through the template produces a
                    different text.
                  example: '{{doc.title}}: {{doc.overview}}'
<span id="response-additional-properties-inner-document-template-max-bytes"></span><span id="response-additional-properties-inner-documenttemplatemaxbytes"></span><span id="response-additional-properties-inner-one-of-1-document-template-max-bytes"></span><span id="response-additional-properties-inner-one-of-1-documenttemplatemaxbytes"></span><span id="response-inner-document-template-max-bytes"></span><span id="response-inner-documenttemplatemaxbytes"></span><span id="response-inner-one-of-1-document-template-max-bytes"></span><span id="response-inner-one-of-1-documenttemplatemaxbytes"></span>                documentTemplateMaxBytes:
                  type:
                    - integer
                    - 'null'
                  description: &gt;-
                    Max size in bytes of the rendered document template.


                    Longer output is truncated.


                    - Available for `openAi`, `huggingFace`, `ollama`, `rest`.

                    - 🏗️ When increased, embeddings are regenerated for
                    documents whose rendering through the template produces a
                    different text.

                    - 🌱 When decreased, embeddings are never regenerated
                  default: 400
                  example: 400
                  minimum: 0
<span id="response-additional-properties-inner-one-of-1-url"></span><span id="response-additional-properties-inner-url"></span><span id="response-inner-one-of-1-url"></span><span id="response-inner-url"></span>                url:
                  type:
                    - string
                    - 'null'
                  description: &gt;-
                    URL of the embedder API.


                    - Mandatory for `rest`. Optional for `openAi` and `ollama`.

                    - 🌱 When modified for `openAi``, embeddings are never
                    regenerated

                    - 🏗️ When modified for `ollama` and `rest`, embeddings are
                    always regenerated

                    - If targetting URL resolving to a non-global IP (such as
                    `localhost`), make sure that
                      `--experimental-allowed-ip-networks` allows it. For details on how use this parameter,
                      refer to [this documentation](https://www.meilisearch.com/docs/learn/self_hosted/configure_meilisearch_at_launch#allow-requests-to-private-networks).
                  example: http://localhost:11434/api/embeddings
<span id="response-additional-properties-inner-indexing-fragments"></span><span id="response-additional-properties-inner-indexingfragments"></span><span id="response-additional-properties-inner-one-of-1-indexing-fragments"></span><span id="response-additional-properties-inner-one-of-1-indexingfragments"></span><span id="response-inner-indexing-fragments"></span><span id="response-inner-indexingfragments"></span><span id="response-inner-one-of-1-indexing-fragments"></span><span id="response-inner-one-of-1-indexingfragments"></span>                indexingFragments:
                  type:
                    - object
                    - 'null'
                  description: &gt;-
                    Fragments (with [Liquid](https://shopify.github.io/liquid/))
                    sent to the embedder at indexing time.


                    For `rest` with multimodal; key is fragment name, value the
                    payload.


                    See also [Image search with multimodal
                    embeddings](https://www.meilisearch.com/docs/learn/ai_powered_search/image_search_with_multimodal_embeddings)


                    - 🏗️ When a fragment is deleted by passing `null` to its
                    name, the corresponding embeddings are removed from
                    documents.

                    - 🏗️ When a fragment is modified, the corresponding
                    embeddings are regenerated if their rendered version
                    changes.
                  default: {}
<span id="response-additional-properties-inner-indexing-fragments-additional-properties"></span><span id="response-additional-properties-inner-one-of-1-indexing-fragments-additional-properties"></span><span id="response-inner-indexing-fragments-additional-properties"></span><span id="response-inner-one-of-1-indexing-fragments-additional-properties"></span>                  additionalProperties: {}
                  propertyNames:
                    type: string
<span id="response-additional-properties-inner-one-of-1-search-fragments"></span><span id="response-additional-properties-inner-one-of-1-searchfragments"></span><span id="response-additional-properties-inner-search-fragments"></span><span id="response-additional-properties-inner-searchfragments"></span><span id="response-inner-one-of-1-search-fragments"></span><span id="response-inner-one-of-1-searchfragments"></span><span id="response-inner-search-fragments"></span><span id="response-inner-searchfragments"></span>                searchFragments:
                  type:
                    - object
                    - 'null'
                  description: &gt;-
                    Fragments (with [Liquid](https://shopify.github.io/liquid/))
                    sent to the embedder at search time.


                    For `rest` with multimodal.


                    See also [Image search with multimodal
                    embeddings](https://www.meilisearch.com/docs/learn/ai_powered_search/image_search_with_multimodal_embeddings)


                    - 🌱 Changing the value of this parameter never regenerates
                    embeddings
                  default: {}
<span id="response-additional-properties-inner-one-of-1-search-fragments-additional-properties"></span><span id="response-additional-properties-inner-search-fragments-additional-properties"></span><span id="response-inner-one-of-1-search-fragments-additional-properties"></span><span id="response-inner-search-fragments-additional-properties"></span>                  additionalProperties: {}
                  propertyNames:
                    type: string
<span id="response-additional-properties-inner-one-of-1-request"></span><span id="response-additional-properties-inner-request"></span><span id="response-inner-one-of-1-request"></span><span id="response-inner-request"></span>                request:
                  description: &gt;-
                    Request body template for `rest` embedder.


                    See also [Configure a REST
                    embedder](https://www.meilisearch.com/docs/learn/ai_powered_search/configure_rest_embedder)


                    - Use `"{{text}}"` for the input.

                    - Mandatory for `rest`.

                    - 🏗️ Changing the value of this parameter always
                    regenerates embeddings
<span id="response-additional-properties-inner-one-of-1-response"></span><span id="response-additional-properties-inner-response"></span><span id="response-inner-one-of-1-response"></span><span id="response-inner-response"></span>                response:
                  description: &gt;-
                    Response template for `rest` embedder.


                    See also [Configure a REST
                    embedder](https://www.meilisearch.com/docs/learn/ai_powered_search/configure_rest_embedder)


                    - Use `"{{embedding}}"` where the embedding array is.

                    - Mandatory for `rest`.
<span id="response-additional-properties-inner-headers"></span><span id="response-additional-properties-inner-one-of-1-headers"></span><span id="response-inner-headers"></span><span id="response-inner-one-of-1-headers"></span>                headers:
                  type:
                    - object
                    - 'null'
                  description: &gt;-
                    Extra HTTP headers sent to the embedder.


                    - Available for `rest`.

                    - 🌱 Changing the value of this parameter never regenerates
                    embeddings
<span id="response-additional-properties-inner-headers-additional-properties"></span><span id="response-additional-properties-inner-one-of-1-headers-additional-properties"></span><span id="response-inner-headers-additional-properties"></span><span id="response-inner-one-of-1-headers-additional-properties"></span>                  additionalProperties:
                    type: string
                  propertyNames:
                    type: string
<span id="response-additional-properties-inner-one-of-1-search-embedder"></span><span id="response-additional-properties-inner-one-of-1-searchembedder"></span><span id="response-additional-properties-inner-search-embedder"></span><span id="response-additional-properties-inner-searchembedder"></span><span id="response-inner-one-of-1-search-embedder"></span><span id="response-inner-one-of-1-searchembedder"></span><span id="response-inner-search-embedder"></span><span id="response-inner-searchembedder"></span>                searchEmbedder:
                  oneOf:
<span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-0"></span><span id="response-additional-properties-inner-search-embedder-one-of-0"></span><span id="response-inner-one-of-1-search-embedder-one-of-0"></span><span id="response-inner-search-embedder-one-of-0"></span>                    - type: 'null'
<span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1"></span><span id="response-additional-properties-inner-search-embedder-one-of-1"></span><span id="response-inner-one-of-1-search-embedder-one-of-1"></span><span id="response-inner-search-embedder-one-of-1"></span>                    - $ref: '#/components/schemas/SubEmbeddingSettings'
                      description: &gt;-
                        Embedder used at search time when using a composite
                        embedder.


                        Same sub-parameters as a top-level embedder (`source`,
                        `model`, `url`, etc.); no
                        `documentTemplate`/`documentTemplateMaxBytes`.
<span id="response-additional-properties-inner-indexing-embedder"></span><span id="response-additional-properties-inner-indexingembedder"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder"></span><span id="response-additional-properties-inner-one-of-1-indexingembedder"></span><span id="response-inner-indexing-embedder"></span><span id="response-inner-indexingembedder"></span><span id="response-inner-one-of-1-indexing-embedder"></span><span id="response-inner-one-of-1-indexingembedder"></span>                indexingEmbedder:
                  oneOf:
<span id="response-additional-properties-inner-indexing-embedder-one-of-0"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-0"></span><span id="response-inner-indexing-embedder-one-of-0"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-0"></span>                    - type: 'null'
<span id="response-additional-properties-inner-indexing-embedder-one-of-1"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1"></span><span id="response-inner-indexing-embedder-one-of-1"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1"></span>                    - $ref: '#/components/schemas/SubEmbeddingSettings'
                      description: &gt;-
                        Embedder used at indexing time when using a composite
                        embedder.


                        Same sub-parameters as a top-level embedder (`source`,
                        `model`, `documentTemplate`, `url`, etc.).
<span id="response-additional-properties-inner-distribution"></span><span id="response-additional-properties-inner-one-of-1-distribution"></span><span id="response-inner-distribution"></span><span id="response-inner-one-of-1-distribution"></span>                distribution:
                  oneOf:
<span id="response-additional-properties-inner-distribution-one-of-0"></span><span id="response-additional-properties-inner-one-of-1-distribution-one-of-0"></span><span id="response-inner-distribution-one-of-0"></span><span id="response-inner-one-of-1-distribution-one-of-0"></span>                    - type: 'null'
<span id="response-additional-properties-inner-distribution-one-of-1"></span><span id="response-additional-properties-inner-one-of-1-distribution-one-of-1"></span><span id="response-inner-distribution-one-of-1"></span><span id="response-inner-one-of-1-distribution-one-of-1"></span>                    - $ref: '#/components/schemas/DistributionShift'
                      description: &gt;-
                        Affine transform (mean, sigma) applied to the semantic
                        score to make it more comparable with the ranking score.


                        See also [this
                        article](https://www.meilisearch.com/blog/fixing-hybrid-search#making-the-scores-comparable).


                        - Available for all sources.

                        - 🌱 Changing the value of this parameter never
                        regenerates embeddings
    ResponseError:
      type: object
      required:
        - message
        - code
        - type
        - link
      properties:
        message:
          type: string
          description: The error message.
        code:
          $ref: '#/components/schemas/Code'
          description: The error code.
        type:
          $ref: '#/components/schemas/ErrorType'
          description: The error type.
        link:
          type: string
          description: A link to the documentation about this specific error.
    EmbedderSource:
      type: string
      enum:
        - openAi
        - huggingFace
        - ollama
        - userProvided
        - rest
        - composite
    OverridePooling:
      type: string
      enum:
        - useModel
        - forceCls
        - forceMean
    SubEmbeddingSettings:
      type: object
      description: &gt;-
        Sub-embedder config for composite embedders. Used by `indexingEmbedder`
        (vectorize documents at index time) and `searchEmbedder` (vectorize
        queries at search time). Same fields as a top-level embedder, except no
        `binaryQuantized`, `distribution`,
        `documentTemplate`/`documentTemplateMaxBytes` for searchEmbedder.
      properties:
<span id="response-additional-properties-inner-indexing-embedder-one-of-1-source"></span><span id="response-additional-properties-inner-indexing-embedder-source"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-source"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-source"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-source"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-source"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-source"></span><span id="response-additional-properties-inner-search-embedder-source"></span><span id="response-inner-indexing-embedder-one-of-1-source"></span><span id="response-inner-indexing-embedder-source"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-source"></span><span id="response-inner-one-of-1-indexing-embedder-source"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-source"></span><span id="response-inner-one-of-1-search-embedder-source"></span><span id="response-inner-search-embedder-one-of-1-source"></span><span id="response-inner-search-embedder-source"></span>        source:
          oneOf:
<span id="response-additional-properties-inner-indexing-embedder-one-of-1-source-one-of-0"></span><span id="response-additional-properties-inner-indexing-embedder-source-one-of-0"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-source-one-of-0"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-source-one-of-0"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-source-one-of-0"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-source-one-of-0"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-source-one-of-0"></span><span id="response-additional-properties-inner-search-embedder-source-one-of-0"></span><span id="response-inner-indexing-embedder-one-of-1-source-one-of-0"></span><span id="response-inner-indexing-embedder-source-one-of-0"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-source-one-of-0"></span><span id="response-inner-one-of-1-indexing-embedder-source-one-of-0"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-source-one-of-0"></span><span id="response-inner-one-of-1-search-embedder-source-one-of-0"></span><span id="response-inner-search-embedder-one-of-1-source-one-of-0"></span><span id="response-inner-search-embedder-source-one-of-0"></span>            - type: 'null'
<span id="response-additional-properties-inner-indexing-embedder-one-of-1-source-one-of-1"></span><span id="response-additional-properties-inner-indexing-embedder-source-one-of-1"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-source-one-of-1"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-source-one-of-1"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-source-one-of-1"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-source-one-of-1"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-source-one-of-1"></span><span id="response-additional-properties-inner-search-embedder-source-one-of-1"></span><span id="response-inner-indexing-embedder-one-of-1-source-one-of-1"></span><span id="response-inner-indexing-embedder-source-one-of-1"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-source-one-of-1"></span><span id="response-inner-one-of-1-indexing-embedder-source-one-of-1"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-source-one-of-1"></span><span id="response-inner-one-of-1-search-embedder-source-one-of-1"></span><span id="response-inner-search-embedder-one-of-1-source-one-of-1"></span><span id="response-inner-search-embedder-source-one-of-1"></span>            - $ref: '#/components/schemas/EmbedderSource'
              description: &gt;-
                Embedding provider.


                - `openAi`: integration with OpenAI. Use `rest` for OpenAI-like
                APIs.

                - `huggingFace`: download a model from [Hugging
                Face](https://huggingface.co/), then compute embeddings on the
                local CPU/GPU.

                - `ollama`: integration with [ollama](https://ollama.com/).

                - `rest`: configurable integration with a remote embedder via
                its REST API.

                - `userProvided`: manually provide embeddings in documents.


                - Determines which other fields are required or allowed.

                - 🏗️ Changing the value of this parameter always regenerates
                embeddings.
          default: openAi
<span id="response-additional-properties-inner-indexing-embedder-model"></span><span id="response-additional-properties-inner-indexing-embedder-one-of-1-model"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-model"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-model"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-model"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-model"></span><span id="response-additional-properties-inner-search-embedder-model"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-model"></span><span id="response-inner-indexing-embedder-model"></span><span id="response-inner-indexing-embedder-one-of-1-model"></span><span id="response-inner-one-of-1-indexing-embedder-model"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-model"></span><span id="response-inner-one-of-1-search-embedder-model"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-model"></span><span id="response-inner-search-embedder-model"></span><span id="response-inner-search-embedder-one-of-1-model"></span>        model:
          type:
            - string
            - 'null'
          description: &gt;-
            Model name.


            - Available for `openAi`, `huggingFace`, `ollama`.

            - Mandatory for `ollama`.

            - For `openAi`/`huggingFace` optional with defaults.

            - 🏗️ Changing the value of this parameter always regenerates
            embeddings.
          example: text-embedding-3-small
<span id="response-additional-properties-inner-indexing-embedder-one-of-1-revision"></span><span id="response-additional-properties-inner-indexing-embedder-revision"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-revision"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-revision"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-revision"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-revision"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-revision"></span><span id="response-additional-properties-inner-search-embedder-revision"></span><span id="response-inner-indexing-embedder-one-of-1-revision"></span><span id="response-inner-indexing-embedder-revision"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-revision"></span><span id="response-inner-one-of-1-indexing-embedder-revision"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-revision"></span><span id="response-inner-one-of-1-search-embedder-revision"></span><span id="response-inner-search-embedder-one-of-1-revision"></span><span id="response-inner-search-embedder-revision"></span>        revision:
          type:
            - string
            - 'null'
          description: &gt;-
            Model revision (e.g. Hugging Face commit SHA).


            - Only available for `huggingFace`.

            - If unset, latest is used.

            - 🏗️ Changing the value of this parameter always regenerates
            embeddings
          example: 617ca489d9e86b49b8167676d8220688b99db36e
<span id="response-additional-properties-inner-indexing-embedder-one-of-1-pooling"></span><span id="response-additional-properties-inner-indexing-embedder-pooling"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-pooling"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-pooling"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-pooling"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-pooling"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-pooling"></span><span id="response-additional-properties-inner-search-embedder-pooling"></span><span id="response-inner-indexing-embedder-one-of-1-pooling"></span><span id="response-inner-indexing-embedder-pooling"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-pooling"></span><span id="response-inner-one-of-1-indexing-embedder-pooling"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-pooling"></span><span id="response-inner-one-of-1-search-embedder-pooling"></span><span id="response-inner-search-embedder-one-of-1-pooling"></span><span id="response-inner-search-embedder-pooling"></span>        pooling:
          oneOf:
<span id="response-additional-properties-inner-indexing-embedder-one-of-1-pooling-one-of-0"></span><span id="response-additional-properties-inner-indexing-embedder-pooling-one-of-0"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-pooling-one-of-0"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-pooling-one-of-0"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-pooling-one-of-0"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-pooling-one-of-0"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-pooling-one-of-0"></span><span id="response-additional-properties-inner-search-embedder-pooling-one-of-0"></span><span id="response-inner-indexing-embedder-one-of-1-pooling-one-of-0"></span><span id="response-inner-indexing-embedder-pooling-one-of-0"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-pooling-one-of-0"></span><span id="response-inner-one-of-1-indexing-embedder-pooling-one-of-0"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-pooling-one-of-0"></span><span id="response-inner-one-of-1-search-embedder-pooling-one-of-0"></span><span id="response-inner-search-embedder-one-of-1-pooling-one-of-0"></span><span id="response-inner-search-embedder-pooling-one-of-0"></span>            - type: 'null'
<span id="response-additional-properties-inner-indexing-embedder-one-of-1-pooling-one-of-1"></span><span id="response-additional-properties-inner-indexing-embedder-pooling-one-of-1"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-pooling-one-of-1"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-pooling-one-of-1"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-pooling-one-of-1"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-pooling-one-of-1"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-pooling-one-of-1"></span><span id="response-additional-properties-inner-search-embedder-pooling-one-of-1"></span><span id="response-inner-indexing-embedder-one-of-1-pooling-one-of-1"></span><span id="response-inner-indexing-embedder-pooling-one-of-1"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-pooling-one-of-1"></span><span id="response-inner-one-of-1-indexing-embedder-pooling-one-of-1"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-pooling-one-of-1"></span><span id="response-inner-one-of-1-search-embedder-pooling-one-of-1"></span><span id="response-inner-search-embedder-one-of-1-pooling-one-of-1"></span><span id="response-inner-search-embedder-pooling-one-of-1"></span>            - $ref: '#/components/schemas/OverridePooling'
              description: &gt;-
                Pooling method when computing embedding locally.


                - Only available for `huggingFace`.

                - `useModel`: automatically determine the pooling method
                depending on the chosen model (recommended).

                - `forceMean`: always use mean pooling.

                - `forceCls`: always use CLS pooling.

                - 🏗️ Changing the value of this parameter always regenerates
                embeddings
          default: useModel
<span id="response-additional-properties-inner-indexing-embedder-api-key"></span><span id="response-additional-properties-inner-indexing-embedder-apikey"></span><span id="response-additional-properties-inner-indexing-embedder-one-of-1-api-key"></span><span id="response-additional-properties-inner-indexing-embedder-one-of-1-apikey"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-api-key"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-apikey"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-api-key"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-apikey"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-api-key"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-apikey"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-api-key"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-apikey"></span><span id="response-additional-properties-inner-search-embedder-api-key"></span><span id="response-additional-properties-inner-search-embedder-apikey"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-api-key"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-apikey"></span><span id="response-inner-indexing-embedder-api-key"></span><span id="response-inner-indexing-embedder-apikey"></span><span id="response-inner-indexing-embedder-one-of-1-api-key"></span><span id="response-inner-indexing-embedder-one-of-1-apikey"></span><span id="response-inner-one-of-1-indexing-embedder-api-key"></span><span id="response-inner-one-of-1-indexing-embedder-apikey"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-api-key"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-apikey"></span><span id="response-inner-one-of-1-search-embedder-api-key"></span><span id="response-inner-one-of-1-search-embedder-apikey"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-api-key"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-apikey"></span><span id="response-inner-search-embedder-api-key"></span><span id="response-inner-search-embedder-apikey"></span><span id="response-inner-search-embedder-one-of-1-api-key"></span><span id="response-inner-search-embedder-one-of-1-apikey"></span>        apiKey:
          type:
            - string
            - 'null'
          description: &gt;-
            API key sent to the embedder.


            - If not set for source `openAi`, the key is read from the
            `OPENAI_API_KEY` then `MEILI_OPENAI_API_KEY` environment variables.

            - If not set for other sources, no bearer token is sent.

            - This setting is partially hidden when returned by the settings

            - 🌱 Changing the value of this parameter never regenerates
            embeddings
          example: your-api-key
<span id="response-additional-properties-inner-indexing-embedder-dimensions"></span><span id="response-additional-properties-inner-indexing-embedder-one-of-1-dimensions"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-dimensions"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-dimensions"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-dimensions"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-dimensions"></span><span id="response-additional-properties-inner-search-embedder-dimensions"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-dimensions"></span><span id="response-inner-indexing-embedder-dimensions"></span><span id="response-inner-indexing-embedder-one-of-1-dimensions"></span><span id="response-inner-one-of-1-indexing-embedder-dimensions"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-dimensions"></span><span id="response-inner-one-of-1-search-embedder-dimensions"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-dimensions"></span><span id="response-inner-search-embedder-dimensions"></span><span id="response-inner-search-embedder-one-of-1-dimensions"></span>        dimensions:
          type:
            - integer
            - 'null'
          description: &gt;-
            Number of dimensions of the embedding vectors.


            - Available for `userProvided`, `openAi`, `ollama`, `rest`.

            - Mandatory for `userProvided`.

            - For `openAi`/`ollama`/`rest` inferred if omitted.

            - 🏗️ For `openAi`, changing the value of this parameter always
            regenerates embeddings

            - 🌱 For other sources, changing the value of this parameter never
            regenerates embeddings
          example: 1536
          minimum: 0
<span id="response-additional-properties-inner-indexing-embedder-document-template"></span><span id="response-additional-properties-inner-indexing-embedder-documenttemplate"></span><span id="response-additional-properties-inner-indexing-embedder-one-of-1-document-template"></span><span id="response-additional-properties-inner-indexing-embedder-one-of-1-documenttemplate"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-document-template"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-documenttemplate"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-document-template"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-documenttemplate"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-document-template"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-documenttemplate"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-document-template"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-documenttemplate"></span><span id="response-additional-properties-inner-search-embedder-document-template"></span><span id="response-additional-properties-inner-search-embedder-documenttemplate"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-document-template"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-documenttemplate"></span><span id="response-inner-indexing-embedder-document-template"></span><span id="response-inner-indexing-embedder-documenttemplate"></span><span id="response-inner-indexing-embedder-one-of-1-document-template"></span><span id="response-inner-indexing-embedder-one-of-1-documenttemplate"></span><span id="response-inner-one-of-1-indexing-embedder-document-template"></span><span id="response-inner-one-of-1-indexing-embedder-documenttemplate"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-document-template"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-documenttemplate"></span><span id="response-inner-one-of-1-search-embedder-document-template"></span><span id="response-inner-one-of-1-search-embedder-documenttemplate"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-document-template"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-documenttemplate"></span><span id="response-inner-search-embedder-document-template"></span><span id="response-inner-search-embedder-documenttemplate"></span><span id="response-inner-search-embedder-one-of-1-document-template"></span><span id="response-inner-search-embedder-one-of-1-documenttemplate"></span>        documentTemplate:
          type:
            - string
            - 'null'
          description: &gt;-
            [Liquid template](https://shopify.github.io/liquid/) to build the
            text sent to the embedder for each document.


            - Only available for `indexingEmbedder`, not `searchEmbedder`.

            - Available for `openAi`, `huggingFace`, `ollama`, `rest`.

            - 🏗️ When modified, embeddings are regenerated for documents whose
            rendering through the template produces a different text.
          example: '{{doc.title}}: {{doc.overview}}'
<span id="response-additional-properties-inner-indexing-embedder-document-template-max-bytes"></span><span id="response-additional-properties-inner-indexing-embedder-documenttemplatemaxbytes"></span><span id="response-additional-properties-inner-indexing-embedder-one-of-1-document-template-max-bytes"></span><span id="response-additional-properties-inner-indexing-embedder-one-of-1-documenttemplatemaxbytes"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-document-template-max-bytes"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-documenttemplatemaxbytes"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-document-template-max-bytes"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-documenttemplatemaxbytes"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-document-template-max-bytes"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-documenttemplatemaxbytes"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-document-template-max-bytes"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-documenttemplatemaxbytes"></span><span id="response-additional-properties-inner-search-embedder-document-template-max-bytes"></span><span id="response-additional-properties-inner-search-embedder-documenttemplatemaxbytes"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-document-template-max-bytes"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-documenttemplatemaxbytes"></span><span id="response-inner-indexing-embedder-document-template-max-bytes"></span><span id="response-inner-indexing-embedder-documenttemplatemaxbytes"></span><span id="response-inner-indexing-embedder-one-of-1-document-template-max-bytes"></span><span id="response-inner-indexing-embedder-one-of-1-documenttemplatemaxbytes"></span><span id="response-inner-one-of-1-indexing-embedder-document-template-max-bytes"></span><span id="response-inner-one-of-1-indexing-embedder-documenttemplatemaxbytes"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-document-template-max-bytes"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-documenttemplatemaxbytes"></span><span id="response-inner-one-of-1-search-embedder-document-template-max-bytes"></span><span id="response-inner-one-of-1-search-embedder-documenttemplatemaxbytes"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-document-template-max-bytes"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-documenttemplatemaxbytes"></span><span id="response-inner-search-embedder-document-template-max-bytes"></span><span id="response-inner-search-embedder-documenttemplatemaxbytes"></span><span id="response-inner-search-embedder-one-of-1-document-template-max-bytes"></span><span id="response-inner-search-embedder-one-of-1-documenttemplatemaxbytes"></span>        documentTemplateMaxBytes:
          type:
            - integer
            - 'null'
          description: &gt;-
            Max size in bytes of the rendered document template.


            Longer output is truncated.


            - Only available for `indexingEmbedder`, not `searchEmbedder`.

            - Available for `openAi`, `huggingFace`, `ollama`, `rest`.

            - 🏗️ When increased, embeddings are regenerated for documents whose
            rendering through the template produces a different text.

            - 🌱 When decreased, embeddings are never regenerated
          default: 400
          example: 400
          minimum: 0
<span id="response-additional-properties-inner-indexing-embedder-one-of-1-url"></span><span id="response-additional-properties-inner-indexing-embedder-url"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-url"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-url"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-url"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-url"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-url"></span><span id="response-additional-properties-inner-search-embedder-url"></span><span id="response-inner-indexing-embedder-one-of-1-url"></span><span id="response-inner-indexing-embedder-url"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-url"></span><span id="response-inner-one-of-1-indexing-embedder-url"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-url"></span><span id="response-inner-one-of-1-search-embedder-url"></span><span id="response-inner-search-embedder-one-of-1-url"></span><span id="response-inner-search-embedder-url"></span>        url:
          type:
            - string
            - 'null'
          description: &gt;-
            URL of the embedder API.


            - Mandatory for `rest`. Optional for `openAi` and `ollama`.

            - 🌱 When modified for `openAi``, embeddings are never regenerated

            - 🏗️ When modified for `ollama` and `rest`, embeddings are always
            regenerated

            - If targetting URL resolving to a non-global IP (such as
            `localhost`), make sure that
              `--experimental-allowed-ip-networks` allows it. For details on how use this parameter,
              refer to [this documentation](https://www.meilisearch.com/docs/learn/self_hosted/configure_meilisearch_at_launch#allow-requests-to-private-networks).
          example: http://localhost:11434/api/embeddings
<span id="response-additional-properties-inner-indexing-embedder-indexing-fragments"></span><span id="response-additional-properties-inner-indexing-embedder-indexingfragments"></span><span id="response-additional-properties-inner-indexing-embedder-one-of-1-indexing-fragments"></span><span id="response-additional-properties-inner-indexing-embedder-one-of-1-indexingfragments"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-indexing-fragments"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-indexingfragments"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-indexing-fragments"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-indexingfragments"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-indexing-fragments"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-indexingfragments"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-indexing-fragments"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-indexingfragments"></span><span id="response-additional-properties-inner-search-embedder-indexing-fragments"></span><span id="response-additional-properties-inner-search-embedder-indexingfragments"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-indexing-fragments"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-indexingfragments"></span><span id="response-inner-indexing-embedder-indexing-fragments"></span><span id="response-inner-indexing-embedder-indexingfragments"></span><span id="response-inner-indexing-embedder-one-of-1-indexing-fragments"></span><span id="response-inner-indexing-embedder-one-of-1-indexingfragments"></span><span id="response-inner-one-of-1-indexing-embedder-indexing-fragments"></span><span id="response-inner-one-of-1-indexing-embedder-indexingfragments"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-indexing-fragments"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-indexingfragments"></span><span id="response-inner-one-of-1-search-embedder-indexing-fragments"></span><span id="response-inner-one-of-1-search-embedder-indexingfragments"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-indexing-fragments"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-indexingfragments"></span><span id="response-inner-search-embedder-indexing-fragments"></span><span id="response-inner-search-embedder-indexingfragments"></span><span id="response-inner-search-embedder-one-of-1-indexing-fragments"></span><span id="response-inner-search-embedder-one-of-1-indexingfragments"></span>        indexingFragments:
          type:
            - object
            - 'null'
          description: &gt;-
            Fragments (with [Liquid](https://shopify.github.io/liquid/)) sent to
            the embedder at indexing time.


            For `rest` with multimodal; key is fragment name, value the payload.


            See also [Image search with multimodal
            embeddings](https://www.meilisearch.com/docs/learn/ai_powered_search/image_search_with_multimodal_embeddings)


            - Only available for `indexingEmbedder`, not `searchEmbedder`.

            - 🏗️ When a fragment is deleted by passing `null` to its name, the
            corresponding embeddings are removed from documents.

            - 🏗️ When a fragment is modified, the corresponding embeddings are
            regenerated if their rendered version changes.
          default: {}
<span id="response-additional-properties-inner-indexing-embedder-indexing-fragments-additional-properties"></span><span id="response-additional-properties-inner-indexing-embedder-one-of-1-indexing-fragments-additional-properties"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-indexing-fragments-additional-properties"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-indexing-fragments-additional-properties"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-indexing-fragments-additional-properties"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-indexing-fragments-additional-properties"></span><span id="response-additional-properties-inner-search-embedder-indexing-fragments-additional-properties"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-indexing-fragments-additional-properties"></span><span id="response-inner-indexing-embedder-indexing-fragments-additional-properties"></span><span id="response-inner-indexing-embedder-one-of-1-indexing-fragments-additional-properties"></span><span id="response-inner-one-of-1-indexing-embedder-indexing-fragments-additional-properties"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-indexing-fragments-additional-properties"></span><span id="response-inner-one-of-1-search-embedder-indexing-fragments-additional-properties"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-indexing-fragments-additional-properties"></span><span id="response-inner-search-embedder-indexing-fragments-additional-properties"></span><span id="response-inner-search-embedder-one-of-1-indexing-fragments-additional-properties"></span>          additionalProperties: {}
          propertyNames:
            type: string
<span id="response-additional-properties-inner-indexing-embedder-one-of-1-search-fragments"></span><span id="response-additional-properties-inner-indexing-embedder-one-of-1-searchfragments"></span><span id="response-additional-properties-inner-indexing-embedder-search-fragments"></span><span id="response-additional-properties-inner-indexing-embedder-searchfragments"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-search-fragments"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-searchfragments"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-search-fragments"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-searchfragments"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-search-fragments"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-searchfragments"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-search-fragments"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-searchfragments"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-search-fragments"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-searchfragments"></span><span id="response-additional-properties-inner-search-embedder-search-fragments"></span><span id="response-additional-properties-inner-search-embedder-searchfragments"></span><span id="response-inner-indexing-embedder-one-of-1-search-fragments"></span><span id="response-inner-indexing-embedder-one-of-1-searchfragments"></span><span id="response-inner-indexing-embedder-search-fragments"></span><span id="response-inner-indexing-embedder-searchfragments"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-search-fragments"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-searchfragments"></span><span id="response-inner-one-of-1-indexing-embedder-search-fragments"></span><span id="response-inner-one-of-1-indexing-embedder-searchfragments"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-search-fragments"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-searchfragments"></span><span id="response-inner-one-of-1-search-embedder-search-fragments"></span><span id="response-inner-one-of-1-search-embedder-searchfragments"></span><span id="response-inner-search-embedder-one-of-1-search-fragments"></span><span id="response-inner-search-embedder-one-of-1-searchfragments"></span><span id="response-inner-search-embedder-search-fragments"></span><span id="response-inner-search-embedder-searchfragments"></span>        searchFragments:
          type:
            - object
            - 'null'
          description: &gt;-
            Fragments (with [Liquid](https://shopify.github.io/liquid/)) sent to
            the embedder at search time.


            For `rest` with multimodal.


            See also [Image search with multimodal
            embeddings](https://www.meilisearch.com/docs/learn/ai_powered_search/image_search_with_multimodal_embeddings)


            - 🌱 Changing the value of this parameter never regenerates
            embeddings
          default: {}
<span id="response-additional-properties-inner-indexing-embedder-one-of-1-search-fragments-additional-properties"></span><span id="response-additional-properties-inner-indexing-embedder-search-fragments-additional-properties"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-search-fragments-additional-properties"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-search-fragments-additional-properties"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-search-fragments-additional-properties"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-search-fragments-additional-properties"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-search-fragments-additional-properties"></span><span id="response-additional-properties-inner-search-embedder-search-fragments-additional-properties"></span><span id="response-inner-indexing-embedder-one-of-1-search-fragments-additional-properties"></span><span id="response-inner-indexing-embedder-search-fragments-additional-properties"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-search-fragments-additional-properties"></span><span id="response-inner-one-of-1-indexing-embedder-search-fragments-additional-properties"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-search-fragments-additional-properties"></span><span id="response-inner-one-of-1-search-embedder-search-fragments-additional-properties"></span><span id="response-inner-search-embedder-one-of-1-search-fragments-additional-properties"></span><span id="response-inner-search-embedder-search-fragments-additional-properties"></span>          additionalProperties: {}
          propertyNames:
            type: string
<span id="response-additional-properties-inner-indexing-embedder-one-of-1-request"></span><span id="response-additional-properties-inner-indexing-embedder-request"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-request"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-request"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-request"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-request"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-request"></span><span id="response-additional-properties-inner-search-embedder-request"></span><span id="response-inner-indexing-embedder-one-of-1-request"></span><span id="response-inner-indexing-embedder-request"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-request"></span><span id="response-inner-one-of-1-indexing-embedder-request"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-request"></span><span id="response-inner-one-of-1-search-embedder-request"></span><span id="response-inner-search-embedder-one-of-1-request"></span><span id="response-inner-search-embedder-request"></span>        request:
          description: &gt;-
            Request body template for `rest` embedder.


            See also [Configure a REST
            embedder](https://www.meilisearch.com/docs/learn/ai_powered_search/configure_rest_embedder)


            - Use `"{{text}}"` for the input.

            - Mandatory for `rest`.

            - 🏗️ Changing the value of this parameter always regenerates
            embeddings
<span id="response-additional-properties-inner-indexing-embedder-one-of-1-response"></span><span id="response-additional-properties-inner-indexing-embedder-response"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-response"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-response"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-response"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-response"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-response"></span><span id="response-additional-properties-inner-search-embedder-response"></span><span id="response-inner-indexing-embedder-one-of-1-response"></span><span id="response-inner-indexing-embedder-response"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-response"></span><span id="response-inner-one-of-1-indexing-embedder-response"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-response"></span><span id="response-inner-one-of-1-search-embedder-response"></span><span id="response-inner-search-embedder-one-of-1-response"></span><span id="response-inner-search-embedder-response"></span>        response:
          description: &gt;-
            Response template for `rest` embedder.


            See also [Configure a REST
            embedder](https://www.meilisearch.com/docs/learn/ai_powered_search/configure_rest_embedder)


            - Use `"{{embedding}}"` where the embedding array is.

            - Mandatory for `rest`.
<span id="response-additional-properties-inner-indexing-embedder-headers"></span><span id="response-additional-properties-inner-indexing-embedder-one-of-1-headers"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-headers"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-headers"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-headers"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-headers"></span><span id="response-additional-properties-inner-search-embedder-headers"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-headers"></span><span id="response-inner-indexing-embedder-headers"></span><span id="response-inner-indexing-embedder-one-of-1-headers"></span><span id="response-inner-one-of-1-indexing-embedder-headers"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-headers"></span><span id="response-inner-one-of-1-search-embedder-headers"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-headers"></span><span id="response-inner-search-embedder-headers"></span><span id="response-inner-search-embedder-one-of-1-headers"></span>        headers:
          type:
            - object
            - 'null'
          description: &gt;-
            Extra HTTP headers sent to the embedder.


            - Available for `rest`.

            - 🌱 Changing the value of this parameter never regenerates
            embeddings
<span id="response-additional-properties-inner-indexing-embedder-headers-additional-properties"></span><span id="response-additional-properties-inner-indexing-embedder-one-of-1-headers-additional-properties"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-headers-additional-properties"></span><span id="response-additional-properties-inner-one-of-1-indexing-embedder-one-of-1-headers-additional-properties"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-headers-additional-properties"></span><span id="response-additional-properties-inner-one-of-1-search-embedder-one-of-1-headers-additional-properties"></span><span id="response-additional-properties-inner-search-embedder-headers-additional-properties"></span><span id="response-additional-properties-inner-search-embedder-one-of-1-headers-additional-properties"></span><span id="response-inner-indexing-embedder-headers-additional-properties"></span><span id="response-inner-indexing-embedder-one-of-1-headers-additional-properties"></span><span id="response-inner-one-of-1-indexing-embedder-headers-additional-properties"></span><span id="response-inner-one-of-1-indexing-embedder-one-of-1-headers-additional-properties"></span><span id="response-inner-one-of-1-search-embedder-headers-additional-properties"></span><span id="response-inner-one-of-1-search-embedder-one-of-1-headers-additional-properties"></span><span id="response-inner-search-embedder-headers-additional-properties"></span><span id="response-inner-search-embedder-one-of-1-headers-additional-properties"></span>          additionalProperties:
            type: string
          propertyNames:
            type: string
      additionalProperties: false
    DistributionShift:
      type: object
      description: &gt;-
        Describes the mean and sigma of distribution of embedding similarity in
        the embedding space.


        The intended use is to make the similarity score more comparable to the
        regular ranking score.

        This allows to correct effects where results are too "packed" around a
        certain value.
      required:
        - current_mean
        - current_sigma
      properties:
<span id="response-additional-properties-inner-distribution-current-mean"></span><span id="response-additional-properties-inner-distribution-one-of-1-current-mean"></span><span id="response-additional-properties-inner-one-of-1-distribution-current-mean"></span><span id="response-additional-properties-inner-one-of-1-distribution-one-of-1-current-mean"></span><span id="response-inner-distribution-current-mean"></span><span id="response-inner-distribution-one-of-1-current-mean"></span><span id="response-inner-one-of-1-distribution-current-mean"></span><span id="response-inner-one-of-1-distribution-one-of-1-current-mean"></span>        current_mean:
          type: number
          format: float
          description: &gt;-
            Value where the results are "packed".


            Similarity scores are translated so that they are packed around 0.5
            instead
<span id="response-additional-properties-inner-distribution-current-sigma"></span><span id="response-additional-properties-inner-distribution-one-of-1-current-sigma"></span><span id="response-additional-properties-inner-one-of-1-distribution-current-sigma"></span><span id="response-additional-properties-inner-one-of-1-distribution-one-of-1-current-sigma"></span><span id="response-inner-distribution-current-sigma"></span><span id="response-inner-distribution-one-of-1-current-sigma"></span><span id="response-inner-one-of-1-distribution-current-sigma"></span><span id="response-inner-one-of-1-distribution-one-of-1-current-sigma"></span>        current_sigma:
          type: number
          format: float
          description: &gt;-
            standard deviation of a similarity score.


            Set below 0.4 to make the results less packed around the mean, and
            above 0.4 to make them more packed.
    Code:
      type: string
      enum:
        - api_key_already_exists
        - api_key_not_found
        - index_scoped_api_key_with_global_action
        - bad_parameter
        - bad_request
        - database_size_limit_reached
        - document_not_found
        - dump_already_processing
        - dump_not_found
        - dump_process_failed
        - duplicate_index_found
        - immutable_api_key_actions
        - immutable_api_key_created_at
        - immutable_api_key_expires_at
        - immutable_api_key_indexes
        - immutable_api_key_key
        - immutable_api_key_uid
        - immutable_api_key_updated_at
        - immutable_index_created_at
        - immutable_index_updated_at
        - import_task_already_received
        - import_task_unknown_remote
        - receive_import_finished_unknown_remote
        - import_task_without_network_task
        - index_already_exists
        - index_creation_failed
        - index_not_found
        - index_primary_key_already_exists
        - index_primary_key_multiple_candidates_found
        - index_primary_key_no_candidate_found
        - internal
        - invalid_api_key
        - invalid_api_key_actions
        - invalid_api_key_description
        - invalid_api_key_expires_at
        - invalid_api_key_indexes
        - invalid_api_key_limit
        - invalid_api_key_name
        - invalid_api_key_offset
        - invalid_api_key_uid
        - invalid_content_type
        - invalid_document_csv_delimiter
        - invalid_document_fields
        - invalid_document_retrieve_vectors
        - missing_document_filter
        - missing_document_edition_function
        - inconsistent_document_change_headers
        - invalid_document_filter
        - invalid_document_sort
        - invalid_document_use_network
        - invalid_document_retrieve
        - invalid_document_geo_field
        - invalid_document_geojson_field
        - invalid_header_value
        - invalid_vector_dimensions
        - invalid_vectors_type
        - invalid_document_id
        - invalid_document_ids
        - invalid_document_limit
        - invalid_document_offset
        - invalid_search_embedder
        - invalid_similar_embedder
        - invalid_search_hybrid_query
        - invalid_index_limit
        - invalid_index_offset
        - invalid_index_primary_key
        - invalid_index_custom_metadata
        - invalid_skip_creation
        - invalid_index_uid
        - invalid_multi_search_facets
        - invalid_multi_search_facets_by_index
        - invalid_multi_search_facet_order
        - invalid_multi_search_query_personalization
        - invalid_multi_search_query_show_performance_details
        - invalid_multi_search_federated
        - invalid_multi_search_federation_options
        - invalid_multi_search_max_values_per_facet
        - invalid_multi_search_merge_facets
        - invalid_multi_search_query_facets
        - invalid_multi_search_distinct
        - invalid_multi_search_query_pagination
        - invalid_multi_search_query_ranking_rules
        - invalid_multi_search_query_position
        - invalid_multi_search_remote
        - invalid_multi_search_weight
        - invalid_network_leader
        - invalid_network_remotes
        - invalid_network_shards
        - invalid_network_self
        - invalid_network_search_api_key
        - invalid_network_write_api_key
        - invalid_network_url
        - invalid_search_attributes_to_search_on
        - invalid_search_attributes_to_crop
        - invalid_search_attributes_to_highlight
        - invalid_similar_attributes_to_retrieve
        - invalid_similar_retrieve_vectors
        - invalid_search_attributes_to_retrieve
        - invalid_search_ranking_score_threshold
        - invalid_similar_ranking_score_threshold
        - invalid_search_retrieve_vectors
        - invalid_search_crop_length
        - invalid_search_crop_marker
        - invalid_search_facets
        - invalid_search_semantic_ratio
        - invalid_search_locales
        - invalid_facet_search_exhaustive_facet_count
        - invalid_facet_search_facet_name
        - invalid_similar_id
        - invalid_search_filter
        - invalid_similar_filter
        - invalid_search_highlight_post_tag
        - invalid_search_highlight_pre_tag
        - invalid_search_hits_per_page
        - invalid_similar_limit
        - invalid_search_limit
        - invalid_search_matching_strategy
        - invalid_similar_offset
        - invalid_search_offset
        - invalid_search_page
        - invalid_search_q
        - invalid_facet_search_query
        - invalid_facet_search_name
        - facet_search_disabled
        - invalid_search_vector
        - invalid_search_media
        - invalid_search_show_matches_position
        - invalid_search_show_ranking_score
        - invalid_similar_show_ranking_score
        - invalid_search_show_ranking_score_details
        - invalid_search_show_performance_details
        - invalid_search_use_network
        - invalid_similar_show_ranking_score_details
        - invalid_similar_show_performance_details
        - invalid_search_sort
        - invalid_search_distinct
        - invalid_search_personalize
        - invalid_search_personalize_user_context
        - invalid_search_media_and_vector
        - invalid_settings_displayed_attributes
        - invalid_settings_distinct_attribute
        - invalid_settings_proximity_precision
        - invalid_settings_facet_search
        - invalid_settings_prefix_search
        - invalid_settings_faceting
        - invalid_settings_filterable_attributes
        - invalid_settings_foreign_keys
        - invalid_settings_pagination
        - invalid_settings_search_cutoff_ms
        - invalid_settings_embedders
        - invalid_settings_ranking_rules
        - invalid_settings_searchable_attributes
        - invalid_settings_sortable_attributes
        - invalid_settings_stop_words
        - invalid_settings_non_separator_tokens
        - invalid_settings_separator_tokens
        - invalid_settings_dictionary
        - invalid_settings_synonyms
        - invalid_settings_typo_tolerance
        - invalid_settings_localized_attributes
        - invalid_state
        - invalid_stats_show_internal_database_sizes
        - invalid_stats_size_format
        - invalid_store_file
        - invalid_swap_duplicate_index_found
        - invalid_swap_indexes
        - invalid_swap_rename
        - invalid_task_after_enqueued_at
        - invalid_task_after_finished_at
        - invalid_task_after_started_at
        - invalid_task_before_enqueued_at
        - invalid_task_before_finished_at
        - invalid_task_before_started_at
        - invalid_task_canceled_by
        - invalid_task_from
        - invalid_task_limit
        - invalid_task_reverse
        - invalid_task_statuses
        - invalid_task_types
        - invalid_task_uids
        - invalid_batch_uids
        - io_error
        - feature_not_enabled
        - malformed_payload
        - max_fields_limit_exceeded
        - missing_api_key_actions
        - missing_api_key_expires_at
        - missing_api_key_indexes
        - missing_authorization_header
        - missing_content_type
        - missing_document_id
        - missing_facet_search_facet_name
        - missing_index_uid
        - missing_master_key
        - missing_network_url
        - missing_payload
        - missing_search_hybrid
        - missing_swap_indexes
        - missing_task_filters
        - network_version_mismatch
        - no_space_left_on_device
        - not_leader
        - payload_too_large
        - remote_bad_response
        - remote_bad_request
        - unknown_remote
        - remote_could_not_send_request
        - remote_invalid_api_key
        - remote_remote_error
        - remote_timeout
        - too_many_search_requests
        - task_not_found
        - task_file_not_found
        - batch_not_found
        - too_many_open_files
        - too_many_vectors
        - unexpected_network_previous_remotes
        - network_version_too_old
        - unprocessed_network_task
        - unretrievable_document
        - unretrievable_error_code
        - unsupported_media_type
        - invalid_s3_snapshot_request
        - invalid_s3_snapshot_parameters
        - s3_snapshot_server_error
        - vector_embedding_error
        - not_found_similar_id
        - invalid_document_edition_context
        - invalid_document_edition_function_filter
        - edit_documents_by_function_error
        - invalid_settings_index_chat
        - invalid_export_url
        - invalid_export_api_key
        - invalid_export_payload_size
        - invalid_export_indexes_patterns
        - invalid_export_index_filter
        - invalid_export_index_override_settings
        - unimplemented_external_function_calling
        - unimplemented_non_streaming_chat_completions
        - unimplemented_multi_choice_chat_completions
        - chat_not_found
        - invalid_chat_setting_document_template
        - invalid_chat_completion_org_id
        - invalid_chat_completion_project_id
        - invalid_chat_completion_api_version
        - invalid_chat_completion_deployment_id
        - invalid_chat_completion_source
        - invalid_chat_completion_base_api
        - invalid_chat_completion_api_key
        - invalid_chat_completion_prompts
        - invalid_chat_completion_system_prompt
        - invalid_chat_completion_search_description_prompt
        - invalid_chat_completion_search_query_param_prompt
        - invalid_chat_completion_search_filter_param_prompt
        - invalid_chat_completion_search_index_uid_param_prompt
        - invalid_chat_completion_pre_query_prompt
        - invalid_index_fields_filter
        - invalid_index_fields_filter_attribute_patterns
        - invalid_index_fields_filter_displayed
        - invalid_index_fields_filter_searchable
        - invalid_index_fields_filter_sortable
        - invalid_index_fields_filter_distinct
        - invalid_index_fields_filter_ranking_rule
        - invalid_index_fields_filter_filterable
        - requires_enterprise_edition
        - invalid_render_template
        - invalid_render_input
        - render_document_not_found
        - template_parsing_error
        - template_rendering_error
        - invalid_webhooks
        - invalid_webhook_url
        - invalid_webhook_headers
        - immutable_webhook
        - invalid_webhook_uuid
        - webhook_not_found
        - immutable_webhook_uuid
        - immutable_webhook_is_editable
        - invalid_dynamic_search_rule_uid
        - invalid_dynamic_search_rule_offset
        - invalid_dynamic_search_rule_limit
        - invalid_dynamic_search_rule_filter
        - invalid_dynamic_search_rule_description
        - invalid_dynamic_search_rule_priority
        - invalid_dynamic_search_rule_active
        - invalid_dynamic_search_rule_conditions
        - invalid_dynamic_search_rule_actions
        - invalid_dynamic_search_rule_filter_query
        - invalid_dynamic_search_rule_filter_active
        - dynamic_search_rule_not_found
    ErrorType:
      type: string
      enum:
        - internal
        - invalid_request
        - auth
        - system
  securitySchemes:
    Bearer:
      type: http
      scheme: bearer
      bearerFormat: Uuidv4, string or JWT
      description: &gt;-
        An API key is a token that you provide when making API calls. Read more
        about [how to secure your
        project](https://www.meilisearch.com/docs/learn/security/basic_security).


        Include the API key to the `Authorization` header, for instance:

        ```bash

        -H 'Authorization: Bearer 6436fc5237b0d6e0d64253fbaac21d135012ecf1'

        ```


        If you use a SDK, ensure you instantiate the client with the API key,
        for instance with [JS
        SDK](https://github.com/meilisearch/meilisearch-js):

        ```js

        const client = new MeiliSearch({
          host: 'MEILISEARCH_URL',
          apiKey: '6436fc5237b0d6e0d64253fbaac21d135012ecf1'
        });

        ```

</code></pre>

This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.