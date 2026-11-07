# Get similar documents with POST

> Retrieve documents similar to a reference document identified by its id.

> Useful for “more like this” or recommendations.



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
  /indexes/{index_uid}/similar:
    post:
      tags:
        - Similar documents
      summary: Get similar documents with POST
      description: |-
        Retrieve documents similar to a reference document identified by its id.

        &gt; Useful for “more like this” or recommendations.
      operationId: similar_post
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
              $ref: '#/components/schemas/SimilarQuery'
        required: true
      responses:
        '200':
          description: The documents are returned.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/SimilarResult'
              example:
                hits:
                  - id: 2770
                    title: American Pie 2
                    poster: &gt;-
                      https://image.tmdb.org/t/p/w1280/q4LNgUnRfltxzp3gf1MAGiK5LhV.jpg
                    overview: &gt;-
                      The whole gang are back and as close as ever. They decide
                      to get even closer by spending the summer together at a
                      beach house. They decide to hold the biggest…
                    release_date: 997405200
                  - id: 190859
                    title: American Sniper
                    poster: &gt;-
                      https://image.tmdb.org/t/p/w1280/svPHnYE7N5NAGO49dBmRhq0vDQ3.jpg
                    overview: &gt;-
                      U.S. Navy SEAL Chris Kyle takes his sole mission—protect
                      his comrades—to heart and becomes one of the most lethal
                      snipers in American history. His pinpoint accuracy not
                      only saves countless lives but also makes him a prime…
                    release_date: 1418256000
                id: '143'
                offset: 0
                limit: 2
                estimatedTotalHits: 976
                processingTimeMs: 35
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
              -X POST 'MEILISEARCH_URL/indexes/INDEX_NAME/similar' \
              -H 'Content-Type: application/json' \
              -H 'Authorization: Bearer DEFAULT_SEARCH_API_KEY' \
              --data-binary '{
                "id": TARGET_DOCUMENT_ID,
                "embedder": "EMBEDDER_NAME"
              }'
        - lang: JS
          source: &gt;-
            client.index('INDEX_NAME').searchSimilarDocuments({ id:
            'TARGET_DOCUMENT_ID', embedder: 'default' })
        - lang: PHP
          source: &gt;-
            $similarQuery = new SimilarDocumentsQuery('TARGET_DOCUMENT_ID',
            'default');

            $client-&gt;index('INDEX_NAME')-&gt;searchSimilarDocuments($similarQuery);
        - lang: Python
          source: &gt;-
            client.index("INDEX_NAME").get_similar_documents({"id":
            "TARGET_DOCUMENT_ID", "embedder": "default"})
        - lang: Java
          source: &gt;-
            SimilarDocumentRequest query = new SimilarDocumentRequest()
            .setId("143") .setEmbedder("manual");
            client.index("movies").searchSimilarDocuments(query)
        - lang: Ruby
          source: &gt;-
            client.index('INDEX_NAME').search_similar_documents('TARGET_DOCUMENT_ID',
            embedder: 'default')
        - lang: Go
          source: &gt;-
            resp := new(meilisearch.SimilarDocumentResult)

            client.Index("INDEX_NAME").SearchSimilarDocuments(&amp;meilisearch.SimilarDocumentQuery{
              Id: "TARGET_DOCUMENT_ID",
              Embedder: "default",
            }, resp)
        - lang: Rust
          source: |-
            let results = index
              .similar_search("TARGET_DOCUMENT_ID", "EMBEDDER_NAME")
              .execute()
              .await
              .unwrap();
components:
  schemas:
    SimilarQuery:
      type: object
      description: Request body for similar document search
      required:
        - id
        - embedder
      properties:
<span id="body-id"></span>        id:
          type: string
          description: Document ID to find similar documents for
<span id="body-offset"></span>        offset:
          type: integer
          description: Number of documents to skip
          default: 0
          minimum: 0
<span id="body-limit"></span>        limit:
          type: integer
          description: Maximum number of documents returned
          default: 20
          minimum: 0
<span id="body-filter"></span>        filter:
          description: Filter queries by an attribute's value
<span id="body-embedder"></span>        embedder:
          type: string
          description: Name of the embedder to use for semantic similarity
<span id="body-attributes-to-retrieve"></span><span id="body-attributestoretrieve"></span>        attributesToRetrieve:
          type:
            - array
            - 'null'
<span id="body-attributes-to-retrieve-items"></span>          items:
            type: string
          description: Attributes to display in the returned documents
          uniqueItems: true
<span id="body-retrieve-vectors"></span><span id="body-retrievevectors"></span>        retrieveVectors:
          type: boolean
          description: Return document vector data
<span id="body-show-ranking-score"></span><span id="body-showrankingscore"></span>        showRankingScore:
          type: boolean
          description: Display the global ranking score of a document
<span id="body-show-ranking-score-details"></span><span id="body-showrankingscoredetails"></span>        showRankingScoreDetails:
          type: boolean
          description: Adds a detailed global ranking score field
<span id="body-show-performance-details"></span><span id="body-showperformancedetails"></span>        showPerformanceDetails:
          type: boolean
          description: Adds a detailed performance details field
<span id="body-ranking-score-threshold"></span><span id="body-rankingscorethreshold"></span>        rankingScoreThreshold:
          type: number
          format: double
          description: Excludes results with low ranking scores
    SimilarResult:
      allOf:
<span id="response-all-of-0"></span>        - $ref: '#/components/schemas/HitsInfo'
          description: Pagination information
<span id="response-all-of-1"></span>        - type: object
          required:
            - hits
            - id
            - processingTimeMs
          properties:
<span id="response-all-of-1-hits"></span><span id="response-hits"></span>            hits:
              type: array
<span id="response-all-of-1-hits-items"></span><span id="response-hits-items"></span>              items:
                $ref: '#/components/schemas/SearchHit'
              description: Results of the query
<span id="response-all-of-1-id"></span><span id="response-id"></span>            id:
              type: string
              description: Document ID that was used as reference
<span id="response-all-of-1-processing-time-ms"></span><span id="response-all-of-1-processingtimems"></span><span id="response-processing-time-ms"></span><span id="response-processingtimems"></span>            processingTimeMs:
              type: integer
              description: Processing time of the query in milliseconds
              minimum: 0
<span id="response-all-of-1-performance-details"></span><span id="response-all-of-1-performancedetails"></span><span id="response-performance-details"></span><span id="response-performancedetails"></span>            performanceDetails:
              description: Performance details of the query
      description: Response containing similar documents
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
    HitsInfo:
      oneOf:
<span id="response-all-of-0-one-of-0"></span><span id="response-one-of-0"></span>        - type: object
          description: |-
            Page-based pagination with exact totals.

            Used when `page` or `hitsPerPage` was set in the request.
          required:
            - hitsPerPage
            - page
            - totalPages
            - totalHits
          properties:
<span id="response-all-of-0-hits-per-page"></span><span id="response-all-of-0-hitsperpage"></span><span id="response-all-of-0-one-of-0-hits-per-page"></span><span id="response-all-of-0-one-of-0-hitsperpage"></span><span id="response-hits-per-page"></span><span id="response-hitsperpage"></span><span id="response-one-of-0-hits-per-page"></span><span id="response-one-of-0-hitsperpage"></span>            hitsPerPage:
              type: integer
              description: Number of results per page.
              minimum: 0
<span id="response-all-of-0-one-of-0-page"></span><span id="response-all-of-0-page"></span><span id="response-one-of-0-page"></span><span id="response-page"></span>            page:
              type: integer
              description: Current page index (1-based).
              minimum: 0
<span id="response-all-of-0-one-of-0-total-pages"></span><span id="response-all-of-0-one-of-0-totalpages"></span><span id="response-all-of-0-total-pages"></span><span id="response-all-of-0-totalpages"></span><span id="response-one-of-0-total-pages"></span><span id="response-one-of-0-totalpages"></span><span id="response-total-pages"></span><span id="response-totalpages"></span>            totalPages:
              type: integer
              description: Exhaustive total number of result pages.
              minimum: 0
<span id="response-all-of-0-one-of-0-total-hits"></span><span id="response-all-of-0-one-of-0-totalhits"></span><span id="response-all-of-0-total-hits"></span><span id="response-all-of-0-totalhits"></span><span id="response-one-of-0-total-hits"></span><span id="response-one-of-0-totalhits"></span><span id="response-total-hits"></span><span id="response-totalhits"></span>            totalHits:
              type: integer
              description: Exhaustive total number of matching documents.
              minimum: 0
<span id="response-all-of-0-one-of-1"></span><span id="response-one-of-1"></span>        - type: object
          description: |-
            Offset-based pagination with estimated total.

            Used when only `offset` and `limit` were set.
          required:
            - limit
            - offset
            - estimatedTotalHits
          properties:
<span id="response-all-of-0-limit"></span><span id="response-all-of-0-one-of-1-limit"></span><span id="response-limit"></span><span id="response-one-of-1-limit"></span>            limit:
              type: integer
              description: Maximum number of documents returned.
              minimum: 0
<span id="response-all-of-0-offset"></span><span id="response-all-of-0-one-of-1-offset"></span><span id="response-offset"></span><span id="response-one-of-1-offset"></span>            offset:
              type: integer
              description: Number of documents skipped.
              minimum: 0
<span id="response-all-of-0-estimated-total-hits"></span><span id="response-all-of-0-estimatedtotalhits"></span><span id="response-all-of-0-one-of-1-estimated-total-hits"></span><span id="response-all-of-0-one-of-1-estimatedtotalhits"></span><span id="response-estimated-total-hits"></span><span id="response-estimatedtotalhits"></span><span id="response-one-of-1-estimated-total-hits"></span><span id="response-one-of-1-estimatedtotalhits"></span>            estimatedTotalHits:
              type: integer
              description: |-
                Estimated total number of matches (not exhaustive).

                Prioritizes relevancy and performance.
              minimum: 0
      description: Pagination information for search results.
    SearchHit:
      type: object
      description: A single search result hit.
      properties:
<span id="response-all-of-1-hits--formatted"></span><span id="response-all-of-1-hits-formatted"></span><span id="response-all-of-1-hits-items--formatted"></span><span id="response-all-of-1-hits-items-formatted"></span><span id="response-hits--formatted"></span><span id="response-hits-formatted"></span><span id="response-hits-items--formatted"></span><span id="response-hits-items-formatted"></span>        _formatted:
          type: object
          description: |-
            Document with highlighted and cropped attributes.

            Present when `attributesToHighlight` or `attributesToCrop` was set.
          additionalProperties: true
<span id="response-all-of-1-hits--matchesposition"></span><span id="response-all-of-1-hits-items--matchesposition"></span><span id="response-all-of-1-hits-items-matches-position"></span><span id="response-all-of-1-hits-matches-position"></span><span id="response-hits--matchesposition"></span><span id="response-hits-items--matchesposition"></span><span id="response-hits-items-matches-position"></span><span id="response-hits-matches-position"></span>        _matchesPosition:
          type:
            - object
            - 'null'
          description: |-
            Byte offsets (`start`, `length`) of each matched term per attribute.

            Present when `showMatchesPosition` was true.
<span id="response-all-of-1-hits-items-matches-position-additional-properties"></span><span id="response-all-of-1-hits-matches-position-additional-properties"></span><span id="response-hits-items-matches-position-additional-properties"></span><span id="response-hits-matches-position-additional-properties"></span>          additionalProperties:
            type: array
<span id="response-all-of-1-hits-items-matches-position-additional-properties-items"></span><span id="response-all-of-1-hits-items-matches-position-items"></span><span id="response-all-of-1-hits-matches-position-additional-properties-items"></span><span id="response-all-of-1-hits-matches-position-items"></span><span id="response-hits-items-matches-position-additional-properties-items"></span><span id="response-hits-items-matches-position-items"></span><span id="response-hits-matches-position-additional-properties-items"></span><span id="response-hits-matches-position-items"></span>            items:
              $ref: '#/components/schemas/MatchBounds'
          propertyNames:
            type: string
<span id="response-all-of-1-hits--rankingscore"></span><span id="response-all-of-1-hits-items--rankingscore"></span><span id="response-all-of-1-hits-items-ranking-score"></span><span id="response-all-of-1-hits-ranking-score"></span><span id="response-hits--rankingscore"></span><span id="response-hits-items--rankingscore"></span><span id="response-hits-items-ranking-score"></span><span id="response-hits-ranking-score"></span>        _rankingScore:
          type:
            - number
            - 'null'
          format: double
          description: &gt;-
            Global [ranking
            score](https://www.meilisearch.com/docs/learn/relevancy/ranking_score)
            from 0.0 to 1.0.


            Present when `showRankingScore` was true.
<span id="response-all-of-1-hits--rankingscoredetails"></span><span id="response-all-of-1-hits-items--rankingscoredetails"></span><span id="response-all-of-1-hits-items-ranking-score-details"></span><span id="response-all-of-1-hits-ranking-score-details"></span><span id="response-hits--rankingscoredetails"></span><span id="response-hits-items--rankingscoredetails"></span><span id="response-hits-items-ranking-score-details"></span><span id="response-hits-ranking-score-details"></span>        _rankingScoreDetails:
          type:
            - object
            - 'null'
          description: |-
            Per-rule score breakdown (words, typo, proximity, etc.).

            Present when `showRankingScoreDetails` was true.
<span id="response-all-of-1-hits-items-ranking-score-details-additional-properties"></span><span id="response-all-of-1-hits-ranking-score-details-additional-properties"></span><span id="response-hits-items-ranking-score-details-additional-properties"></span><span id="response-hits-ranking-score-details-additional-properties"></span>          additionalProperties: {}
          propertyNames:
            type: string
<span id="response-all-of-1-hits-additional-properties"></span><span id="response-all-of-1-hits-items-additional-properties"></span><span id="response-hits-additional-properties"></span><span id="response-hits-items-additional-properties"></span>      additionalProperties:
        description: |-
          Document fields as stored in the index.

          According to `attributesToRetrieve`.
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
    MatchBounds:
      type: object
      description: |-
        Represents the position of a matching term in a document field. Used to
        indicate where query terms were found within attribute values, enabling
        features like highlighting and match position display.
      required:
        - start
        - length
      properties:
<span id="response-all-of-1-hits-items-matches-position-additional-properties-items-start"></span><span id="response-all-of-1-hits-items-matches-position-additional-properties-start"></span><span id="response-all-of-1-hits-items-matches-position-items-start"></span><span id="response-all-of-1-hits-items-matches-position-start"></span><span id="response-all-of-1-hits-matches-position-additional-properties-items-start"></span><span id="response-all-of-1-hits-matches-position-additional-properties-start"></span><span id="response-all-of-1-hits-matches-position-items-start"></span><span id="response-all-of-1-hits-matches-position-start"></span><span id="response-hits-items-matches-position-additional-properties-items-start"></span><span id="response-hits-items-matches-position-additional-properties-start"></span><span id="response-hits-items-matches-position-items-start"></span><span id="response-hits-items-matches-position-start"></span><span id="response-hits-matches-position-additional-properties-items-start"></span><span id="response-hits-matches-position-additional-properties-start"></span><span id="response-hits-matches-position-items-start"></span><span id="response-hits-matches-position-start"></span>        start:
          type: integer
          description: |-
            The byte offset where the match begins within the attribute value.
            This is a zero-indexed position from the start of the string.
          minimum: 0
<span id="response-all-of-1-hits-items-matches-position-additional-properties-items-length"></span><span id="response-all-of-1-hits-items-matches-position-additional-properties-length"></span><span id="response-all-of-1-hits-items-matches-position-items-length"></span><span id="response-all-of-1-hits-items-matches-position-length"></span><span id="response-all-of-1-hits-matches-position-additional-properties-items-length"></span><span id="response-all-of-1-hits-matches-position-additional-properties-length"></span><span id="response-all-of-1-hits-matches-position-items-length"></span><span id="response-all-of-1-hits-matches-position-length"></span><span id="response-hits-items-matches-position-additional-properties-items-length"></span><span id="response-hits-items-matches-position-additional-properties-length"></span><span id="response-hits-items-matches-position-items-length"></span><span id="response-hits-items-matches-position-length"></span><span id="response-hits-matches-position-additional-properties-items-length"></span><span id="response-hits-matches-position-additional-properties-length"></span><span id="response-hits-matches-position-items-length"></span><span id="response-hits-matches-position-length"></span>        length:
          type: integer
          description: |-
            The length in bytes of the matched text. Combined with `start`, this
            defines the exact substring that matched the query term.
          minimum: 0
<span id="response-all-of-1-hits-items-matches-position-additional-properties-indices"></span><span id="response-all-of-1-hits-items-matches-position-additional-properties-items-indices"></span><span id="response-all-of-1-hits-items-matches-position-indices"></span><span id="response-all-of-1-hits-items-matches-position-items-indices"></span><span id="response-all-of-1-hits-matches-position-additional-properties-indices"></span><span id="response-all-of-1-hits-matches-position-additional-properties-items-indices"></span><span id="response-all-of-1-hits-matches-position-indices"></span><span id="response-all-of-1-hits-matches-position-items-indices"></span><span id="response-hits-items-matches-position-additional-properties-indices"></span><span id="response-hits-items-matches-position-additional-properties-items-indices"></span><span id="response-hits-items-matches-position-indices"></span><span id="response-hits-items-matches-position-items-indices"></span><span id="response-hits-matches-position-additional-properties-indices"></span><span id="response-hits-matches-position-additional-properties-items-indices"></span><span id="response-hits-matches-position-indices"></span><span id="response-hits-matches-position-items-indices"></span>        indices:
          type:
            - array
            - 'null'
<span id="response-all-of-1-hits-items-matches-position-additional-properties-indices-items"></span><span id="response-all-of-1-hits-items-matches-position-additional-properties-items-indices-items"></span><span id="response-all-of-1-hits-items-matches-position-indices-items"></span><span id="response-all-of-1-hits-items-matches-position-items-indices-items"></span><span id="response-all-of-1-hits-matches-position-additional-properties-indices-items"></span><span id="response-all-of-1-hits-matches-position-additional-properties-items-indices-items"></span><span id="response-all-of-1-hits-matches-position-indices-items"></span><span id="response-all-of-1-hits-matches-position-items-indices-items"></span><span id="response-hits-items-matches-position-additional-properties-indices-items"></span><span id="response-hits-items-matches-position-additional-properties-items-indices-items"></span><span id="response-hits-items-matches-position-indices-items"></span><span id="response-hits-items-matches-position-items-indices-items"></span><span id="response-hits-matches-position-additional-properties-indices-items"></span><span id="response-hits-matches-position-additional-properties-items-indices-items"></span><span id="response-hits-matches-position-indices-items"></span><span id="response-hits-matches-position-items-indices-items"></span>          items:
            type: integer
            minimum: 0
          description: |-
            Byte indices of individual matched characters when the match spans
            multiple positions (e.g., for prefix matches). This is `null` for
            simple contiguous matches.
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