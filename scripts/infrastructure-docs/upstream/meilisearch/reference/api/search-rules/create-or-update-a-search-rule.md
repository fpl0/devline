# Create or update a search rule

> Partially update a search rule by replacing the provided fields. If the rule doesn't exist, it will be created.



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
  /dynamic-search-rules/{uid}:
    patch:
      tags:
        - Search rules
      summary: Create or update a search rule
      description: &gt;-
        Partially update a search rule by replacing the provided fields. If the
        rule doesn't exist, it will be created.
      operationId: update_or_create_rule
      parameters:
        - name: uid
<span id="parameter-uid"></span>          in: path
          description: Unique identifier of the search rule.
          required: true
          schema:
            type: string
          example: black-friday
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/DynamicSearchRuleUpdateRequest'
        required: true
      responses:
        '202':
          description: Task enqueued to update dynamic search rule.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/SummarizedTaskView'
              example:
                taskUid: 147
                indexUid: null
                status: enqueued
                type: dsrUpdate
                enqueuedAt: '2024-08-08T17:05:55.791772Z'
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
      security:
        - Bearer:
            - dynamicSearchRules.update
            - dynamicSearchRules.*
            - '*'
      x-codeSamples:
        - lang: cURL
          source: |-
            curl \
              -X PATCH 'MEILISEARCH_URL/dynamic-search-rules/black-friday' \
              -H 'Authorization: Bearer API_KEY' \
              -H 'Content-Type: application/json' \
              --data-binary '{
                "description": "Black Friday 2025 rules",
                "precedence": 10,
                "active": true,
                "conditions": {
                  "query": {
                    "isEmpty": true
                  },
                  "time": {
                    "start": "2025-11-28T00:00:00Z",
                    "end": "2025-11-28T23:59:59Z"
                  }
                },
                "actions": {
                  "pin": [
                    {
                      "indexUid": "products",
                      "id": "123",
                      "position": 1
                    }
                  ]
                }
              }'
        - lang: JS
          source: |-
            client.updateDynamicSearchRule('RULE_UID', {
              description: 'Promote billing help for invoice searches',
              precedence: 1,
              active: true,
              conditions: {
                query: { words: 'invoice' }
              },
              actions: {
                pin: [{ id: '299537', position: 0, indexUid: 'movies' }]
              }
            })
        - lang: PHP
          source: |-
            $client-&gt;updateDynamicSearchRule(
              (new \Meilisearch\Contracts\UpdateDynamicSearchRuleQuery('RULE_UID'))
                -&gt;setActions([
                  'pin' =&gt; [
                    [
                      'id' =&gt; '299537',
                      'position' =&gt; 0,
                      'indexUid' =&gt; 'movies'
                    ]
                  ]
                ])
            );
        - lang: Python
          source: |-
            client.update_dynamic_search_rule('black-friday', {
              'description': 'Black Friday 2025 rules',
              'precedence': 10,
              'active': True,
              'conditions': {
                'query': {'isEmpty': True},
                'time': {
                  'start': '2025-11-28T00:00:00Z',
                  'end': '2025-11-28T23:59:59Z'
                }
              },
              'actions': {
                'pin': [
                  {'indexUid': 'products', 'id': '123', 'position': 1}
                ]
              }
            })
        - lang: Java
          source: &gt;-
            Map&lt;String, Object&gt; conditions = Map.of(
                "query", Map.of("isEmpty", true),
                "time", Map.of("start", "2025-11-28T00:00:00Z", "end", "2025-11-28T23:59:59Z"));

            List&lt;Map&lt;String, Object&gt;&gt; actions = List.of(
                Map.of(
                    "selector", Map.of("indexUid", "products", "id", "123"),
                    "action", Map.of("type", "pin", "position", 1)));

            DynamicSearchRule rule = new DynamicSearchRule(
                "black-friday", "Black Friday 2025 rules", 10, true, conditions, actions);

            TaskInfo task = client.updateDynamicSearchRule("black-friday",
            rule);
        - lang: Go
          source: &gt;-
            active := true

            isEmpty := true

            startTime, _ := time.Parse(time.RFC3339, "2025-11-28T00:00:00Z")

            endTime, _ := time.Parse(time.RFC3339, "2025-11-28T23:59:59Z")

            client.UpdateSearchRule("black-friday",
            &amp;meilisearch.SearchRulesRequest{
              Description: "Black Friday 2025 rules",
              Priority: 10,
              Active: &amp;active,
              Conditions: &amp;meilisearch.Conditions{
                Query: &amp;meilisearch.QueryCondition{
                  IsEmpty: &amp;isEmpty,
                },
                Time: &amp;meilisearch.TimeCondition{
                  Start: &amp;startTime,
                  End: &amp;endTime,
                },
              },
              Actions: []meilisearch.Action{
                {
                  Selector: meilisearch.Selector{
                    IndexUid: "products",
                    ID:       "123",
                  },
                  Action: meilisearch.ActionDef{
                    Type:     "pin",
                    Position: 1,
                  },
                },
              },
            })
        - lang: C#
          source: &gt;-
            var patchQuery = new PatchDynamicSearchRule

            {
              Description = "Promote billing help for invoice searches",
              Precedence = 1,
              Active = true,
              Conditions = new DynamicSearchRuleConditions
              {
                Query = new QueryCondition { Words = "invoice" }
              },
              Actions = new DSRActions
              {
                Pin = new[]
                {
                  new DSRPin { Id = "299537", Position = 0, IndexUid = "movies" }
                }
              }
            };

            await client.CreateOrUpdateDynamicSearchRuleAsync("RULE_UID",
            patchQuery);
components:
  schemas:
    DynamicSearchRuleUpdateRequest:
      type: object
      properties:
<span id="body-description"></span>        description:
          type:
            - string
            - 'null'
          description: Human-readable description of the dynamic search rule.
<span id="body-precedence"></span>        precedence:
          type:
            - integer
            - 'null'
          format: u-int64
          description: &gt;-
            Precedence of the dynamic search rule. Lower numeric values take
            precedence over higher

            ones. If omitted, the rule is treated as having the lowest
            precedence. This precedence is

            used to resolve conflicts between matching rules:

            - If the same document is selected by multiple rules, the smallest
            `priority` number wins

            - If different documents are pinned to the same position, they are
            ordered by ascending `priority`
          minimum: 0
<span id="body-active"></span>        active:
          type:
            - boolean
            - 'null'
          description: Whether the dynamic search rule is active.
<span id="body-conditions"></span>        conditions:
          oneOf:
<span id="body-conditions-one-of-0"></span>            - type: 'null'
<span id="body-conditions-one-of-1"></span>            - $ref: '#/components/schemas/Conditions'
              description: &gt;-
                Conditions that must match before the dynamic search rule
                applies.
<span id="body-actions"></span>        actions:
          oneOf:
<span id="body-actions-one-of-0"></span>            - type: 'null'
<span id="body-actions-one-of-1"></span>            - $ref: '#/components/schemas/RuleActions'
              description: Actions to apply when the dynamic search rule matches.
    SummarizedTaskView:
      type: object
      description: A summarized view of a task, returned when a task is enqueued
      required:
        - taskUid
        - status
        - type
        - enqueuedAt
      properties:
<span id="response-task-uid"></span><span id="response-taskuid"></span>        taskUid:
          type: integer
          format: u-int32
          description: Unique sequential identifier of the task.
          minimum: 0
<span id="response-index-uid"></span><span id="response-indexuid"></span>        indexUid:
          type:
            - string
            - 'null'
          description: Unique identifier of the targeted index. Null for global tasks.
<span id="response-status"></span>        status:
          $ref: '#/components/schemas/Status'
          description: |-
            Status of the task. Possible values are `enqueued`, `processing`,
            `succeeded`, `failed`, and `canceled`.
<span id="response-type"></span>        type:
          $ref: '#/components/schemas/Kind'
          description: Type of operation performed by the task.
<span id="response-enqueued-at"></span><span id="response-enqueuedat"></span>        enqueuedAt:
          type: string
          format: date-time
          description: Date and time when the task was enqueued.
<span id="response-custom-metadata"></span><span id="response-custommetadata"></span>        customMetadata:
          type:
            - string
            - 'null'
          description: &gt;-
            Custom metadata attached to this task at creation. Use it to
            associate

            tasks with external systems or add application-specific information.
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
    Conditions:
      type: object
      properties:
<span id="body-conditions-one-of-1-time"></span><span id="body-conditions-time"></span>        time:
          oneOf:
<span id="body-conditions-one-of-1-time-one-of-0"></span><span id="body-conditions-time-one-of-0"></span>            - type: 'null'
<span id="body-conditions-one-of-1-time-one-of-1"></span><span id="body-conditions-time-one-of-1"></span>            - $ref: '#/components/schemas/TimeCondition'
              description: Time range where the rule is active
<span id="body-conditions-one-of-1-query"></span><span id="body-conditions-query"></span>        query:
          oneOf:
<span id="body-conditions-one-of-1-query-one-of-0"></span><span id="body-conditions-query-one-of-0"></span>            - type: 'null'
<span id="body-conditions-one-of-1-query-one-of-1"></span><span id="body-conditions-query-one-of-1"></span>            - $ref: '#/components/schemas/QueryCondition'
              description: &gt;-
                Conditions on the search query that determines whether the rule
                is active
<span id="body-conditions-filter"></span><span id="body-conditions-one-of-1-filter"></span>        filter:
          oneOf:
<span id="body-conditions-filter-one-of-0"></span><span id="body-conditions-one-of-1-filter-one-of-0"></span>            - type: 'null'
<span id="body-conditions-filter-one-of-1"></span><span id="body-conditions-one-of-1-filter-one-of-1"></span>            - $ref: '#/components/schemas/FilterCondition'
              description: &gt;-
                Conditions on the values matching the filter of the search query
                that determines whether the rule is active
    RuleActions:
      type: object
      description: List of actions to apply when this rule is active for the query.
      properties:
<span id="body-actions-one-of-1-pin"></span><span id="body-actions-pin"></span>        pin:
          type: array
<span id="body-actions-one-of-1-pin-items"></span><span id="body-actions-pin-items"></span>          items:
            $ref: '#/components/schemas/PinAction'
          description: Pins a selected document.
<span id="body-actions-one-of-1-scale"></span><span id="body-actions-scale"></span>        scale:
          type: array
<span id="body-actions-one-of-1-scale-items"></span><span id="body-actions-scale-items"></span>          items:
            $ref: '#/components/schemas/ScaleAction'
          description: Applies a multiplicative factor to the score of selected documents.
    Status:
      type: string
      description: The status of a task.
      enum:
        - enqueued
        - processing
        - succeeded
        - failed
        - canceled
      example: processing
    Kind:
      type: string
      description: The type of the task.
      enum:
        - documentAdditionOrUpdate
        - documentEdition
        - documentDeletion
        - settingsUpdate
        - indexCreation
        - indexDeletion
        - indexUpdate
        - indexSwap
        - taskCancelation
        - taskDeletion
        - dumpCreation
        - snapshotCreation
        - export
        - upgradeDatabase
        - indexCompaction
        - networkTopologyChange
        - dsrUpdate
        - dsrClear
      example: documentAdditionOrUpdate
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
    TimeCondition:
      type: object
      properties:
<span id="body-conditions-one-of-1-time-one-of-1-start"></span><span id="body-conditions-one-of-1-time-start"></span><span id="body-conditions-time-one-of-1-start"></span><span id="body-conditions-time-start"></span>        start:
          type:
            - string
            - 'null'
          format: date-time
          description: |-
            Start of the time range where this rule can be considered active.

            Specify as a RFC3339 datetime.
<span id="body-conditions-one-of-1-time-end"></span><span id="body-conditions-one-of-1-time-one-of-1-end"></span><span id="body-conditions-time-end"></span><span id="body-conditions-time-one-of-1-end"></span>        end:
          type:
            - string
            - 'null'
          format: date-time
          description: |-
            End of the time range where this rule can be considered active.

            Specify as a RFC3339 datetime.
    QueryCondition:
      type: object
      properties:
<span id="body-conditions-one-of-1-query-is-empty"></span><span id="body-conditions-one-of-1-query-isempty"></span><span id="body-conditions-one-of-1-query-one-of-1-is-empty"></span><span id="body-conditions-one-of-1-query-one-of-1-isempty"></span><span id="body-conditions-query-is-empty"></span><span id="body-conditions-query-isempty"></span><span id="body-conditions-query-one-of-1-is-empty"></span><span id="body-conditions-query-one-of-1-isempty"></span>        isEmpty:
          type:
            - boolean
            - 'null'
          description: &gt;-
            If present and non-null, specifies either:


            - That this rule can only be active when the search query is empty

            - That this rule can only be active when the search query is
            non-empty (contains at least one word)
<span id="body-conditions-one-of-1-query-one-of-1-words"></span><span id="body-conditions-one-of-1-query-words"></span><span id="body-conditions-query-one-of-1-words"></span><span id="body-conditions-query-words"></span>        words:
          type:
            - string
            - 'null'
          description: &gt;-
            If present and non-null, specifies that the rule can only be active
            if all the specified words are

            present in the search query.
    FilterCondition:
      type: object
      properties:
<span id="body-conditions-filter-one-of-1-values"></span><span id="body-conditions-filter-values"></span><span id="body-conditions-one-of-1-filter-one-of-1-values"></span><span id="body-conditions-one-of-1-filter-values"></span>        values:
          type: object
<span id="body-conditions-filter-one-of-1-values-additional-properties"></span><span id="body-conditions-filter-values-additional-properties"></span><span id="body-conditions-one-of-1-filter-one-of-1-values-additional-properties"></span><span id="body-conditions-one-of-1-filter-values-additional-properties"></span>          additionalProperties: {}
          propertyNames:
            type: string
    PinAction:
      type: object
      description: An action that pins a selected document.
      required:
        - id
        - position
      properties:
<span id="body-actions-one-of-1-pin-index-uid"></span><span id="body-actions-one-of-1-pin-indexuid"></span><span id="body-actions-one-of-1-pin-items-index-uid"></span><span id="body-actions-one-of-1-pin-items-indexuid"></span><span id="body-actions-pin-index-uid"></span><span id="body-actions-pin-indexuid"></span><span id="body-actions-pin-items-index-uid"></span><span id="body-actions-pin-items-indexuid"></span>        indexUid:
          type:
            - string
            - 'null'
          description: &gt;-
            Index name.


            For the action to select any document, when this parameter is
            provided,

            the index of the query must match the provided parameter.
<span id="body-actions-one-of-1-pin-id"></span><span id="body-actions-one-of-1-pin-items-id"></span><span id="body-actions-pin-id"></span><span id="body-actions-pin-items-id"></span>        id:
          type: string
          description: &gt;-
            Document ID of the document to select.


            Only the document whose [primary
            key](https://www.meilisearch.com/docs/learn/getting_started/primary_key)
            value

            matches the specified id will be selected by the action.


            If there is no such document in the index of the query, then no
            documents will be selected and no pinning will occur.
<span id="body-actions-one-of-1-pin-items-position"></span><span id="body-actions-one-of-1-pin-position"></span><span id="body-actions-pin-items-position"></span><span id="body-actions-pin-position"></span>        position:
          type: integer
          format: u-int32
          description: Position at which the document should be pinned.
          minimum: 0
    ScaleAction:
      type: object
      description: &gt;-
        An action that applies a multiplicative factor to the score of selected
        documents.
      required:
        - weight
      properties:
<span id="body-actions-one-of-1-scale-index-uid"></span><span id="body-actions-one-of-1-scale-indexuid"></span><span id="body-actions-one-of-1-scale-items-index-uid"></span><span id="body-actions-one-of-1-scale-items-indexuid"></span><span id="body-actions-scale-index-uid"></span><span id="body-actions-scale-indexuid"></span><span id="body-actions-scale-items-index-uid"></span><span id="body-actions-scale-items-indexuid"></span>        indexUid:
          type:
            - string
            - 'null'
          description: &gt;-
            List of index patterns.


            For the action to select any document, when this parameter is
            provided,

            the index of the query must match the provided parameter.
<span id="body-actions-one-of-1-scale-ids"></span><span id="body-actions-one-of-1-scale-items-ids"></span><span id="body-actions-scale-ids"></span><span id="body-actions-scale-items-ids"></span>        ids:
          type:
            - array
            - 'null'
<span id="body-actions-one-of-1-scale-ids-items"></span><span id="body-actions-one-of-1-scale-items-ids-items"></span><span id="body-actions-scale-ids-items"></span><span id="body-actions-scale-items-ids-items"></span>          items:
            type: string
          description: &gt;-
            Array of specific document IDs to select.


            Only documents whose [primary
            key](https://www.meilisearch.com/docs/learn/getting_started/primary_key)
            value

            matches the specified ids will be selected by the action.


            If `filter` is also specified,

            the documents must also satisfy the filter to be selected.
<span id="body-actions-one-of-1-scale-filter"></span><span id="body-actions-one-of-1-scale-items-filter"></span><span id="body-actions-scale-filter"></span><span id="body-actions-scale-items-filter"></span>        filter:
          description: &gt;-
            Filter expression to select documents. Attributes must be added to
            the

            `filterableAttributes` index setting before they can be used in
            filters.

            Accepts a string or an array of arrays of strings for AND/OR
            combinations.


            Only documents matching the specified filter will be selected.


            If `ids` is also specified,

            the documents matching the filter must also have their primary key
            part of the `ids`

            list to be selected.


            If the filter cannot be evaluated for the current index due to
            referencing attributes

            that are not filterable, then no document will be applied for this
            action.
<span id="body-actions-one-of-1-scale-items-weight"></span><span id="body-actions-one-of-1-scale-weight"></span><span id="body-actions-scale-items-weight"></span><span id="body-actions-scale-weight"></span>        weight:
          type: number
          format: double
          description: |-
            Scale factor for selected documents.

            - Set it &gt;1.0 to boost the selected documents.
            - Set it &lt;1.0 to deboost the selected documents.
            - Set it =0.0 to hide the selected documents.
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