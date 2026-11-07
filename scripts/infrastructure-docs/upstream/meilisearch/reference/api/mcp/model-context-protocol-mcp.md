# Model context protocol (MCP)

> The `/mcp` route exposes an [MCP server](https://www.meilisearch.com/docs/getting_started/integrations/mcp) for
LLM applications to search Meilisearch.



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
  /mcp:
    post:
      tags:
        - MCP
      summary: Model context protocol (MCP)
      description: &gt;-
        The `/mcp` route exposes an [MCP
        server](https://www.meilisearch.com/docs/getting_started/integrations/mcp)
        for

        LLM applications to search Meilisearch.
      operationId: mcp
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/McpQuery'
        required: true
      responses:
        '200':
          description: Stream of batches changes.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/McpResponse'
              example:
                jsonrpc: '2.0'
                id: 42
                result:
                  resultType: complete
                  supportedVersions:
                    - '2026-07-28'
                  capabilities:
                    tools: {}
                  _meta:
                    io.modelcontextprotocol/serverInfo:
                      name: Meilisearch
                      version: 1.52.0
                  instructions: &gt;-
                    This is a Meilisearch instance that is capable of returning
                    documents based on a search query.
                  ttlMs: 3600000
                  cacheScope: public
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
      security: []
      x-codeSamples:
        - lang: cURL
          source: |-
            curl \
              -X POST 'MEILISEARCH_URL/mcp' \
              -H 'Authorization: Bearer API_KEY' \
              -H 'Content-Type: application/json' \
              -H 'Accept: application/json, text/event-stream' \
              --data-binary '{
                "jsonrpc": "2.0",
                "id": 1,
                "method": "server/discover",
                "params": {
                  "_meta": {
                    "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                    "io.modelcontextprotocol/clientInfo": {
                      "name": "ExampleClient",
                      "version": "1.0.0"
                    }
                  }
                }
              }'
components:
  schemas:
    McpQuery:
      type: object
      required:
        - jsonrpc
        - id
        - method
        - params
      properties:
<span id="body-jsonrpc"></span>        jsonrpc:
          type: string
          description: &gt;-
            Defines the version of a JSON-RPC request.


            You can find more information about this field on [the JSON-RPC
            specification](https://www.jsonrpc.org/specification#request_object).
<span id="body-id"></span>        id:
          $ref: '#/components/schemas/RequestId'
          description: &gt;-
            Defines the id of a JSON-RPC request.


            You can find more information about this field on [the JSON-RPC
            specification](https://www.jsonrpc.org/specification#request_object).
<span id="body-method"></span>        method:
          type: string
          description: &gt;-
            The method to call using the JSON-RPC format.


            You can find more information about this field on [the JSON-RPC
            specification](https://www.jsonrpc.org/specification#request_object).
<span id="body-params"></span>        params:
          $ref: '#/components/schemas/ParamsWithMeta'
          description: &gt;-
            The parameters to call the method with following the JSON-RPC
            format.


            You can find more information about this field on [the JSON-RPC
            specification](https://www.jsonrpc.org/specification#request_object).
    McpResponse:
      type: object
      required:
        - jsonrpc
        - id
      properties:
<span id="response-jsonrpc"></span>        jsonrpc:
          type: string
          description: The JSON-RPC version.
<span id="response-id"></span>        id:
          $ref: '#/components/schemas/RequestId'
          description: The JSON-RPC request ID.
<span id="response-result"></span>        result:
          oneOf:
<span id="response-result-one-of-0"></span>            - type: 'null'
<span id="response-result-one-of-1"></span>            - $ref: '#/components/schemas/McpResult'
              description: The JSON-RPC result.
<span id="response-error"></span>        error:
          oneOf:
<span id="response-error-one-of-0"></span>            - type: 'null'
<span id="response-error-one-of-1"></span>            - $ref: '#/components/schemas/McpError'
              description: The JSON-RPC error.
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
    RequestId:
      oneOf:
<span id="body-id-one-of-0"></span><span id="response-id-one-of-0"></span>        - type: integer
<span id="body-id-one-of-1"></span><span id="response-id-one-of-1"></span>        - type: string
      description: &gt;-
        The request ID MUST NOT match the ID of any other request the sender has
        issued and not yet received a response for
    ParamsWithMeta:
      type: object
      properties:
<span id="body-params--meta"></span><span id="body-params-meta"></span>        _meta:
          oneOf:
<span id="body-params-meta-one-of-0"></span>            - type: 'null'
<span id="body-params-meta-one-of-1"></span>            - $ref: '#/components/schemas/McpClientMeta'
              description: &gt;-
                The _meta property used by the MCP protocol.


                You can find more information about this field on [the MCP
                specification](https://modelcontextprotocol.io/specification/2026-07-28/basic/index#_meta).
<span id="body-params-name"></span>        name:
          type:
            - string
            - 'null'
          description: &gt;-
            The tool name to call used by the MCP protocol.


            You can find more information about this field on [the MCP
            specification](https://modelcontextprotocol.io/specification/2026-07-28/server/tools#tool-names).
<span id="body-params-arguments"></span>        arguments:
          description: &gt;-
            The arguments to give to the tool from the MCP protocol.


            You can find more information about this field on [the MCP
            specification](https://modelcontextprotocol.io/specification/2026-07-28/schema#calltoolrequestparams).
    McpResult:
      type: object
      required:
        - resultType
        - ttlMs
        - cacheScope
      properties:
<span id="response-result-one-of-1-result-type"></span><span id="response-result-one-of-1-resulttype"></span><span id="response-result-result-type"></span><span id="response-result-resulttype"></span>        resultType:
          type: string
<span id="response-result-is-error"></span><span id="response-result-iserror"></span><span id="response-result-one-of-1-is-error"></span><span id="response-result-one-of-1-iserror"></span>        isError:
          type:
            - boolean
            - 'null'
<span id="response-result-one-of-1-supported-versions"></span><span id="response-result-one-of-1-supportedversions"></span><span id="response-result-supported-versions"></span><span id="response-result-supportedversions"></span>        supportedVersions:
          type:
            - array
            - 'null'
<span id="response-result-one-of-1-supported-versions-items"></span><span id="response-result-supported-versions-items"></span>          items:
            type: string
          description: &gt;-
            Protocol versions the server supports. The client should choose one
            of these for subsequent requests.
<span id="response-result-one-of-1-tools"></span><span id="response-result-tools"></span>        tools:
          type:
            - array
            - 'null'
<span id="response-result-one-of-1-tools-items"></span><span id="response-result-tools-items"></span>          items:
            $ref: '#/components/schemas/McpToolDefinition'
<span id="response-result-one-of-1-resources"></span><span id="response-result-resources"></span>        resources:
          type:
            - array
            - 'null'
<span id="response-result-one-of-1-resources-items"></span><span id="response-result-resources-items"></span>          items: {}
<span id="response-result-one-of-1-prompts"></span><span id="response-result-prompts"></span>        prompts:
          type:
            - array
            - 'null'
<span id="response-result-one-of-1-prompts-items"></span><span id="response-result-prompts-items"></span>          items: {}
<span id="response-result--meta"></span><span id="response-result-meta"></span><span id="response-result-one-of-1--meta"></span><span id="response-result-one-of-1-meta"></span>        _meta:
          oneOf:
<span id="response-result-meta-one-of-0"></span><span id="response-result-one-of-1-meta-one-of-0"></span>            - type: 'null'
<span id="response-result-meta-one-of-1"></span><span id="response-result-one-of-1-meta-one-of-1"></span>            - $ref: '#/components/schemas/McpServerMeta'
<span id="response-result-capabilities"></span><span id="response-result-one-of-1-capabilities"></span>        capabilities:
          oneOf:
<span id="response-result-capabilities-one-of-0"></span><span id="response-result-one-of-1-capabilities-one-of-0"></span>            - type: 'null'
<span id="response-result-capabilities-one-of-1"></span><span id="response-result-one-of-1-capabilities-one-of-1"></span>            - $ref: '#/components/schemas/McpCapabilities'
              description: &gt;-
                Capabilities the server supports (tools, resources, prompts,
                etc.).
<span id="response-result-content"></span><span id="response-result-one-of-1-content"></span>        content:
          type:
            - array
            - 'null'
<span id="response-result-content-items"></span><span id="response-result-one-of-1-content-items"></span>          items:
            $ref: '#/components/schemas/McpTextContentOutput'
<span id="response-result-one-of-1-structured-content"></span><span id="response-result-one-of-1-structuredcontent"></span><span id="response-result-structured-content"></span><span id="response-result-structuredcontent"></span>        structuredContent: {}
<span id="response-result-instructions"></span><span id="response-result-one-of-1-instructions"></span>        instructions:
          type:
            - string
            - 'null'
          description: &gt;-
            Optional natural-language guidance for LLMs on how to use this
            server effectively.
<span id="response-result-one-of-1-ttl-ms"></span><span id="response-result-one-of-1-ttlms"></span><span id="response-result-ttl-ms"></span><span id="response-result-ttlms"></span>        ttlMs:
          type: integer
          minimum: 0
<span id="response-result-cache-scope"></span><span id="response-result-cachescope"></span><span id="response-result-one-of-1-cache-scope"></span><span id="response-result-one-of-1-cachescope"></span>        cacheScope:
          type: string
    McpError:
      type: object
      required:
        - code
        - message
      properties:
<span id="response-error-code"></span><span id="response-error-one-of-1-code"></span>        code:
          type: integer
          format: int64
<span id="response-error-message"></span><span id="response-error-one-of-1-message"></span>        message:
          type: string
<span id="response-error-data"></span><span id="response-error-one-of-1-data"></span>        data:
          oneOf:
<span id="response-error-data-one-of-0"></span><span id="response-error-one-of-1-data-one-of-0"></span>            - type: 'null'
<span id="response-error-data-one-of-1"></span><span id="response-error-one-of-1-data-one-of-1"></span>            - $ref: '#/components/schemas/McpErrorData'
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
    McpClientMeta:
      type: object
      required:
        - io.modelcontextprotocol/protocolVersion
        - io.modelcontextprotocol/clientInfo
      properties:
<span id="body-params-meta-io-modelcontextprotocol-protocol-version"></span><span id="body-params-meta-io.modelcontextprotocol/protocolversion"></span><span id="body-params-meta-one-of-1-io-modelcontextprotocol-protocol-version"></span><span id="body-params-meta-one-of-1-io.modelcontextprotocol/protocolversion"></span>        io.modelcontextprotocol/protocolVersion:
          type: string
<span id="body-params-meta-io-modelcontextprotocol-client-info"></span><span id="body-params-meta-io.modelcontextprotocol/clientinfo"></span><span id="body-params-meta-one-of-1-io-modelcontextprotocol-client-info"></span><span id="body-params-meta-one-of-1-io.modelcontextprotocol/clientinfo"></span>        io.modelcontextprotocol/clientInfo: {}
<span id="body-params-meta-io-modelcontextprotocol-client-capabilities"></span><span id="body-params-meta-io.modelcontextprotocol/clientcapabilities"></span><span id="body-params-meta-one-of-1-io-modelcontextprotocol-client-capabilities"></span><span id="body-params-meta-one-of-1-io.modelcontextprotocol/clientcapabilities"></span>        io.modelcontextprotocol/clientCapabilities: {}
    McpToolDefinition:
      type: object
      description: &gt;-
        &lt;https://modelcontextprotocol.io/specification/2026-07-28/server/tools#data-types&gt;
      required:
        - name
        - title
        - description
        - inputSchema
      properties:
<span id="response-result-one-of-1-tools-items-name"></span><span id="response-result-one-of-1-tools-name"></span><span id="response-result-tools-items-name"></span><span id="response-result-tools-name"></span>        name:
          type: string
          description: Unique identifier for the tool.
<span id="response-result-one-of-1-tools-items-title"></span><span id="response-result-one-of-1-tools-title"></span><span id="response-result-tools-items-title"></span><span id="response-result-tools-title"></span>        title:
          type: string
          description: Optional human-readable name of the tool for display purposes.
<span id="response-result-one-of-1-tools-description"></span><span id="response-result-one-of-1-tools-items-description"></span><span id="response-result-tools-description"></span><span id="response-result-tools-items-description"></span>        description:
          type: string
          description: Human-readable description of functionality.
<span id="response-result-one-of-1-tools-input-schema"></span><span id="response-result-one-of-1-tools-inputschema"></span><span id="response-result-one-of-1-tools-items-input-schema"></span><span id="response-result-one-of-1-tools-items-inputschema"></span><span id="response-result-tools-input-schema"></span><span id="response-result-tools-inputschema"></span><span id="response-result-tools-items-input-schema"></span><span id="response-result-tools-items-inputschema"></span>        inputSchema:
          description: JSON Schema defining expected parameters.
    McpServerMeta:
      type: object
      required:
        - io.modelcontextprotocol/serverInfo
      properties:
<span id="response-result-meta-io-modelcontextprotocol-server-info"></span><span id="response-result-meta-io.modelcontextprotocol/serverinfo"></span><span id="response-result-meta-one-of-1-io-modelcontextprotocol-server-info"></span><span id="response-result-meta-one-of-1-io.modelcontextprotocol/serverinfo"></span><span id="response-result-one-of-1-meta-io-modelcontextprotocol-server-info"></span><span id="response-result-one-of-1-meta-io.modelcontextprotocol/serverinfo"></span><span id="response-result-one-of-1-meta-one-of-1-io-modelcontextprotocol-server-info"></span><span id="response-result-one-of-1-meta-one-of-1-io.modelcontextprotocol/serverinfo"></span>        io.modelcontextprotocol/serverInfo:
          $ref: '#/components/schemas/ClientServerInfo'
    McpCapabilities:
      type: object
      properties:
<span id="response-result-capabilities-one-of-1-tools"></span><span id="response-result-capabilities-tools"></span><span id="response-result-one-of-1-capabilities-one-of-1-tools"></span><span id="response-result-one-of-1-capabilities-tools"></span>        tools:
          type:
            - object
            - 'null'
<span id="response-result-capabilities-one-of-1-tools-additional-properties"></span><span id="response-result-capabilities-tools-additional-properties"></span><span id="response-result-one-of-1-capabilities-one-of-1-tools-additional-properties"></span><span id="response-result-one-of-1-capabilities-tools-additional-properties"></span>          additionalProperties:
            default: null
          propertyNames:
            default: null
    McpTextContentOutput:
      type: object
      required:
        - type
        - text
      properties:
<span id="response-result-content-items-type"></span><span id="response-result-content-type"></span><span id="response-result-one-of-1-content-items-type"></span><span id="response-result-one-of-1-content-type"></span>        type:
          type: string
<span id="response-result-content-items-text"></span><span id="response-result-content-text"></span><span id="response-result-one-of-1-content-items-text"></span><span id="response-result-one-of-1-content-text"></span>        text:
          type: string
    McpErrorData:
      type: object
      required:
        - statusCode
        - errorName
        - errorType
        - errorLink
      properties:
<span id="response-error-data-one-of-1-status-code"></span><span id="response-error-data-one-of-1-statuscode"></span><span id="response-error-data-status-code"></span><span id="response-error-data-statuscode"></span><span id="response-error-one-of-1-data-one-of-1-status-code"></span><span id="response-error-one-of-1-data-one-of-1-statuscode"></span><span id="response-error-one-of-1-data-status-code"></span><span id="response-error-one-of-1-data-statuscode"></span>        statusCode:
          type: integer
          format: u-int16
          minimum: 0
<span id="response-error-data-error-name"></span><span id="response-error-data-errorname"></span><span id="response-error-data-one-of-1-error-name"></span><span id="response-error-data-one-of-1-errorname"></span><span id="response-error-one-of-1-data-error-name"></span><span id="response-error-one-of-1-data-errorname"></span><span id="response-error-one-of-1-data-one-of-1-error-name"></span><span id="response-error-one-of-1-data-one-of-1-errorname"></span>        errorName:
          type: string
<span id="response-error-data-error-type"></span><span id="response-error-data-errortype"></span><span id="response-error-data-one-of-1-error-type"></span><span id="response-error-data-one-of-1-errortype"></span><span id="response-error-one-of-1-data-error-type"></span><span id="response-error-one-of-1-data-errortype"></span><span id="response-error-one-of-1-data-one-of-1-error-type"></span><span id="response-error-one-of-1-data-one-of-1-errortype"></span>        errorType:
          type: string
<span id="response-error-data-error-link"></span><span id="response-error-data-errorlink"></span><span id="response-error-data-one-of-1-error-link"></span><span id="response-error-data-one-of-1-errorlink"></span><span id="response-error-one-of-1-data-error-link"></span><span id="response-error-one-of-1-data-errorlink"></span><span id="response-error-one-of-1-data-one-of-1-error-link"></span><span id="response-error-one-of-1-data-one-of-1-errorlink"></span>        errorLink:
          type: string
    ClientServerInfo:
      type: object
      required:
        - name
        - version
      properties:
<span id="response-result-meta-io-modelcontextprotocol-server-info-name"></span><span id="response-result-meta-one-of-1-io-modelcontextprotocol-server-info-name"></span><span id="response-result-one-of-1-meta-io-modelcontextprotocol-server-info-name"></span><span id="response-result-one-of-1-meta-one-of-1-io-modelcontextprotocol-server-info-name"></span>        name:
          type: string
<span id="response-result-meta-io-modelcontextprotocol-server-info-version"></span><span id="response-result-meta-one-of-1-io-modelcontextprotocol-server-info-version"></span><span id="response-result-one-of-1-meta-io-modelcontextprotocol-server-info-version"></span><span id="response-result-one-of-1-meta-one-of-1-io-modelcontextprotocol-server-info-version"></span>        version:
          type: string

</code></pre>

This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.