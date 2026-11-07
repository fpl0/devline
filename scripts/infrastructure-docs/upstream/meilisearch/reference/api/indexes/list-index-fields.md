# List index fields

> Returns a paginated list of fields in the index with their metadata: whether they are displayed, searchable, sortable, filterable, distinct, have a custom ranking rule (asc/desc), and for filterable fields the sort order for facet values.



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
  /indexes/{index_uid}/fields:
    post:
      tags:
        - Indexes
      summary: List index fields
      description: &gt;-
        Returns a paginated list of fields in the index with their metadata:
        whether they are displayed, searchable, sortable, filterable, distinct,
        have a custom ranking rule (asc/desc), and for filterable fields the
        sort order for facet values.
      operationId: post_index_fields
      parameters:
        - name: index_uid
<span id="parameter-index-uid"></span>          in: path
          description: Unique identifier of the index whose fields to list.
          required: true
          schema:
            type: string
          example: movies
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ListFields'
        required: true
      responses:
        '200':
          description: ''
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/PaginationView_Field'
              example:
                results:
                  - name: title
                    displayed:
                      enabled: true
                    searchable:
                      enabled: true
                    sortable:
                      enabled: true
                    distinct:
                      enabled: false
                    rankingRule:
                      enabled: false
                      order: []
                    filterable:
                      enabled: false
                      sortBy: count
                      facetSearch: false
                      equality: false
                      comparison: false
                    localized:
                      locales: []
                  - name: genre
                    displayed:
                      enabled: true
                    searchable:
                      enabled: false
                    sortable:
                      enabled: false
                    distinct:
                      enabled: false
                    rankingRule:
                      enabled: false
                      order: []
                    filterable:
                      enabled: true
                      sortBy: alpha
                      facetSearch: true
                      equality: true
                      comparison: false
                    localized:
                      locales: []
                offset: 0
                limit: 20
                total: 2
        '401':
          description: Missing or invalid authorization header.
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
            - fields.post
            - fields.*
            - '*'
      x-codeSamples:
        - lang: cURL
          source: |-
            curl \
              -X POST 'MEILISEARCH_URL/indexes/INDEX_NAME/fields' \
              -H 'Authorization: Bearer API_KEY' \
              -H 'Content-Type: application/json'
components:
  schemas:
    ListFields:
      type: object
      properties:
<span id="body-offset"></span>        offset:
          type: integer
          description: Number of fields to skip. Defaults to 0.
          minimum: 0
<span id="body-limit"></span>        limit:
          type: integer
          description: Maximum number of fields to return. Defaults to 20.
          minimum: 0
<span id="body-filter"></span>        filter:
          oneOf:
<span id="body-filter-one-of-0"></span>            - type: 'null'
<span id="body-filter-one-of-1"></span>            - $ref: '#/components/schemas/ListFieldsFilter'
              description: &gt;-
                Optional filter to restrict which fields are returned (e.g. by
                attribute patterns or by capability: displayed, searchable,
                sortable, filterable, etc.).
    PaginationView_Field:
      type: object
      required:
        - results
        - offset
        - limit
        - total
      properties:
<span id="response-results"></span>        results:
          type: array
<span id="response-results-items"></span>          items:
            type: object
            required:
              - name
              - displayed
              - searchable
              - sortable
              - distinct
              - rankingRule
              - filterable
              - localized
            properties:
<span id="response-results-items-name"></span><span id="response-results-name"></span>              name:
                type: string
<span id="response-results-displayed"></span><span id="response-results-items-displayed"></span>              displayed:
                $ref: '#/components/schemas/FieldDisplayConfig'
<span id="response-results-items-searchable"></span><span id="response-results-searchable"></span>              searchable:
                $ref: '#/components/schemas/FieldSearchConfig'
<span id="response-results-items-sortable"></span><span id="response-results-sortable"></span>              sortable:
                $ref: '#/components/schemas/FieldSortableConfig'
<span id="response-results-distinct"></span><span id="response-results-items-distinct"></span>              distinct:
                $ref: '#/components/schemas/FieldDistinctConfig'
<span id="response-results-items-ranking-rule"></span><span id="response-results-items-rankingrule"></span><span id="response-results-ranking-rule"></span><span id="response-results-rankingrule"></span>              rankingRule:
                $ref: '#/components/schemas/FieldRankingRuleConfig'
<span id="response-results-filterable"></span><span id="response-results-items-filterable"></span>              filterable:
                $ref: '#/components/schemas/FieldFilterableConfig'
<span id="response-results-items-localized"></span><span id="response-results-localized"></span>              localized:
                $ref: '#/components/schemas/FieldLocalizedConfig'
          description: Items for the current page.
<span id="response-offset"></span>        offset:
          type: integer
          description: Number of items skipped.
          minimum: 0
<span id="response-limit"></span>        limit:
          type: integer
          description: Maximum number of items returned.
          minimum: 0
<span id="response-total"></span>        total:
          type: integer
          description: Total number of items matching the query.
          minimum: 0
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
    ListFieldsFilter:
      type: object
      description: &gt;-
        Filter fields by attribute name patterns or by capability (displayed,
        searchable, sortable, etc.). All criteria are ANDed.
      properties:
<span id="body-filter-attribute-patterns"></span><span id="body-filter-attributepatterns"></span><span id="body-filter-one-of-1-attribute-patterns"></span><span id="body-filter-one-of-1-attributepatterns"></span>        attributePatterns:
          oneOf:
<span id="body-filter-attribute-patterns-one-of-0"></span><span id="body-filter-one-of-1-attribute-patterns-one-of-0"></span>            - type: 'null'
<span id="body-filter-attribute-patterns-one-of-1"></span><span id="body-filter-one-of-1-attribute-patterns-one-of-1"></span>            - $ref: '#/components/schemas/AttributePatterns'
              description: &gt;-
                Only include fields whose names match these patterns (e.g.
                `["title", "desc*"]`).
<span id="body-filter-displayed"></span><span id="body-filter-one-of-1-displayed"></span>        displayed:
          type:
            - boolean
            - 'null'
          description: &gt;-
            Only include fields that are displayed (true) or not displayed
            (false) in search results.
<span id="body-filter-one-of-1-searchable"></span><span id="body-filter-searchable"></span>        searchable:
          type:
            - boolean
            - 'null'
          description: &gt;-
            Only include fields that are searchable (true) or not searchable
            (false).
<span id="body-filter-one-of-1-sortable"></span><span id="body-filter-sortable"></span>        sortable:
          type:
            - boolean
            - 'null'
          description: &gt;-
            Only include fields that are sortable (true) or not sortable
            (false).
<span id="body-filter-distinct"></span><span id="body-filter-one-of-1-distinct"></span>        distinct:
          type:
            - boolean
            - 'null'
          description: &gt;-
            Only include fields that are used as distinct attribute (true) or
            not (false).
<span id="body-filter-one-of-1-ranking-rule"></span><span id="body-filter-one-of-1-rankingrule"></span><span id="body-filter-ranking-rule"></span><span id="body-filter-rankingrule"></span>        rankingRule:
          type:
            - boolean
            - 'null'
          description: &gt;-
            Only include fields that have a custom ranking rule (asc/desc)
            (true) or not (false).
<span id="body-filter-filterable"></span><span id="body-filter-one-of-1-filterable"></span>        filterable:
          type:
            - boolean
            - 'null'
          description: &gt;-
            Only include fields that are filterable (true) or not filterable
            (false).
    FieldDisplayConfig:
      type: object
      required:
        - enabled
      properties:
<span id="response-results-displayed-enabled"></span><span id="response-results-items-displayed-enabled"></span>        enabled:
          type: boolean
    FieldSearchConfig:
      type: object
      required:
        - enabled
      properties:
<span id="response-results-items-searchable-enabled"></span><span id="response-results-searchable-enabled"></span>        enabled:
          type: boolean
    FieldSortableConfig:
      type: object
      required:
        - enabled
      properties:
<span id="response-results-items-sortable-enabled"></span><span id="response-results-sortable-enabled"></span>        enabled:
          type: boolean
    FieldDistinctConfig:
      type: object
      required:
        - enabled
      properties:
<span id="response-results-distinct-enabled"></span><span id="response-results-items-distinct-enabled"></span>        enabled:
          type: boolean
    FieldRankingRuleConfig:
      type: object
      required:
        - enabled
      properties:
<span id="response-results-items-ranking-rule-enabled"></span><span id="response-results-ranking-rule-enabled"></span>        enabled:
          type: boolean
<span id="response-results-items-ranking-rule-order"></span><span id="response-results-ranking-rule-order"></span>        order:
          type: array
<span id="response-results-items-ranking-rule-order-items"></span><span id="response-results-ranking-rule-order-items"></span>          items:
            type: string
    FieldFilterableConfig:
      type: object
      required:
        - enabled
        - sortBy
        - facetSearch
        - equality
        - comparison
      properties:
<span id="response-results-filterable-enabled"></span><span id="response-results-items-filterable-enabled"></span>        enabled:
          type: boolean
<span id="response-results-filterable-sort-by"></span><span id="response-results-filterable-sortby"></span><span id="response-results-items-filterable-sort-by"></span><span id="response-results-items-filterable-sortby"></span>        sortBy:
          $ref: '#/components/schemas/FacetValuesSort'
<span id="response-results-filterable-facet-search"></span><span id="response-results-filterable-facetsearch"></span><span id="response-results-items-filterable-facet-search"></span><span id="response-results-items-filterable-facetsearch"></span>        facetSearch:
          type: boolean
<span id="response-results-filterable-equality"></span><span id="response-results-items-filterable-equality"></span>        equality:
          type: boolean
<span id="response-results-filterable-comparison"></span><span id="response-results-items-filterable-comparison"></span>        comparison:
          type: boolean
    FieldLocalizedConfig:
      type: object
      required:
        - locales
      properties:
<span id="response-results-items-localized-locales"></span><span id="response-results-localized-locales"></span>        locales:
          type: array
<span id="response-results-items-localized-locales-items"></span><span id="response-results-localized-locales-items"></span>          items:
            type: string
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
    AttributePatterns:
      type: array
<span id="body-filter-attribute-patterns-items"></span><span id="body-filter-attribute-patterns-one-of-1-items"></span><span id="body-filter-one-of-1-attribute-patterns-items"></span><span id="body-filter-one-of-1-attribute-patterns-one-of-1-items"></span>      items:
        type: string
      description: &gt;-
        A collection of patterns used to match attribute names. Patterns can
        include wildcards (`*`) for flexible matching. For example, `title`
        matches exactly, `overview_*` matches any attribute starting with
        `overview_`, and `*_date` matches any attribute ending with `_date`.
      examples:
        - '["*", "title", "description_*", "*_date", "*content*"]'
    FacetValuesSort:
      type: string
      enum:
        - alpha
        - count
        - sampledCount
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