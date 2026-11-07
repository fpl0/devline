# List all settings

> Returns all settings of the index. Each setting is returned with its current value or the default if not set.



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
  /indexes/{index_uid}/settings:
    get:
      tags:
        - Settings
      summary: List all settings
      description: &gt;-
        Returns all settings of the index. Each setting is returned with its
        current value or the default if not set.
      operationId: get_all
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
          description: &gt;-
            Returns all settings with their current or default values. Same
            structure as the PATCH request body.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Settings_Unchecked'
              example:
                displayedAttributes:
                  - id
                  - title
                  - description
                  - url
                searchableAttributes:
                  - title
                  - description
                filterableAttributes:
                  - release_date
                  - genre
                sortableAttributes:
                  - release_date
                rankingRules:
                  - words
                  - typo
                  - proximity
                  - attributeRank
                  - sort
                  - wordPosition
                  - exactness
                stopWords:
                  - the
                  - a
                nonSeparatorTokens:
                  - '@'
                  - '#'
                separatorTokens:
                  - '|'
                dictionary:
                  - J. R. R.
                synonyms:
                  phone:
                    - iPhone
                distinctAttribute: null
                typoTolerance:
                  enabled: true
                  minWordSizeForTypos:
                    oneTypo: 5
                    twoTypos: 9
                  disableOnWords: []
                  disableOnAttributes:
                    - title
                  disableOnNumbers: false
                faceting:
                  maxValuesPerFacet: 100
                  sortFacetValuesBy:
                    genre: count
                pagination:
                  maxTotalHits: 1000
                proximityPrecision: byWord
                embedders:
                  default:
                    source: openAi
                    model: text-embedding-3-small
                    documentTemplate: '{{doc.title}}: {{doc.overview}}'
                searchCutoffMs: null
                localizedAttributes:
                  - locales:
                      - jpn
                    attributePatterns:
                      - '*_ja'
                facetSearch: true
                prefixSearch: indexingTime
                chat:
                  description: A comprehensive movie database
                  documentTemplateMaxBytes: 400
                  searchParameters:
                    limit: 20
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
              -X GET 'MEILISEARCH_URL/indexes/movies/settings' \
              -H 'Authorization: Bearer API_KEY'
        - lang: JS
          source: client.index('movies').getSettings()
        - lang: PHP
          source: $client-&gt;index('movies')-&gt;getSettings();
        - lang: Python
          source: client.index('movies').get_settings()
        - lang: Java
          source: client.index("movies").getSettings();
        - lang: Ruby
          source: client.index('movies').settings
        - lang: Go
          source: client.Index("movies").GetSettings()
        - lang: C#
          source: await client.Index("movies").GetSettingsAsync();
        - lang: Rust
          source: |-
            let settings: Settings = client
              .index("movies")
              .get_settings()
              .await
              .unwrap();
        - lang: Dart
          source: await client.index('movies').getSettings();
        - lang: Swift
          source: |-
            client.index("movies").getSettings { (result) in
                switch result {
                case .success(let setting):
                    print(setting)
                case .failure(let error):
                    print(error)
                }
            }
components:
  schemas:
    Settings_Unchecked:
      type: object
      description: &gt;-
        Index settings: every option you can configure for search and index
        behavior.


        Used as the request body for PATCH settings. Only the fields you send
        are updated; pass `null` to reset a setting to its default.


        See also: [Configuring index settings on the
        Cloud](https://www.meilisearch.com/docs/learn/configuration/configuring_index_settings).
      properties:
<span id="response-displayed-attributes"></span><span id="response-displayedattributes"></span>        displayedAttributes:
          type:
            - array
            - 'null'
<span id="response-displayed-attributes-items"></span>          items:
            type: string
          description: &gt;-
            Fields returned in search results. Affects only search endpoints,
            not get-document endpoints. See [displayed and searchable
            attributes](https://www.meilisearch.com/docs/learn/relevancy/displayed_searchable_attributes).
          example:
            - id
            - title
            - description
            - url
          default:
            - '*'
<span id="response-searchable-attributes"></span><span id="response-searchableattributes"></span>        searchableAttributes:
          type:
            - array
            - 'null'
<span id="response-searchable-attributes-items"></span>          items:
            type: string
          description: &gt;-
            Fields searched for query words, in order of importance. Defines
            [attribute ranking
            order](https://www.meilisearch.com/docs/learn/relevancy/attribute_ranking_order).
            See [displayed and searchable
            attributes](https://www.meilisearch.com/docs/learn/relevancy/displayed_searchable_attributes).
          example:
            - title
            - description
          default:
            - '*'
<span id="response-filterable-attributes"></span><span id="response-filterableattributes"></span>        filterableAttributes:
          type:
            - array
            - 'null'
<span id="response-filterable-attributes-items"></span>          items:
            $ref: '#/components/schemas/FilterableAttributesRule'
          description: &gt;-
            Attributes that can be used as
            [filters](https://www.meilisearch.com/docs/learn/filtering_and_sorting/filter_search_results)
            and
            [facets](https://www.meilisearch.com/docs/learn/filtering_and_sorting/search_with_facet_filters).
            Strings or objects with `attributePatterns` and `features`.
          example:
            - genres
            - director
            - attributePatterns:
                - '*_ratings'
              features:
                facetSearch: false
                filter:
                  equality: true
                  comparison: true
          default: []
<span id="response-sortable-attributes"></span><span id="response-sortableattributes"></span>        sortableAttributes:
          type:
            - array
            - 'null'
<span id="response-sortable-attributes-items"></span>          items:
            type: string
          description: &gt;-
            Attributes that can be used to [sort search
            results](https://www.meilisearch.com/docs/learn/filtering_and_sorting/sort_search_results).
          example:
            - release_date
          default: []
<span id="response-foreign-keys"></span><span id="response-foreignkeys"></span>        foreignKeys:
          type:
            - array
            - 'null'
<span id="response-foreign-keys-items"></span>          items:
            $ref: '#/components/schemas/ForeignKey'
          description: Foreign keys to use for cross-index filtering search.
          example:
            - foreignIndexUid: products
              fieldName: productId
<span id="response-ranking-rules"></span><span id="response-rankingrules"></span>        rankingRules:
          type:
            - array
            - 'null'
<span id="response-ranking-rules-items"></span>          items:
            type: string
          description: &gt;-
            [Ranking
            rules](https://www.meilisearch.com/docs/learn/relevancy/ranking_rules)
            in order of importance. Built-in rules and custom sort rules
            (`attribute:asc` or `attribute:desc`).
          example:
            - words
            - typo
            - proximity
            - attributeRank
            - sort
            - wordPosition
            - exactness
          default:
            - words
            - typo
            - proximity
            - attributeRank
            - sort
            - wordPosition
            - exactness
<span id="response-stop-words"></span><span id="response-stopwords"></span>        stopWords:
          type:
            - array
            - 'null'
<span id="response-stop-words-items"></span>          items:
            type: string
          description: Words ignored when present in search queries.
          example:
            - the
            - a
          default: []
<span id="response-non-separator-tokens"></span><span id="response-nonseparatortokens"></span>        nonSeparatorTokens:
          type:
            - array
            - 'null'
<span id="response-non-separator-tokens-items"></span>          items:
            type: string
          description: &gt;-
            Characters that are not treated as word separators. Removed from the
            default separator set.
          example:
            - '@'
            - '#'
          default: []
<span id="response-separator-tokens"></span><span id="response-separatortokens"></span>        separatorTokens:
          type:
            - array
            - 'null'
<span id="response-separator-tokens-items"></span>          items:
            type: string
          description: &gt;-
            Characters that delimit words. Added on top of the default
            separators.
          example:
            - '|'
          default: []
<span id="response-dictionary"></span>        dictionary:
          type:
            - array
            - 'null'
<span id="response-dictionary-items"></span>          items:
            type: string
          description: &gt;-
            Strings Meilisearch parses as a single term. Useful for names or
            domain terms.
          example:
            - J. R. R.
          default: []
<span id="response-synonyms"></span>        synonyms:
          type:
            - object
            - 'null'
          description: &gt;-
            Pairs of words or phrases treated as equivalent for search. Key maps
            to an array of
            [synonyms](https://www.meilisearch.com/docs/learn/relevancy/synonyms).
          default: {}
<span id="response-synonyms-additional-properties"></span>          additionalProperties:
            type: array
<span id="response-synonyms-additional-properties-items"></span><span id="response-synonyms-items"></span>            items:
              type: string
          propertyNames:
            type: string
          example:
            phone:
              - iPhone
<span id="response-distinct-attribute"></span><span id="response-distinctattribute"></span>        distinctAttribute:
          type:
            - string
            - 'null'
          description: &gt;-
            Field whose value must be unique in the returned documents. One
            document per distinct value. See [distinct
            attribute](https://www.meilisearch.com/docs/learn/relevancy/distinct_attribute).
          example: sku
<span id="response-proximity-precision"></span><span id="response-proximityprecision"></span>        proximityPrecision:
          type:
            - string
            - 'null'
          description: &gt;-
            Precision for the proximity ranking rule and phrase search: `byWord`
            (exact distance) or `byAttribute` (same attribute).
          default: byWord
          example: byWord
<span id="response-typo-tolerance"></span><span id="response-typotolerance"></span>        typoTolerance:
          oneOf:
<span id="response-typo-tolerance-one-of-0"></span>            - type: 'null'
<span id="response-typo-tolerance-one-of-1"></span>            - $ref: '#/components/schemas/TypoSettings'
              description: &gt;-
                [Typo
                tolerance](https://www.meilisearch.com/docs/learn/relevancy/typo_tolerance_settings):
                enable/disable, minimum word length for typos, and where to
                disable it.
          default:
            enabled: true
            minWordSizeForTypos:
              oneTypo: 5
              twoTypos: 9
            disableOnWords: []
            disableOnAttributes: []
            disableOnNumbers: false
<span id="response-faceting"></span>        faceting:
          oneOf:
<span id="response-faceting-one-of-0"></span>            - type: 'null'
<span id="response-faceting-one-of-1"></span>            - $ref: '#/components/schemas/FacetingSettings'
              description: &gt;-
                Related to
                [faceting](https://www.meilisearch.com/docs/learn/filtering_and_sorting/search_with_facet_filters):
                max facet values per facet and how facet values are sorted.
          default:
            maxValuesPerFacet: 100
            sortFacetValuesBy:
              '*': alpha
<span id="response-pagination"></span>        pagination:
          oneOf:
<span id="response-pagination-one-of-0"></span>            - type: 'null'
<span id="response-pagination-one-of-1"></span>            - $ref: '#/components/schemas/PaginationSettings'
              description: &gt;-
                Related to
                [pagination](https://www.meilisearch.com/docs/guides/front_end/pagination):
                maximum number of results a search can return.
          default:
            maxTotalHits: 1000
<span id="response-embedders"></span>        embedders:
          type:
            - object
            - 'null'
          description: &gt;-
            [Embedders](https://www.meilisearch.com/docs/learn/ai_powered_search/getting_started_with_ai_search)
            used for semantic and [hybrid
            search](https://www.meilisearch.com/docs/learn/ai_powered_search/getting_started_with_ai_search).
            Map of embedder name to config (`source`, `model`,
            `documentTemplate`, etc.).
          default: {}
<span id="response-embedders-additional-properties"></span>          additionalProperties:
            $ref: '#/components/schemas/EmbeddingSettings'
          propertyNames:
            type: string
          example:
            default:
              source: openAi
              model: text-embedding-3-small
              documentTemplate: '{{doc.title}}: {{doc.overview}}'
<span id="response-search-cutoff-ms"></span><span id="response-searchcutoffms"></span>        searchCutoffMs:
          type:
            - integer
            - 'null'
          format: u-int64
          description: &gt;-
            Maximum duration of a search in milliseconds. If reached, the search
            stops and returns results computed so far. When null, 1500 ms is
            used.
          example: 1500
          minimum: 0
<span id="response-localized-attributes"></span><span id="response-localizedattributes"></span>        localizedAttributes:
          type:
            - array
            - 'null'
<span id="response-localized-attributes-items"></span>          items:
            $ref: '#/components/schemas/LocalizedAttributesRuleView'
          description: &gt;-
            Locales and attribute patterns for [language-specific
            tokenization](https://www.meilisearch.com/docs/learn/resources/language).
            Affects searchable, filterable, and sortable attributes.
          example:
            - locales:
                - jpn
              attributePatterns:
                - '*_ja'
          default: []
<span id="response-facet-search"></span><span id="response-facetsearch"></span>        facetSearch:
          type:
            - boolean
            - 'null'
          description: &gt;-
            When true, [facet
            search](https://www.meilisearch.com/docs/learn/filtering_and_sorting/search_with_facet_filters)
            is enabled. When false, the facet-search endpoint is disabled.
          default: true
          example: true
<span id="response-prefix-search"></span><span id="response-prefixsearch"></span>        prefixSearch:
          oneOf:
<span id="response-prefix-search-one-of-0"></span>            - type: 'null'
<span id="response-prefix-search-one-of-1"></span>            - $ref: '#/components/schemas/PrefixSearchSettings'
              description: &gt;-
                When to compute prefix matches: `indexingTime` or `disabled`.
                `disabled` speeds up indexing but reduces relevancy.
          default: indexingTime
<span id="response-chat"></span>        chat:
          oneOf:
<span id="response-chat-one-of-0"></span>            - type: 'null'
<span id="response-chat-one-of-1"></span>            - $ref: '#/components/schemas/ChatSettings'
              description: &gt;-
                [Chat
                (conversation)](https://www.meilisearch.com/docs/learn/chat/getting_started_with_chat)
                settings: index description, document template, and search
                parameters used when the LLM queries this index.
          default: {}
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
    FilterableAttributesRule:
      oneOf:
<span id="response-filterable-attributes-items-one-of-0"></span><span id="response-filterable-attributes-one-of-0"></span>        - $ref: '#/components/schemas/FilterableAttributesPatterns'
<span id="response-filterable-attributes-items-one-of-1"></span><span id="response-filterable-attributes-one-of-1"></span>        - type: string
      description: &gt;-
        A rule declaring which attributes can be used for filtering and
        faceting.


        A rule takes one of two forms. The object form lists attribute patterns

        together with the filtering and faceting features they enable. The
        string

        form names a single attribute and is a shorthand for an object matching
        that

        attribute with the default features.
    ForeignKey:
      type: object
      required:
        - foreignIndexUid
        - fieldName
      properties:
<span id="response-foreign-keys-foreign-index-uid"></span><span id="response-foreign-keys-foreignindexuid"></span><span id="response-foreign-keys-items-foreign-index-uid"></span><span id="response-foreign-keys-items-foreignindexuid"></span>        foreignIndexUid:
          type: string
<span id="response-foreign-keys-field-name"></span><span id="response-foreign-keys-fieldname"></span><span id="response-foreign-keys-items-field-name"></span><span id="response-foreign-keys-items-fieldname"></span>        fieldName:
          type: string
    TypoSettings:
      type: object
      description: 'Typo tolerance: how spelling mistakes in queries are handled.'
      properties:
<span id="response-typo-tolerance-enabled"></span><span id="response-typo-tolerance-one-of-1-enabled"></span>        enabled:
          type:
            - boolean
            - 'null'
          description: &gt;-
            When true, typo tolerance is enabled. When false, only exact matches
            are returned.
          default: true
          example: true
<span id="response-typo-tolerance-min-word-size-for-typos"></span><span id="response-typo-tolerance-minwordsizefortypos"></span><span id="response-typo-tolerance-one-of-1-min-word-size-for-typos"></span><span id="response-typo-tolerance-one-of-1-minwordsizefortypos"></span>        minWordSizeForTypos:
          oneOf:
<span id="response-typo-tolerance-min-word-size-for-typos-one-of-0"></span><span id="response-typo-tolerance-one-of-1-min-word-size-for-typos-one-of-0"></span>            - type: 'null'
<span id="response-typo-tolerance-min-word-size-for-typos-one-of-1"></span><span id="response-typo-tolerance-one-of-1-min-word-size-for-typos-one-of-1"></span>            - $ref: '#/components/schemas/MinWordSizeTyposSetting'
              description: &gt;-
                Minimum word length before typos are allowed. Contains `oneTypo`
                (min length for 1 typo) and `twoTypos` (min length for 2 typos).
                Example: `{ "oneTypo": 5, "twoTypos": 9 }`.
          default:
            oneTypo: 5
            twoTypos: 9
<span id="response-typo-tolerance-disable-on-words"></span><span id="response-typo-tolerance-disableonwords"></span><span id="response-typo-tolerance-one-of-1-disable-on-words"></span><span id="response-typo-tolerance-one-of-1-disableonwords"></span>        disableOnWords:
          type:
            - array
            - 'null'
<span id="response-typo-tolerance-disable-on-words-items"></span><span id="response-typo-tolerance-one-of-1-disable-on-words-items"></span>          items:
            type: string
          description: &gt;-
            Words for which typo tolerance is disabled. Use for brand names or
            terms that must match exactly.
          example:
            - iPhone
            - phone
          default: []
          uniqueItems: true
<span id="response-typo-tolerance-disable-on-attributes"></span><span id="response-typo-tolerance-disableonattributes"></span><span id="response-typo-tolerance-one-of-1-disable-on-attributes"></span><span id="response-typo-tolerance-one-of-1-disableonattributes"></span>        disableOnAttributes:
          type:
            - array
            - 'null'
<span id="response-typo-tolerance-disable-on-attributes-items"></span><span id="response-typo-tolerance-one-of-1-disable-on-attributes-items"></span>          items:
            type: string
          description: &gt;-
            Attributes for which typo tolerance is disabled. Those attributes
            only return exact matches. Useful for fields like product codes or
            IDs.
          example:
            - uuid
            - url
          default: []
          uniqueItems: true
<span id="response-typo-tolerance-disable-on-numbers"></span><span id="response-typo-tolerance-disableonnumbers"></span><span id="response-typo-tolerance-one-of-1-disable-on-numbers"></span><span id="response-typo-tolerance-one-of-1-disableonnumbers"></span>        disableOnNumbers:
          type:
            - boolean
            - 'null'
          description: &gt;-
            When true, typo tolerance is disabled on numeric tokens. For
            example, 123 will not match 132.
          default: false
          example: false
    FacetingSettings:
      type: object
      description: 'Faceting: maximum number of facet values and how they are sorted.'
      properties:
<span id="response-faceting-max-values-per-facet"></span><span id="response-faceting-maxvaluesperfacet"></span><span id="response-faceting-one-of-1-max-values-per-facet"></span><span id="response-faceting-one-of-1-maxvaluesperfacet"></span>        maxValuesPerFacet:
          type:
            - integer
            - 'null'
          description: &gt;-
            Maximum number of facet values returned per facet. Values are sorted
            in ascending lexicographical order.
          default: 100
          example: 100
          minimum: 0
<span id="response-faceting-one-of-1-sort-facet-values-by"></span><span id="response-faceting-one-of-1-sortfacetvaluesby"></span><span id="response-faceting-sort-facet-values-by"></span><span id="response-faceting-sortfacetvaluesby"></span>        sortFacetValuesBy:
          type:
            - object
            - 'null'
          description: &gt;-
            Sort order per facet: by descending count (`count`) or ascending
            alphanumeric (`alpha`). Key `*` applies to all facets.
          default:
            '*': alpha
<span id="response-faceting-one-of-1-sort-facet-values-by-additional-properties"></span><span id="response-faceting-sort-facet-values-by-additional-properties"></span>          additionalProperties:
            $ref: '#/components/schemas/FacetValuesSort'
          propertyNames:
            type: string
          example:
            '*': alpha
    PaginationSettings:
      type: object
      description: 'Pagination: cap on how many results a search can return.'
      properties:
<span id="response-pagination-max-total-hits"></span><span id="response-pagination-maxtotalhits"></span><span id="response-pagination-one-of-1-max-total-hits"></span><span id="response-pagination-one-of-1-maxtotalhits"></span>        maxTotalHits:
          type:
            - integer
            - 'null'
          description: &gt;-
            Maximum number of search results Meilisearch can return. Limit and
            offset cannot go beyond this value.
          default: 1000
          example: 1000
          minimum: 0
    EmbeddingSettings:
      type: object
      description: &gt;-
        Embedder configuration for [AI-powered / hybrid
        search](https://www.meilisearch.com/docs/learn/ai_powered_search/getting_started_with_ai_search).
      properties:
<span id="response-embedders-additional-properties-source"></span><span id="response-embedders-source"></span>        source:
          oneOf:
<span id="response-embedders-additional-properties-source-one-of-0"></span><span id="response-embedders-source-one-of-0"></span>            - type: 'null'
<span id="response-embedders-additional-properties-source-one-of-1"></span><span id="response-embedders-source-one-of-1"></span>            - $ref: '#/components/schemas/EmbedderSource'
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

                - `composite`: combine two embedding providers at indexing and
                search time.
                  The generated embeddings must be **exactly the same** regardless of the embedding provider.
                - `userProvided`: manually provide embeddings in documents.


                - Determines which other fields are required or allowed.

                - 🏗️ Changing the value of this parameter always regenerates
                embeddings.
          default: openAi
<span id="response-embedders-additional-properties-model"></span><span id="response-embedders-model"></span>        model:
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
<span id="response-embedders-additional-properties-revision"></span><span id="response-embedders-revision"></span>        revision:
          type:
            - string
            - 'null'
          description: &gt;-
            Model revision (e.g. Hugging Face commit SHA).


            - Only available for `huggingFace`.

            - If unset, latest is used.

            - 🏗️ Changing the value of this parameter always regenerates
            embeddings
<span id="response-embedders-additional-properties-pooling"></span><span id="response-embedders-pooling"></span>        pooling:
          oneOf:
<span id="response-embedders-additional-properties-pooling-one-of-0"></span><span id="response-embedders-pooling-one-of-0"></span>            - type: 'null'
<span id="response-embedders-additional-properties-pooling-one-of-1"></span><span id="response-embedders-pooling-one-of-1"></span>            - $ref: '#/components/schemas/OverridePooling'
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
<span id="response-embedders-additional-properties-api-key"></span><span id="response-embedders-additional-properties-apikey"></span><span id="response-embedders-api-key"></span><span id="response-embedders-apikey"></span>        apiKey:
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
<span id="response-embedders-additional-properties-dimensions"></span><span id="response-embedders-dimensions"></span>        dimensions:
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
<span id="response-embedders-additional-properties-binary-quantized"></span><span id="response-embedders-additional-properties-binaryquantized"></span><span id="response-embedders-binary-quantized"></span><span id="response-embedders-binaryquantized"></span>        binaryQuantized:
          type:
            - boolean
            - 'null'
          description: &gt;-
            When true, vectors are stored as 1-bit (smaller, faster, less
            precise).


            - Available for all sources.

            - Irreversible once enabled.

            - 🏗️ When set to true, embeddings are not regenerated, but they are
            binary quantized, which takes time.
<span id="response-embedders-additional-properties-document-template"></span><span id="response-embedders-additional-properties-documenttemplate"></span><span id="response-embedders-document-template"></span><span id="response-embedders-documenttemplate"></span>        documentTemplate:
          type:
            - string
            - 'null'
          description: &gt;-
            [Liquid template](https://shopify.github.io/liquid/) to build the
            text sent to the embedder for each document.


            - Available for `openAi`, `huggingFace`, `ollama`, `rest`.

            - 🏗️ When modified, embeddings are regenerated for documents whose
            rendering through the template produces a different text.
          example: '{{doc.title}}: {{doc.overview}}'
<span id="response-embedders-additional-properties-document-template-max-bytes"></span><span id="response-embedders-additional-properties-documenttemplatemaxbytes"></span><span id="response-embedders-document-template-max-bytes"></span><span id="response-embedders-documenttemplatemaxbytes"></span>        documentTemplateMaxBytes:
          type:
            - integer
            - 'null'
          description: &gt;-
            Max size in bytes of the rendered document template.


            Longer output is truncated.


            - Available for `openAi`, `huggingFace`, `ollama`, `rest`.

            - 🏗️ When increased, embeddings are regenerated for documents whose
            rendering through the template produces a different text.

            - 🌱 When decreased, embeddings are never regenerated
          default: 400
          example: 400
          minimum: 0
<span id="response-embedders-additional-properties-url"></span><span id="response-embedders-url"></span>        url:
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
<span id="response-embedders-additional-properties-indexing-fragments"></span><span id="response-embedders-additional-properties-indexingfragments"></span><span id="response-embedders-indexing-fragments"></span><span id="response-embedders-indexingfragments"></span>        indexingFragments:
          type:
            - object
            - 'null'
          description: &gt;-
            Fragments (with [Liquid](https://shopify.github.io/liquid/)) sent to
            the embedder at indexing time.


            For `rest` with multimodal; key is fragment name, value the payload.


            See also [Image search with multimodal
            embeddings](https://www.meilisearch.com/docs/learn/ai_powered_search/image_search_with_multimodal_embeddings)


            - 🏗️ When a fragment is deleted by passing `null` to its name, the
            corresponding embeddings are removed from documents.

            - 🏗️ When a fragment is modified, the corresponding embeddings are
            regenerated if their rendered version changes.
          default: {}
<span id="response-embedders-additional-properties-indexing-fragments-additional-properties"></span><span id="response-embedders-indexing-fragments-additional-properties"></span>          additionalProperties: {}
          propertyNames:
            type: string
<span id="response-embedders-additional-properties-search-fragments"></span><span id="response-embedders-additional-properties-searchfragments"></span><span id="response-embedders-search-fragments"></span><span id="response-embedders-searchfragments"></span>        searchFragments:
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
<span id="response-embedders-additional-properties-search-fragments-additional-properties"></span><span id="response-embedders-search-fragments-additional-properties"></span>          additionalProperties: {}
          propertyNames:
            type: string
<span id="response-embedders-additional-properties-request"></span><span id="response-embedders-request"></span>        request:
          description: &gt;-
            Request body template for `rest` embedder.


            See also [Configure a REST
            embedder](https://www.meilisearch.com/docs/learn/ai_powered_search/configure_rest_embedder)


            - Use `"{{text}}"` for the input.

            - Mandatory for `rest`.

            - 🏗️ Changing the value of this parameter always regenerates
            embeddings
<span id="response-embedders-additional-properties-response"></span><span id="response-embedders-response"></span>        response:
          description: &gt;-
            Response template for `rest` embedder.


            See also [Configure a REST
            embedder](https://www.meilisearch.com/docs/learn/ai_powered_search/configure_rest_embedder)


            - Use `"{{embedding}}"` where the embedding array is.

            - Mandatory for `rest`.
<span id="response-embedders-additional-properties-headers"></span><span id="response-embedders-headers"></span>        headers:
          type:
            - object
            - 'null'
          description: &gt;-
            Extra HTTP headers sent to the embedder.


            - Available for `rest`.

            - 🌱 Changing the value of this parameter never regenerates
            embeddings
<span id="response-embedders-additional-properties-headers-additional-properties"></span><span id="response-embedders-headers-additional-properties"></span>          additionalProperties:
            type: string
          propertyNames:
            type: string
<span id="response-embedders-additional-properties-search-embedder"></span><span id="response-embedders-additional-properties-searchembedder"></span><span id="response-embedders-search-embedder"></span><span id="response-embedders-searchembedder"></span>        searchEmbedder:
          oneOf:
<span id="response-embedders-additional-properties-search-embedder-one-of-0"></span><span id="response-embedders-search-embedder-one-of-0"></span>            - type: 'null'
<span id="response-embedders-additional-properties-search-embedder-one-of-1"></span><span id="response-embedders-search-embedder-one-of-1"></span>            - $ref: '#/components/schemas/SubEmbeddingSettings'
              description: &gt;-
                Embedder used at search time when using a composite embedder.


                Same sub-parameters as a top-level embedder (`source`, `model`,
                `url`, etc.); no `documentTemplate`/`documentTemplateMaxBytes`.
<span id="response-embedders-additional-properties-indexing-embedder"></span><span id="response-embedders-additional-properties-indexingembedder"></span><span id="response-embedders-indexing-embedder"></span><span id="response-embedders-indexingembedder"></span>        indexingEmbedder:
          oneOf:
<span id="response-embedders-additional-properties-indexing-embedder-one-of-0"></span><span id="response-embedders-indexing-embedder-one-of-0"></span>            - type: 'null'
<span id="response-embedders-additional-properties-indexing-embedder-one-of-1"></span><span id="response-embedders-indexing-embedder-one-of-1"></span>            - $ref: '#/components/schemas/SubEmbeddingSettings'
              description: &gt;-
                Embedder used at indexing time when using a composite embedder.


                Same sub-parameters as a top-level embedder (`source`, `model`,
                `documentTemplate`, `url`, etc.).
<span id="response-embedders-additional-properties-distribution"></span><span id="response-embedders-distribution"></span>        distribution:
          oneOf:
<span id="response-embedders-additional-properties-distribution-one-of-0"></span><span id="response-embedders-distribution-one-of-0"></span>            - type: 'null'
<span id="response-embedders-additional-properties-distribution-one-of-1"></span><span id="response-embedders-distribution-one-of-1"></span>            - $ref: '#/components/schemas/DistributionShift'
              description: &gt;-
                Affine transform (mean, sigma) applied to the semantic score to
                make it more comparable with the ranking score.


                See also [this
                article](https://www.meilisearch.com/blog/fixing-hybrid-search#making-the-scores-comparable).


                - Available for all sources.

                - 🌱 Changing the value of this parameter never regenerates
                embeddings
    LocalizedAttributesRuleView:
      type: object
      description: &gt;-
        Defines a rule for associating specific locales (languages) with

        attributes. This allows Meilisearch to use language-specific
        tokenization

        and processing for matched attributes, improving search quality for

        multilingual content.
      required:
        - attributePatterns
        - locales
      properties:
<span id="response-localized-attributes-attribute-patterns"></span><span id="response-localized-attributes-attributepatterns"></span><span id="response-localized-attributes-items-attribute-patterns"></span><span id="response-localized-attributes-items-attributepatterns"></span>        attributePatterns:
          type: array
<span id="response-localized-attributes-attribute-patterns-items"></span><span id="response-localized-attributes-items-attribute-patterns-items"></span>          items:
            type: string
          description: &gt;-
            Patterns to match attribute names. Use `*` as a wildcard to match
            any

            characters. For example, `["title_*", "description"]` matches

            `title_en`, `title_fr`, and `description`.
          example:
            - '*_ja'
<span id="response-localized-attributes-items-locales"></span><span id="response-localized-attributes-locales"></span>        locales:
          type: array
<span id="response-localized-attributes-items-locales-items"></span><span id="response-localized-attributes-locales-items"></span>          items:
            $ref: '#/components/schemas/Locale'
          description: &gt;-
            The list of locales (languages) to apply to matching attributes.
            When

            these attributes are indexed, Meilisearch will use language-specific

            tokenization rules. Examples: `["en", "fr"]` or `["jpn", "zho"]`.
    PrefixSearchSettings:
      type: string
      enum:
        - indexingTime
        - disabled
    ChatSettings:
      type: object
      description: &gt;-
        [Chat
        (conversation)](https://www.meilisearch.com/docs/learn/chat/getting_started_with_chat)
        settings: how the index is described to the LLM and how it is queried.
      properties:
<span id="response-chat-description"></span><span id="response-chat-one-of-1-description"></span>        description:
          type:
            - string
            - 'null'
          description: &gt;-
            Index description shown to the LLM so it can decide when and how to
            query this index.
          default: ''
          example: &gt;-
            A comprehensive movie database containing titles, overviews, genres,
            and release dates
<span id="response-chat-document-template"></span><span id="response-chat-documenttemplate"></span><span id="response-chat-one-of-1-document-template"></span><span id="response-chat-one-of-1-documenttemplate"></span>        documentTemplate:
          type:
            - string
            - 'null'
          description: &gt;-
            Liquid template that defines the text sent to the LLM for each
            document.
          example: &gt;-
            {% for field in fields %}{% if field.is_searchable and field.value
            != nil %}{{ field.name }}: {{ field.value }}

            {% endif %}{% endfor %}
<span id="response-chat-document-template-max-bytes"></span><span id="response-chat-documenttemplatemaxbytes"></span><span id="response-chat-one-of-1-document-template-max-bytes"></span><span id="response-chat-one-of-1-documenttemplatemaxbytes"></span>        documentTemplateMaxBytes:
          type:
            - integer
            - 'null'
          description: &gt;-
            Maximum size in bytes of the rendered document template. Longer text
            is truncated.
          default: 400
          example: 400
          minimum: 0
<span id="response-chat-one-of-1-search-parameters"></span><span id="response-chat-one-of-1-searchparameters"></span><span id="response-chat-search-parameters"></span><span id="response-chat-searchparameters"></span>        searchParameters:
          oneOf:
<span id="response-chat-one-of-1-search-parameters-one-of-0"></span><span id="response-chat-search-parameters-one-of-0"></span>            - type: 'null'
<span id="response-chat-one-of-1-search-parameters-one-of-1"></span><span id="response-chat-search-parameters-one-of-1"></span>            - $ref: '#/components/schemas/ChatSearchParams'
              description: &gt;-
                Search parameters used when the LLM queries this index
                (`hybrid`, `limit`, `sort`, `distinct`, etc.).
          default: {}
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
    FilterableAttributesPatterns:
      type: object
      description: |-
        Defines a set of attribute patterns with specific filtering and faceting
        features. This allows fine-grained control over which operations are
        allowed on matched attributes.
      required:
        - attributePatterns
      properties:
<span id="response-filterable-attributes-attribute-patterns"></span><span id="response-filterable-attributes-attributepatterns"></span><span id="response-filterable-attributes-items-attribute-patterns"></span><span id="response-filterable-attributes-items-attributepatterns"></span><span id="response-filterable-attributes-items-one-of-0-attribute-patterns"></span><span id="response-filterable-attributes-items-one-of-0-attributepatterns"></span><span id="response-filterable-attributes-one-of-0-attribute-patterns"></span><span id="response-filterable-attributes-one-of-0-attributepatterns"></span>        attributePatterns:
          type: array
<span id="response-filterable-attributes-attribute-patterns-items"></span><span id="response-filterable-attributes-items-attribute-patterns-items"></span><span id="response-filterable-attributes-items-one-of-0-attribute-patterns-items"></span><span id="response-filterable-attributes-one-of-0-attribute-patterns-items"></span>          items:
            type: string
          description: &gt;-
            Patterns to match attribute names. Use `*` as a wildcard to match
            any

            characters. For example, `["price_*", "stock"]` matches `price_usd`,

            `price_eur`, and `stock`.
<span id="response-filterable-attributes-features"></span><span id="response-filterable-attributes-items-features"></span><span id="response-filterable-attributes-items-one-of-0-features"></span><span id="response-filterable-attributes-one-of-0-features"></span>        features:
          $ref: '#/components/schemas/FilterableAttributesFeatures'
          description: |-
            The filtering and faceting features enabled for attributes matching
            these patterns. If not specified, defaults to equality filtering
            enabled.
      additionalProperties: false
    MinWordSizeTyposSetting:
      type: object
      description: &gt;-
        Minimum word length required before typos are allowed.


        This helps prevent matching very short words with typos, which can lead
        to irrelevant results.
      properties:
<span id="response-typo-tolerance-min-word-size-for-typos-one-of-1-one-typo"></span><span id="response-typo-tolerance-min-word-size-for-typos-one-of-1-onetypo"></span><span id="response-typo-tolerance-min-word-size-for-typos-one-typo"></span><span id="response-typo-tolerance-min-word-size-for-typos-onetypo"></span><span id="response-typo-tolerance-one-of-1-min-word-size-for-typos-one-of-1-one-typo"></span><span id="response-typo-tolerance-one-of-1-min-word-size-for-typos-one-of-1-onetypo"></span><span id="response-typo-tolerance-one-of-1-min-word-size-for-typos-one-typo"></span><span id="response-typo-tolerance-one-of-1-min-word-size-for-typos-onetypo"></span>        oneTypo:
          type:
            - integer
            - 'null'
          format: u-int8
          description: &gt;-
            Minimum word length to accept one typo. Shorter words must match
            exactly. For example, if set to 5, "apple" can have one typo but
            "app" cannot.
          default: 5
          example: 5
          minimum: 0
<span id="response-typo-tolerance-min-word-size-for-typos-one-of-1-two-typos"></span><span id="response-typo-tolerance-min-word-size-for-typos-one-of-1-twotypos"></span><span id="response-typo-tolerance-min-word-size-for-typos-two-typos"></span><span id="response-typo-tolerance-min-word-size-for-typos-twotypos"></span><span id="response-typo-tolerance-one-of-1-min-word-size-for-typos-one-of-1-two-typos"></span><span id="response-typo-tolerance-one-of-1-min-word-size-for-typos-one-of-1-twotypos"></span><span id="response-typo-tolerance-one-of-1-min-word-size-for-typos-two-typos"></span><span id="response-typo-tolerance-one-of-1-min-word-size-for-typos-twotypos"></span>        twoTypos:
          type:
            - integer
            - 'null'
          format: u-int8
          description: &gt;-
            Minimum word length to accept two typos. Must be greater than or
            equal to `oneTypo`. For example, if set to 9, "computing" can have
            two typos.
          default: 9
          example: 9
          minimum: 0
    FacetValuesSort:
      type: string
      enum:
        - alpha
        - count
        - sampledCount
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
<span id="response-embedders-additional-properties-indexing-embedder-one-of-1-source"></span><span id="response-embedders-additional-properties-indexing-embedder-source"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-source"></span><span id="response-embedders-additional-properties-search-embedder-source"></span><span id="response-embedders-indexing-embedder-one-of-1-source"></span><span id="response-embedders-indexing-embedder-source"></span><span id="response-embedders-search-embedder-one-of-1-source"></span><span id="response-embedders-search-embedder-source"></span>        source:
          oneOf:
<span id="response-embedders-additional-properties-indexing-embedder-one-of-1-source-one-of-0"></span><span id="response-embedders-additional-properties-indexing-embedder-source-one-of-0"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-source-one-of-0"></span><span id="response-embedders-additional-properties-search-embedder-source-one-of-0"></span><span id="response-embedders-indexing-embedder-one-of-1-source-one-of-0"></span><span id="response-embedders-indexing-embedder-source-one-of-0"></span><span id="response-embedders-search-embedder-one-of-1-source-one-of-0"></span><span id="response-embedders-search-embedder-source-one-of-0"></span>            - type: 'null'
<span id="response-embedders-additional-properties-indexing-embedder-one-of-1-source-one-of-1"></span><span id="response-embedders-additional-properties-indexing-embedder-source-one-of-1"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-source-one-of-1"></span><span id="response-embedders-additional-properties-search-embedder-source-one-of-1"></span><span id="response-embedders-indexing-embedder-one-of-1-source-one-of-1"></span><span id="response-embedders-indexing-embedder-source-one-of-1"></span><span id="response-embedders-search-embedder-one-of-1-source-one-of-1"></span><span id="response-embedders-search-embedder-source-one-of-1"></span>            - $ref: '#/components/schemas/EmbedderSource'
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
<span id="response-embedders-additional-properties-indexing-embedder-model"></span><span id="response-embedders-additional-properties-indexing-embedder-one-of-1-model"></span><span id="response-embedders-additional-properties-search-embedder-model"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-model"></span><span id="response-embedders-indexing-embedder-model"></span><span id="response-embedders-indexing-embedder-one-of-1-model"></span><span id="response-embedders-search-embedder-model"></span><span id="response-embedders-search-embedder-one-of-1-model"></span>        model:
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
<span id="response-embedders-additional-properties-indexing-embedder-one-of-1-revision"></span><span id="response-embedders-additional-properties-indexing-embedder-revision"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-revision"></span><span id="response-embedders-additional-properties-search-embedder-revision"></span><span id="response-embedders-indexing-embedder-one-of-1-revision"></span><span id="response-embedders-indexing-embedder-revision"></span><span id="response-embedders-search-embedder-one-of-1-revision"></span><span id="response-embedders-search-embedder-revision"></span>        revision:
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
<span id="response-embedders-additional-properties-indexing-embedder-one-of-1-pooling"></span><span id="response-embedders-additional-properties-indexing-embedder-pooling"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-pooling"></span><span id="response-embedders-additional-properties-search-embedder-pooling"></span><span id="response-embedders-indexing-embedder-one-of-1-pooling"></span><span id="response-embedders-indexing-embedder-pooling"></span><span id="response-embedders-search-embedder-one-of-1-pooling"></span><span id="response-embedders-search-embedder-pooling"></span>        pooling:
          oneOf:
<span id="response-embedders-additional-properties-indexing-embedder-one-of-1-pooling-one-of-0"></span><span id="response-embedders-additional-properties-indexing-embedder-pooling-one-of-0"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-pooling-one-of-0"></span><span id="response-embedders-additional-properties-search-embedder-pooling-one-of-0"></span><span id="response-embedders-indexing-embedder-one-of-1-pooling-one-of-0"></span><span id="response-embedders-indexing-embedder-pooling-one-of-0"></span><span id="response-embedders-search-embedder-one-of-1-pooling-one-of-0"></span><span id="response-embedders-search-embedder-pooling-one-of-0"></span>            - type: 'null'
<span id="response-embedders-additional-properties-indexing-embedder-one-of-1-pooling-one-of-1"></span><span id="response-embedders-additional-properties-indexing-embedder-pooling-one-of-1"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-pooling-one-of-1"></span><span id="response-embedders-additional-properties-search-embedder-pooling-one-of-1"></span><span id="response-embedders-indexing-embedder-one-of-1-pooling-one-of-1"></span><span id="response-embedders-indexing-embedder-pooling-one-of-1"></span><span id="response-embedders-search-embedder-one-of-1-pooling-one-of-1"></span><span id="response-embedders-search-embedder-pooling-one-of-1"></span>            - $ref: '#/components/schemas/OverridePooling'
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
<span id="response-embedders-additional-properties-indexing-embedder-api-key"></span><span id="response-embedders-additional-properties-indexing-embedder-apikey"></span><span id="response-embedders-additional-properties-indexing-embedder-one-of-1-api-key"></span><span id="response-embedders-additional-properties-indexing-embedder-one-of-1-apikey"></span><span id="response-embedders-additional-properties-search-embedder-api-key"></span><span id="response-embedders-additional-properties-search-embedder-apikey"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-api-key"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-apikey"></span><span id="response-embedders-indexing-embedder-api-key"></span><span id="response-embedders-indexing-embedder-apikey"></span><span id="response-embedders-indexing-embedder-one-of-1-api-key"></span><span id="response-embedders-indexing-embedder-one-of-1-apikey"></span><span id="response-embedders-search-embedder-api-key"></span><span id="response-embedders-search-embedder-apikey"></span><span id="response-embedders-search-embedder-one-of-1-api-key"></span><span id="response-embedders-search-embedder-one-of-1-apikey"></span>        apiKey:
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
<span id="response-embedders-additional-properties-indexing-embedder-dimensions"></span><span id="response-embedders-additional-properties-indexing-embedder-one-of-1-dimensions"></span><span id="response-embedders-additional-properties-search-embedder-dimensions"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-dimensions"></span><span id="response-embedders-indexing-embedder-dimensions"></span><span id="response-embedders-indexing-embedder-one-of-1-dimensions"></span><span id="response-embedders-search-embedder-dimensions"></span><span id="response-embedders-search-embedder-one-of-1-dimensions"></span>        dimensions:
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
<span id="response-embedders-additional-properties-indexing-embedder-document-template"></span><span id="response-embedders-additional-properties-indexing-embedder-documenttemplate"></span><span id="response-embedders-additional-properties-indexing-embedder-one-of-1-document-template"></span><span id="response-embedders-additional-properties-indexing-embedder-one-of-1-documenttemplate"></span><span id="response-embedders-additional-properties-search-embedder-document-template"></span><span id="response-embedders-additional-properties-search-embedder-documenttemplate"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-document-template"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-documenttemplate"></span><span id="response-embedders-indexing-embedder-document-template"></span><span id="response-embedders-indexing-embedder-documenttemplate"></span><span id="response-embedders-indexing-embedder-one-of-1-document-template"></span><span id="response-embedders-indexing-embedder-one-of-1-documenttemplate"></span><span id="response-embedders-search-embedder-document-template"></span><span id="response-embedders-search-embedder-documenttemplate"></span><span id="response-embedders-search-embedder-one-of-1-document-template"></span><span id="response-embedders-search-embedder-one-of-1-documenttemplate"></span>        documentTemplate:
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
<span id="response-embedders-additional-properties-indexing-embedder-document-template-max-bytes"></span><span id="response-embedders-additional-properties-indexing-embedder-documenttemplatemaxbytes"></span><span id="response-embedders-additional-properties-indexing-embedder-one-of-1-document-template-max-bytes"></span><span id="response-embedders-additional-properties-indexing-embedder-one-of-1-documenttemplatemaxbytes"></span><span id="response-embedders-additional-properties-search-embedder-document-template-max-bytes"></span><span id="response-embedders-additional-properties-search-embedder-documenttemplatemaxbytes"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-document-template-max-bytes"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-documenttemplatemaxbytes"></span><span id="response-embedders-indexing-embedder-document-template-max-bytes"></span><span id="response-embedders-indexing-embedder-documenttemplatemaxbytes"></span><span id="response-embedders-indexing-embedder-one-of-1-document-template-max-bytes"></span><span id="response-embedders-indexing-embedder-one-of-1-documenttemplatemaxbytes"></span><span id="response-embedders-search-embedder-document-template-max-bytes"></span><span id="response-embedders-search-embedder-documenttemplatemaxbytes"></span><span id="response-embedders-search-embedder-one-of-1-document-template-max-bytes"></span><span id="response-embedders-search-embedder-one-of-1-documenttemplatemaxbytes"></span>        documentTemplateMaxBytes:
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
<span id="response-embedders-additional-properties-indexing-embedder-one-of-1-url"></span><span id="response-embedders-additional-properties-indexing-embedder-url"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-url"></span><span id="response-embedders-additional-properties-search-embedder-url"></span><span id="response-embedders-indexing-embedder-one-of-1-url"></span><span id="response-embedders-indexing-embedder-url"></span><span id="response-embedders-search-embedder-one-of-1-url"></span><span id="response-embedders-search-embedder-url"></span>        url:
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
<span id="response-embedders-additional-properties-indexing-embedder-indexing-fragments"></span><span id="response-embedders-additional-properties-indexing-embedder-indexingfragments"></span><span id="response-embedders-additional-properties-indexing-embedder-one-of-1-indexing-fragments"></span><span id="response-embedders-additional-properties-indexing-embedder-one-of-1-indexingfragments"></span><span id="response-embedders-additional-properties-search-embedder-indexing-fragments"></span><span id="response-embedders-additional-properties-search-embedder-indexingfragments"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-indexing-fragments"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-indexingfragments"></span><span id="response-embedders-indexing-embedder-indexing-fragments"></span><span id="response-embedders-indexing-embedder-indexingfragments"></span><span id="response-embedders-indexing-embedder-one-of-1-indexing-fragments"></span><span id="response-embedders-indexing-embedder-one-of-1-indexingfragments"></span><span id="response-embedders-search-embedder-indexing-fragments"></span><span id="response-embedders-search-embedder-indexingfragments"></span><span id="response-embedders-search-embedder-one-of-1-indexing-fragments"></span><span id="response-embedders-search-embedder-one-of-1-indexingfragments"></span>        indexingFragments:
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
<span id="response-embedders-additional-properties-indexing-embedder-indexing-fragments-additional-properties"></span><span id="response-embedders-additional-properties-indexing-embedder-one-of-1-indexing-fragments-additional-properties"></span><span id="response-embedders-additional-properties-search-embedder-indexing-fragments-additional-properties"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-indexing-fragments-additional-properties"></span><span id="response-embedders-indexing-embedder-indexing-fragments-additional-properties"></span><span id="response-embedders-indexing-embedder-one-of-1-indexing-fragments-additional-properties"></span><span id="response-embedders-search-embedder-indexing-fragments-additional-properties"></span><span id="response-embedders-search-embedder-one-of-1-indexing-fragments-additional-properties"></span>          additionalProperties: {}
          propertyNames:
            type: string
<span id="response-embedders-additional-properties-indexing-embedder-one-of-1-search-fragments"></span><span id="response-embedders-additional-properties-indexing-embedder-one-of-1-searchfragments"></span><span id="response-embedders-additional-properties-indexing-embedder-search-fragments"></span><span id="response-embedders-additional-properties-indexing-embedder-searchfragments"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-search-fragments"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-searchfragments"></span><span id="response-embedders-additional-properties-search-embedder-search-fragments"></span><span id="response-embedders-additional-properties-search-embedder-searchfragments"></span><span id="response-embedders-indexing-embedder-one-of-1-search-fragments"></span><span id="response-embedders-indexing-embedder-one-of-1-searchfragments"></span><span id="response-embedders-indexing-embedder-search-fragments"></span><span id="response-embedders-indexing-embedder-searchfragments"></span><span id="response-embedders-search-embedder-one-of-1-search-fragments"></span><span id="response-embedders-search-embedder-one-of-1-searchfragments"></span><span id="response-embedders-search-embedder-search-fragments"></span><span id="response-embedders-search-embedder-searchfragments"></span>        searchFragments:
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
<span id="response-embedders-additional-properties-indexing-embedder-one-of-1-search-fragments-additional-properties"></span><span id="response-embedders-additional-properties-indexing-embedder-search-fragments-additional-properties"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-search-fragments-additional-properties"></span><span id="response-embedders-additional-properties-search-embedder-search-fragments-additional-properties"></span><span id="response-embedders-indexing-embedder-one-of-1-search-fragments-additional-properties"></span><span id="response-embedders-indexing-embedder-search-fragments-additional-properties"></span><span id="response-embedders-search-embedder-one-of-1-search-fragments-additional-properties"></span><span id="response-embedders-search-embedder-search-fragments-additional-properties"></span>          additionalProperties: {}
          propertyNames:
            type: string
<span id="response-embedders-additional-properties-indexing-embedder-one-of-1-request"></span><span id="response-embedders-additional-properties-indexing-embedder-request"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-request"></span><span id="response-embedders-additional-properties-search-embedder-request"></span><span id="response-embedders-indexing-embedder-one-of-1-request"></span><span id="response-embedders-indexing-embedder-request"></span><span id="response-embedders-search-embedder-one-of-1-request"></span><span id="response-embedders-search-embedder-request"></span>        request:
          description: &gt;-
            Request body template for `rest` embedder.


            See also [Configure a REST
            embedder](https://www.meilisearch.com/docs/learn/ai_powered_search/configure_rest_embedder)


            - Use `"{{text}}"` for the input.

            - Mandatory for `rest`.

            - 🏗️ Changing the value of this parameter always regenerates
            embeddings
<span id="response-embedders-additional-properties-indexing-embedder-one-of-1-response"></span><span id="response-embedders-additional-properties-indexing-embedder-response"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-response"></span><span id="response-embedders-additional-properties-search-embedder-response"></span><span id="response-embedders-indexing-embedder-one-of-1-response"></span><span id="response-embedders-indexing-embedder-response"></span><span id="response-embedders-search-embedder-one-of-1-response"></span><span id="response-embedders-search-embedder-response"></span>        response:
          description: &gt;-
            Response template for `rest` embedder.


            See also [Configure a REST
            embedder](https://www.meilisearch.com/docs/learn/ai_powered_search/configure_rest_embedder)


            - Use `"{{embedding}}"` where the embedding array is.

            - Mandatory for `rest`.
<span id="response-embedders-additional-properties-indexing-embedder-headers"></span><span id="response-embedders-additional-properties-indexing-embedder-one-of-1-headers"></span><span id="response-embedders-additional-properties-search-embedder-headers"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-headers"></span><span id="response-embedders-indexing-embedder-headers"></span><span id="response-embedders-indexing-embedder-one-of-1-headers"></span><span id="response-embedders-search-embedder-headers"></span><span id="response-embedders-search-embedder-one-of-1-headers"></span>        headers:
          type:
            - object
            - 'null'
          description: &gt;-
            Extra HTTP headers sent to the embedder.


            - Available for `rest`.

            - 🌱 Changing the value of this parameter never regenerates
            embeddings
<span id="response-embedders-additional-properties-indexing-embedder-headers-additional-properties"></span><span id="response-embedders-additional-properties-indexing-embedder-one-of-1-headers-additional-properties"></span><span id="response-embedders-additional-properties-search-embedder-headers-additional-properties"></span><span id="response-embedders-additional-properties-search-embedder-one-of-1-headers-additional-properties"></span><span id="response-embedders-indexing-embedder-headers-additional-properties"></span><span id="response-embedders-indexing-embedder-one-of-1-headers-additional-properties"></span><span id="response-embedders-search-embedder-headers-additional-properties"></span><span id="response-embedders-search-embedder-one-of-1-headers-additional-properties"></span>          additionalProperties:
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
<span id="response-embedders-additional-properties-distribution-current-mean"></span><span id="response-embedders-additional-properties-distribution-one-of-1-current-mean"></span><span id="response-embedders-distribution-current-mean"></span><span id="response-embedders-distribution-one-of-1-current-mean"></span>        current_mean:
          type: number
          format: float
          description: &gt;-
            Value where the results are "packed".


            Similarity scores are translated so that they are packed around 0.5
            instead
<span id="response-embedders-additional-properties-distribution-current-sigma"></span><span id="response-embedders-additional-properties-distribution-one-of-1-current-sigma"></span><span id="response-embedders-distribution-current-sigma"></span><span id="response-embedders-distribution-one-of-1-current-sigma"></span>        current_sigma:
          type: number
          format: float
          description: &gt;-
            standard deviation of a similarity score.


            Set below 0.4 to make the results less packed around the mean, and
            above 0.4 to make them more packed.
    Locale:
      type: string
      enum:
        - af
        - ak
        - am
        - ar
        - az
        - be
        - bn
        - bg
        - ca
        - cs
        - da
        - de
        - cy
        - el
        - en
        - eo
        - et
        - fi
        - fr
        - gu
        - he
        - hi
        - hr
        - hu
        - hy
        - id
        - it
        - jv
        - ja
        - kn
        - ka
        - km
        - ko
        - la
        - lv
        - lt
        - ml
        - mr
        - mk
        - my
        - ne
        - nl
        - nb
        - or
        - pa
        - fa
        - pl
        - pt
        - ro
        - ru
        - si
        - sk
        - sl
        - sn
        - es
        - sr
        - sv
        - ta
        - te
        - tl
        - th
        - tk
        - tr
        - uk
        - ur
        - uz
        - vi
        - yi
        - zh
        - zu
        - afr
        - aka
        - amh
        - ara
        - aze
        - bel
        - ben
        - bul
        - cat
        - ces
        - dan
        - deu
        - cym
        - ell
        - eng
        - epo
        - est
        - fin
        - fra
        - guj
        - heb
        - hin
        - hrv
        - hun
        - hye
        - ind
        - ita
        - jav
        - jpn
        - kan
        - kat
        - khm
        - kor
        - lat
        - lav
        - lit
        - mal
        - mar
        - mkd
        - mya
        - nep
        - nld
        - nob
        - ori
        - pan
        - pes
        - pol
        - por
        - ron
        - rus
        - sin
        - slk
        - slv
        - sna
        - spa
        - srp
        - swe
        - tam
        - tel
        - tgl
        - tha
        - tuk
        - tur
        - ukr
        - urd
        - uzb
        - vie
        - yid
        - zho
        - zul
        - cmn
    ChatSearchParams:
      type: object
      description: Search parameters applied when the LLM queries this index.
      properties:
<span id="response-chat-one-of-1-search-parameters-hybrid"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-hybrid"></span><span id="response-chat-search-parameters-hybrid"></span><span id="response-chat-search-parameters-one-of-1-hybrid"></span>        hybrid:
          oneOf:
<span id="response-chat-one-of-1-search-parameters-hybrid-one-of-0"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-hybrid-one-of-0"></span><span id="response-chat-search-parameters-hybrid-one-of-0"></span><span id="response-chat-search-parameters-one-of-1-hybrid-one-of-0"></span>            - type: 'null'
<span id="response-chat-one-of-1-search-parameters-hybrid-one-of-1"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-hybrid-one-of-1"></span><span id="response-chat-search-parameters-hybrid-one-of-1"></span><span id="response-chat-search-parameters-one-of-1-hybrid-one-of-1"></span>            - $ref: '#/components/schemas/HybridQuery'
              description: &gt;-
                Hybrid search: mix of keyword and semantic search. Requires an
                embedder (set via `embedder` and optionally `semanticRatio`).
<span id="response-chat-one-of-1-search-parameters-limit"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-limit"></span><span id="response-chat-search-parameters-limit"></span><span id="response-chat-search-parameters-one-of-1-limit"></span>        limit:
          type:
            - integer
            - 'null'
          description: &gt;-
            Maximum number of documents returned per search performed by the
            LLM.
          example: 20
          minimum: 0
<span id="response-chat-one-of-1-search-parameters-one-of-1-sort"></span><span id="response-chat-one-of-1-search-parameters-sort"></span><span id="response-chat-search-parameters-one-of-1-sort"></span><span id="response-chat-search-parameters-sort"></span>        sort:
          type:
            - array
            - 'null'
<span id="response-chat-one-of-1-search-parameters-one-of-1-sort-items"></span><span id="response-chat-one-of-1-search-parameters-sort-items"></span><span id="response-chat-search-parameters-one-of-1-sort-items"></span><span id="response-chat-search-parameters-sort-items"></span>          items:
            type: string
          description: &gt;-
            Sort order: array of strings like `attribute:asc` or
            `attribute:desc`.
          example:
            - price:asc
            - rating:desc
<span id="response-chat-one-of-1-search-parameters-distinct"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-distinct"></span><span id="response-chat-search-parameters-distinct"></span><span id="response-chat-search-parameters-one-of-1-distinct"></span>        distinct:
          type:
            - string
            - 'null'
          description: Attribute used to return at most one document per distinct value.
          example: sku
<span id="response-chat-one-of-1-search-parameters-matching-strategy"></span><span id="response-chat-one-of-1-search-parameters-matchingstrategy"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-matching-strategy"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-matchingstrategy"></span><span id="response-chat-search-parameters-matching-strategy"></span><span id="response-chat-search-parameters-matchingstrategy"></span><span id="response-chat-search-parameters-one-of-1-matching-strategy"></span><span id="response-chat-search-parameters-one-of-1-matchingstrategy"></span>        matchingStrategy:
          oneOf:
<span id="response-chat-one-of-1-search-parameters-matching-strategy-one-of-0"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-matching-strategy-one-of-0"></span><span id="response-chat-search-parameters-matching-strategy-one-of-0"></span><span id="response-chat-search-parameters-one-of-1-matching-strategy-one-of-0"></span>            - type: 'null'
<span id="response-chat-one-of-1-search-parameters-matching-strategy-one-of-1"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-matching-strategy-one-of-1"></span><span id="response-chat-search-parameters-matching-strategy-one-of-1"></span><span id="response-chat-search-parameters-one-of-1-matching-strategy-one-of-1"></span>            - $ref: '#/components/schemas/MatchingStrategy'
              description: 'How query terms are matched: `last`, `all`, or `frequency`.'
<span id="response-chat-one-of-1-search-parameters-attributes-to-search-on"></span><span id="response-chat-one-of-1-search-parameters-attributestosearchon"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-attributes-to-search-on"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-attributestosearchon"></span><span id="response-chat-search-parameters-attributes-to-search-on"></span><span id="response-chat-search-parameters-attributestosearchon"></span><span id="response-chat-search-parameters-one-of-1-attributes-to-search-on"></span><span id="response-chat-search-parameters-one-of-1-attributestosearchon"></span>        attributesToSearchOn:
          type:
            - array
            - 'null'
<span id="response-chat-one-of-1-search-parameters-attributes-to-search-on-items"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-attributes-to-search-on-items"></span><span id="response-chat-search-parameters-attributes-to-search-on-items"></span><span id="response-chat-search-parameters-one-of-1-attributes-to-search-on-items"></span>          items:
            type: string
          description: &gt;-
            Attributes on which to run the search. If unset, all searchable
            attributes are used.
          example:
            - title
            - description
<span id="response-chat-one-of-1-search-parameters-one-of-1-ranking-score-threshold"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-rankingscorethreshold"></span><span id="response-chat-one-of-1-search-parameters-ranking-score-threshold"></span><span id="response-chat-one-of-1-search-parameters-rankingscorethreshold"></span><span id="response-chat-search-parameters-one-of-1-ranking-score-threshold"></span><span id="response-chat-search-parameters-one-of-1-rankingscorethreshold"></span><span id="response-chat-search-parameters-ranking-score-threshold"></span><span id="response-chat-search-parameters-rankingscorethreshold"></span>        rankingScoreThreshold:
          oneOf:
<span id="response-chat-one-of-1-search-parameters-one-of-1-ranking-score-threshold-one-of-0"></span><span id="response-chat-one-of-1-search-parameters-ranking-score-threshold-one-of-0"></span><span id="response-chat-search-parameters-one-of-1-ranking-score-threshold-one-of-0"></span><span id="response-chat-search-parameters-ranking-score-threshold-one-of-0"></span>            - type: 'null'
<span id="response-chat-one-of-1-search-parameters-one-of-1-ranking-score-threshold-one-of-1"></span><span id="response-chat-one-of-1-search-parameters-ranking-score-threshold-one-of-1"></span><span id="response-chat-search-parameters-one-of-1-ranking-score-threshold-one-of-1"></span><span id="response-chat-search-parameters-ranking-score-threshold-one-of-1"></span>            - $ref: '#/components/schemas/RankingScoreThreshold'
              description: &gt;-
                Minimum ranking score (0.0–1.0) for a document to be included.
                Lower scores are excluded.
    FilterableAttributesFeatures:
      type: object
      description: &gt;-
        Controls which filtering and faceting operations are enabled for
        matching

        attributes. This allows restricting certain operations on specific
        fields

        for security or performance reasons.
      properties:
<span id="response-filterable-attributes-features-facet-search"></span><span id="response-filterable-attributes-features-facetsearch"></span><span id="response-filterable-attributes-items-features-facet-search"></span><span id="response-filterable-attributes-items-features-facetsearch"></span><span id="response-filterable-attributes-items-one-of-0-features-facet-search"></span><span id="response-filterable-attributes-items-one-of-0-features-facetsearch"></span><span id="response-filterable-attributes-one-of-0-features-facet-search"></span><span id="response-filterable-attributes-one-of-0-features-facetsearch"></span>        facetSearch:
          type: boolean
          description: |-
            When `true`, allows searching within facet values for matching
            attributes. This enables the facet search feature which lets users
            search for specific facet values. Defaults to `false`.
<span id="response-filterable-attributes-features-filter"></span><span id="response-filterable-attributes-items-features-filter"></span><span id="response-filterable-attributes-items-one-of-0-features-filter"></span><span id="response-filterable-attributes-one-of-0-features-filter"></span>        filter:
          $ref: '#/components/schemas/FilterFeatures'
          description: |-
            Controls which filter operators are allowed for matching attributes.
            See `FilterFeatures` for available options.
      additionalProperties: false
    HybridQuery:
      type: object
      description: 'Hybrid search: balance between keyword and semantic search.'
      required:
        - embedder
      properties:
<span id="response-chat-one-of-1-search-parameters-hybrid-one-of-1-semantic-ratio"></span><span id="response-chat-one-of-1-search-parameters-hybrid-one-of-1-semanticratio"></span><span id="response-chat-one-of-1-search-parameters-hybrid-semantic-ratio"></span><span id="response-chat-one-of-1-search-parameters-hybrid-semanticratio"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-hybrid-one-of-1-semantic-ratio"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-hybrid-one-of-1-semanticratio"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-hybrid-semantic-ratio"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-hybrid-semanticratio"></span><span id="response-chat-search-parameters-hybrid-one-of-1-semantic-ratio"></span><span id="response-chat-search-parameters-hybrid-one-of-1-semanticratio"></span><span id="response-chat-search-parameters-hybrid-semantic-ratio"></span><span id="response-chat-search-parameters-hybrid-semanticratio"></span><span id="response-chat-search-parameters-one-of-1-hybrid-one-of-1-semantic-ratio"></span><span id="response-chat-search-parameters-one-of-1-hybrid-one-of-1-semanticratio"></span><span id="response-chat-search-parameters-one-of-1-hybrid-semantic-ratio"></span><span id="response-chat-search-parameters-one-of-1-hybrid-semanticratio"></span>        semanticRatio:
          type: number
          format: float
          description: Balance between keyword (0.0) and semantic (1.0) search.
          example: 0.5
<span id="response-chat-one-of-1-search-parameters-hybrid-embedder"></span><span id="response-chat-one-of-1-search-parameters-hybrid-one-of-1-embedder"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-hybrid-embedder"></span><span id="response-chat-one-of-1-search-parameters-one-of-1-hybrid-one-of-1-embedder"></span><span id="response-chat-search-parameters-hybrid-embedder"></span><span id="response-chat-search-parameters-hybrid-one-of-1-embedder"></span><span id="response-chat-search-parameters-one-of-1-hybrid-embedder"></span><span id="response-chat-search-parameters-one-of-1-hybrid-one-of-1-embedder"></span>        embedder:
          type: string
          description: &gt;-
            Name of the embedder from the index embedders setting (`embedder` in
            JSON). Used to vectorize the query.
          example: default
    MatchingStrategy:
      type: string
      description: Strategy used to match query terms within documents
      enum:
        - last
        - all
        - frequency
    RankingScoreThreshold:
      type: number
      format: double
    FilterFeatures:
      type: object
      description: |-
        Controls which filter operators are allowed for an attribute. This
        provides fine-grained control over filtering capabilities.
      properties:
<span id="response-filterable-attributes-features-filter-equality"></span><span id="response-filterable-attributes-items-features-filter-equality"></span><span id="response-filterable-attributes-items-one-of-0-features-filter-equality"></span><span id="response-filterable-attributes-one-of-0-features-filter-equality"></span>        equality:
          type: boolean
          description: |-
            When `true`, enables equality operators: `=`, `!=`, and `IN`. These
            allow filtering for exact matches or membership in a set of values.
            Also enables `IS EMPTY`, `IS NULL`, and `EXISTS` operators. Defaults
            to `true`.
<span id="response-filterable-attributes-features-filter-comparison"></span><span id="response-filterable-attributes-items-features-filter-comparison"></span><span id="response-filterable-attributes-items-one-of-0-features-filter-comparison"></span><span id="response-filterable-attributes-one-of-0-features-filter-comparison"></span>        comparison:
          type: boolean
          description: |-
            When `true`, enables comparison operators: `&lt;`, `&gt;`, `&lt;=`, `&gt;=`, and
            `TO` (range). These allow filtering based on numeric or string
            comparisons. Defaults to `false`.
      additionalProperties: false
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