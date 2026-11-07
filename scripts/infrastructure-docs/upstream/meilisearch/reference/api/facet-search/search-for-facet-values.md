# Search for facet values

> Search for facet values matching a query within a given facet attribute. Use this to build
autocomplete or dropdown UIs for facet filters.

**Prerequisite:** The `facetName` attribute must be in the index's `filterableAttributes` list.
Facet search will not work without this configuration.

**Note:** Facet search only considers the first word of `facetQuery`. Searching for `Jane`
returns `Jane Austen`, but searching for `Austen` does not.

**Note:** Numeric facet values are not searchable. Convert numbers to strings if you need
to search them as facets.



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
  /indexes/{index_uid}/facet-search:
    post:
      tags:
        - Facet Search
      summary: Search for facet values
      description: &gt;-
        Search for facet values matching a query within a given facet attribute.
        Use this to build

        autocomplete or dropdown UIs for facet filters.


        **Prerequisite:** The `facetName` attribute must be in the index's
        `filterableAttributes` list.

        Facet search will not work without this configuration.


        **Note:** Facet search only considers the first word of `facetQuery`.
        Searching for `Jane`

        returns `Jane Austen`, but searching for `Austen` does not.


        **Note:** Numeric facet values are not searchable. Convert numbers to
        strings if you need

        to search them as facets.
      operationId: search
      parameters:
        - name: index_uid
<span id="parameter-index-uid"></span>          in: path
          description: Unique identifier of the index.
          required: true
          schema:
            type: string
          example: movies
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/FacetSearchQuery'
        required: true
      responses:
        '200':
          description: Facet values matching the query are returned.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/FacetSearchResult'
              example:
                facetHits:
                  - value: action
                    count: 273
                  - value: animated
                    count: 15
                facetQuery: ac
                processingTimeMs: 0
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
            - search
            - '*'
      x-codeSamples:
        - lang: cURL
          source: |-
            curl \
              -X POST 'MEILISEARCH_URL/indexes/books/facet-search' \
              -H 'Authorization: Bearer API_KEY' \
              -H 'Content-Type: application/json' \
              --data-binary '{
                "facetQuery": "fiction",
                "facetName": "genres",
                "filter": "rating &gt; 3"
              }'
        - lang: JS
          source: |-
            client.index('books').searchForFacetValues({
              facetQuery: 'fiction',
              facetName: 'genres'
              filter: 'rating &gt; 3'
            })
        - lang: PHP
          source: |-
            $client-&gt;index('books')-&gt;facetSearch(
              (new FacetSearchQuery())
                  -&gt;setFacetQuery('fiction')
                  -&gt;setFacetName('genres')
                  -&gt;setFilter(['rating &gt; 3'])
            );
        - lang: Python
          source: |-
            client.index('books').facet_search('genres', 'fiction', {
              'filter': 'rating &gt; 3'
            })
        - lang: Java
          source: &gt;-
            FacetSearchRequest fsr =
            FacetSearchRequest.builder().facetName("genres").facetQuery("fiction").filter(new
            String[]{"rating &gt; 3"}).build();

            client.index("books").facetSearch(fsr);
        - lang: Ruby
          source: &gt;-
            client.index('books').facet_search('genres', 'fiction', filter:
            'rating &gt; 3')
        - lang: Go
          source: |-
            client.Index("books").FacetSearch(&amp;meilisearch.FacetSearchRequest{
              FacetQuery: "fiction",
              FacetName: "genres",
              Filter: "rating &gt; 3",
            })
        - lang: C#
          source: |-
            var query = new SearchFacetsQuery()
            {
              FacetQuery = "fiction",
              Filter = "rating &gt; 3"
            };
            await client.Index("books").FacetSearchAsync("genres", query);
        - lang: Rust
          source: |-
            let res = client.index("books")
              .facet_search("genres")
              .with_facet_query("fiction")
              .with_filter("rating &gt; 3")
              .execute()
              .await
              .unwrap();
        - lang: Dart
          source: |-
            await client.index('books').facetSearch(
                  FacetSearchQuery(
                    facetQuery: 'fiction',
                    facetName: 'genres',
                    filter: 'rating &gt; 3',
                  ),
                );
components:
  schemas:
    FacetSearchQuery:
      type: object
      description: Request body for searching facet values
      required:
        - facetName
      properties:
<span id="body-facet-query"></span><span id="body-facetquery"></span>        facetQuery:
          type:
            - string
            - 'null'
          description: &gt;-
            Query string to search for matching facet values. If not specified,
            Meilisearch returns all facet values for the `facetName`, limited to
            100 results. Note: only the first word is used for matching.
<span id="body-facet-name"></span><span id="body-facetname"></span>        facetName:
          type: string
          description: &gt;-
            Name of the facet attribute to search. Must be present in the
            index's `filterableAttributes` list.
<span id="body-q"></span>        q:
          type:
            - string
            - 'null'
          description: &gt;-
            Query string to filter the underlying documents before computing
            facet values. This affects which facet values appear and their
            counts, but does not filter the facet values themselves.
<span id="body-vector"></span>        vector:
          type:
            - array
            - 'null'
<span id="body-vector-items"></span>          items:
            type: number
            format: float
          description: Custom query vector for semantic search
<span id="body-media"></span>        media:
          description: Multimodal content for AI-powered search
<span id="body-hybrid"></span>        hybrid:
          oneOf:
<span id="body-hybrid-one-of-0"></span>            - type: 'null'
<span id="body-hybrid-one-of-1"></span>            - $ref: '#/components/schemas/HybridQuery'
              description: &gt;-
                Hybrid search configuration that combines keyword search with
                semantic

                (vector) search. Set `semanticRatio` to balance between keyword

                matching (0.0) and semantic similarity (1.0). Requires an
                embedder to

                be configured in the index settings.
<span id="body-filter"></span>        filter:
          description: &gt;-
            Filter expression to restrict which documents are considered when
            computing facet values. Attributes must be in `filterableAttributes`
            before they can be used in filters.
<span id="body-matching-strategy"></span><span id="body-matchingstrategy"></span>        matchingStrategy:
          oneOf:
<span id="body-matching-strategy-one-of-0"></span>            - $ref: '#/components/schemas/MatchingStrategy'
              description: Strategy used to match query terms
<span id="body-attributes-to-search-on"></span><span id="body-attributestosearchon"></span>        attributesToSearchOn:
          type:
            - array
            - 'null'
<span id="body-attributes-to-search-on-items"></span>          items:
            type: string
          description: Restrict search to specified attributes
<span id="body-ranking-score-threshold"></span><span id="body-rankingscorethreshold"></span>        rankingScoreThreshold:
          type:
            - number
            - 'null'
          format: double
          description: |-
            Minimum ranking score threshold (0.0 to 1.0) that documents must
            achieve to be considered when computing facet counts. Documents with
            scores below this threshold are excluded from facet value counts.
<span id="body-locales"></span>        locales:
          type:
            - array
            - 'null'
<span id="body-locales-items"></span>          items:
            $ref: '#/components/schemas/Locale'
          description: Languages to use for query processing
<span id="body-exhaustive-facet-count"></span><span id="body-exhaustivefacetcount"></span>        exhaustiveFacetCount:
          type:
            - boolean
            - 'null'
          description: Return exhaustive facet count instead of an estimate
<span id="body-use-network"></span><span id="body-usenetwork"></span>        useNetwork:
          type:
            - boolean
            - 'null'
          description: &gt;-
            When `true`, runs the query on the whole network (all shards covered
            exactly once).


            When `false`, the query runs locally.


            When omitted or `null`, the default value depends on whether the
            sharding is enabled for the instance:


            - If the instance has sharding enabled (has a leader), defaults to
            `true`.

            - Otherwise defaults to `false`.


            It also requires the `network` [experimental
            feature](http://localhost:3000/reference/api/experimental-features/configure-experimental-features).


            Values: `true` = use the whole network; `false` = local, default =
            see above.


            When using the network, the index must exist with compatible
            settings on all remotes.
    FacetSearchResult:
      type: object
      description: Result of a facet search operation.
      required:
        - facetHits
        - processingTimeMs
      properties:
<span id="response-facet-hits"></span><span id="response-facethits"></span>        facetHits:
          type: array
<span id="response-facet-hits-items"></span>          items:
            $ref: '#/components/schemas/FacetValueHitSchema'
          description: &gt;-
            Array of matching facet values with their document counts, sorted
            lexicographically

            in ascending order (or by count if `sortFacetValuesBy` is set to
            `"count"`).
<span id="response-facet-query"></span><span id="response-facetquery"></span>        facetQuery:
          type:
            - string
            - 'null'
          description: &gt;-
            The original `facetQuery` from the request. `null` if no query was
            provided.
<span id="response-processing-time-ms"></span><span id="response-processingtimems"></span>        processingTimeMs:
          type: integer
          description: Time in milliseconds Meilisearch took to process the request.
          minimum: 0
<span id="response-remote-errors"></span><span id="response-remoteerrors"></span>        remoteErrors:
          type:
            - object
            - 'null'
          description: &gt;-
            Errors from remote shards. Only present in federated search when
            some remotes failed.
<span id="response-remote-errors-additional-properties"></span>          additionalProperties:
            $ref: '#/components/schemas/ResponseError'
          propertyNames:
            type: string
    ResponseError:
      type: object
      required:
        - message
        - code
        - type
        - link
      properties:
<span id="response-remote-errors-additional-properties-message"></span><span id="response-remote-errors-message"></span>        message:
          type: string
          description: The error message.
<span id="response-remote-errors-additional-properties-code"></span><span id="response-remote-errors-code"></span>        code:
          $ref: '#/components/schemas/Code'
          description: The error code.
<span id="response-remote-errors-additional-properties-type"></span><span id="response-remote-errors-type"></span>        type:
          $ref: '#/components/schemas/ErrorType'
          description: The error type.
<span id="response-remote-errors-additional-properties-link"></span><span id="response-remote-errors-link"></span>        link:
          type: string
          description: A link to the documentation about this specific error.
    HybridQuery:
      type: object
      description: 'Hybrid search: balance between keyword and semantic search.'
      required:
        - embedder
      properties:
<span id="body-hybrid-one-of-1-semantic-ratio"></span><span id="body-hybrid-one-of-1-semanticratio"></span><span id="body-hybrid-semantic-ratio"></span><span id="body-hybrid-semanticratio"></span>        semanticRatio:
          type: number
          format: float
          description: Balance between keyword (0.0) and semantic (1.0) search.
          example: 0.5
<span id="body-hybrid-embedder"></span><span id="body-hybrid-one-of-1-embedder"></span>        embedder:
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
    FacetValueHitSchema:
      type: object
      description: &gt;-
        Schema representation of a facet value hit (for OpenAPI documentation
        only).
      required:
        - value
        - count
      properties:
<span id="response-facet-hits-items-value"></span><span id="response-facet-hits-value"></span>        value:
          type: string
          description: The facet value that matched the query.
<span id="response-facet-hits-count"></span><span id="response-facet-hits-items-count"></span>        count:
          type: integer
          format: u-int64
          description: Number of documents with this facet value among the search results.
          minimum: 0
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