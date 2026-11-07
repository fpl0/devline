# Get task

> Retrieve a single [task](https://www.meilisearch.com/docs/learn/async/asynchronous_operations) by its uid.



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
  /tasks/{task_id}:
    get:
      tags:
        - Async task management
      summary: Get task
      description: &gt;-
        Retrieve a single
        [task](https://www.meilisearch.com/docs/learn/async/asynchronous_operations)
        by its uid.
      operationId: get_task
      parameters:
        - name: task_id
<span id="parameter-task-id"></span>          in: path
          description: The task identifier.
          required: true
          schema:
            type: integer
            format: u-int32
            minimum: 0
          example: 0
      responses:
        '200':
          description: Task successfully retrieved.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/TaskView'
              example:
                uid: 1
                indexUid: movies
                status: succeeded
                type: indexCreation
                canceledBy: null
                details: null
                error: null
                duration: PT1S
                enqueuedAt: '2021-01-01T09:39:00.000000Z'
                startedAt: '2021-01-01T09:39:01.000000Z'
                finishedAt: '2021-01-01T09:39:02.000000Z'
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
          description: The task uid does not exist.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ResponseError'
              example:
                message: Task :taskUid not found.
                code: task_not_found
                type: invalid_request
                link: https://docs.meilisearch.com/errors/#task_not_found
      security:
        - Bearer:
            - tasks.get
            - tasks.*
            - '*'
      x-codeSamples:
        - lang: cURL
          source: |-
            curl \
              -X GET 'MEILISEARCH_URL/tasks/1' \
              -H 'Authorization: Bearer API_KEY'
        - lang: JS
          source: client.tasks.getTask(1)
        - lang: PHP
          source: $client-&gt;getTask(1);
        - lang: Python
          source: client.get_task(1)
        - lang: Java
          source: client.getTask(1);
        - lang: Ruby
          source: client.task(1)
        - lang: Go
          source: client.GetTask(1);
        - lang: C#
          source: TaskInfo task = await client.GetTaskAsync(1);
        - lang: Rust
          source: |-
            let task: Task = client
              .get_task(1)
              .await
              .unwrap();
        - lang: Dart
          source: await client.getTask(1);
        - lang: Swift
          source: |-
            client.getTask(taskUid: 1) { (result) in
                  switch result {
                  case .success(let task):
                      print(task)
                  case .failure(let error):
                      print(error)
                  }
              }
components:
  schemas:
    TaskView:
      type: object
      description: |-
        Represents the current state and details of an asynchronous task.

        Tasks are created when you perform operations like adding documents,
        updating settings, or creating indexes. Use this view to monitor task
        progress and check for errors.
      required:
        - uid
        - status
        - type
        - enqueuedAt
      properties:
<span id="response-uid"></span>        uid:
          type: integer
          format: u-int32
          description: &gt;-
            The unique sequential identifier assigned to this task. Task UIDs
            are

            assigned in order of creation and can be used to retrieve specific

            task information or track task dependencies.
          example: 4312
          minimum: 0
<span id="response-batch-uid"></span><span id="response-batchuid"></span>        batchUid:
          type:
            - integer
            - 'null'
          format: u-int32
          description: &gt;-
            The unique identifier of the batch that processed this task.
            Multiple

            tasks may share the same batch UID if they were processed together

            for efficiency. This is `null` for tasks that haven't been
            processed.
          example: 12
          minimum: 0
<span id="response-index-uid"></span><span id="response-indexuid"></span>        indexUid:
          type:
            - string
            - 'null'
          description: |-
            The unique identifier of the index this task operates on. This is
            `null` for global tasks like `dumpCreation` or `taskDeletion` that
            don't target a specific index.
<span id="response-status"></span>        status:
          $ref: '#/components/schemas/Status'
          description: |-
            The current processing status of the task. Possible values are:
            `enqueued` (waiting), `processing` (executing), `succeeded`,
            `failed`, or `canceled`.
<span id="response-type"></span>        type:
          $ref: '#/components/schemas/Kind'
          description: |-
            The type of operation this task performs. Examples include
            `documentAdditionOrUpdate`, `documentDeletion`, `settingsUpdate`,
            `indexCreation`, `indexDeletion`, `dumpCreation`, etc.
<span id="response-canceled-by"></span><span id="response-canceledby"></span>        canceledBy:
          type:
            - integer
            - 'null'
          format: u-int32
          description: |-
            If this task was canceled, this field contains the UID of the
            `taskCancelation` task that canceled it. This is `null` for tasks
            that were not canceled.
          example: 4326
          minimum: 0
<span id="response-details"></span>        details:
          type:
            - object
            - 'null'
          description: &gt;-
            Contains type-specific information about the task, such as the
            number

            of documents processed, settings that were applied, or filters that

            were used. The structure varies depending on the task type.
<span id="response-error"></span>        error:
          oneOf:
<span id="response-error-one-of-0"></span>            - type: 'null'
<span id="response-error-one-of-1"></span>            - $ref: '#/components/schemas/ResponseError'
              description: &gt;-
                If the task failed, this field contains detailed error
                information

                including an error message, error code, error type, and a link
                to

                documentation. This is `null` for tasks that succeeded or are
                still

                processing.
<span id="response-duration"></span>        duration:
          type:
            - string
            - 'null'
          description: |-
            The total time spent processing this task, formatted as an ISO-8601
            duration (e.g., `PT0.5S` for 0.5 seconds). This is `null` for tasks
            that haven't finished processing yet.
          example: null
<span id="response-enqueued-at"></span><span id="response-enqueuedat"></span>        enqueuedAt:
          type: string
          description: |-
            The timestamp when this task was added to the queue, formatted as an
            RFC 3339 date-time string. All tasks have an enqueued timestamp as
            it's set when the task is created.
          example: 2024-08-08_14:12:09.393Z
<span id="response-started-at"></span><span id="response-startedat"></span>        startedAt:
          type: string
          description: |-
            The timestamp when Meilisearch began processing this task, formatted
            as an RFC 3339 date-time string. This is `null` for tasks that are
            still in the queue waiting to be processed.
          example: 2024-08-08_14:12:09.393Z
<span id="response-finished-at"></span><span id="response-finishedat"></span>        finishedAt:
          type: string
          description: &gt;-
            The timestamp when this task finished processing (whether
            successfully

            or with an error), formatted as an RFC 3339 date-time string. This
            is

            `null` for tasks that haven't finished yet.
          example: 2024-08-08_14:12:09.393Z
<span id="response-network"></span>        network:
          oneOf:
<span id="response-network-one-of-0"></span>            - type: 'null'
<span id="response-network-one-of-1"></span>            - $ref: '#/components/schemas/DbTaskNetwork'
              description: &gt;-
                Network topology information for distributed deployments.
                Contains

                details about which nodes are involved in processing this task.
                This

                is only present when running Meilisearch in a distributed
                config.
<span id="response-custom-metadata"></span><span id="response-custommetadata"></span>        customMetadata:
          type:
            - string
            - 'null'
          description: |-
            Custom metadata string that was attached to this task when it was
            created. This can be used to associate tasks with external systems,
            track task origins, or add any application-specific information.
    ResponseError:
      type: object
      required:
        - message
        - code
        - type
        - link
      properties:
<span id="response-error-message"></span><span id="response-error-one-of-1-message"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-additional-properties-error-message"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-additional-properties-error-one-of-1-message"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-error-message"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-error-one-of-1-message"></span><span id="response-network-one-of-1-remote-tasks-additional-properties-error-message"></span><span id="response-network-one-of-1-remote-tasks-additional-properties-error-one-of-1-message"></span><span id="response-network-one-of-1-remote-tasks-error-message"></span><span id="response-network-one-of-1-remote-tasks-error-one-of-1-message"></span><span id="response-network-remote-tasks-additional-properties-error-message"></span><span id="response-network-remote-tasks-additional-properties-error-one-of-1-message"></span><span id="response-network-remote-tasks-error-message"></span><span id="response-network-remote-tasks-error-one-of-1-message"></span>        message:
          type: string
          description: The error message.
<span id="response-error-code"></span><span id="response-error-one-of-1-code"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-additional-properties-error-code"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-additional-properties-error-one-of-1-code"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-error-code"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-error-one-of-1-code"></span><span id="response-network-one-of-1-remote-tasks-additional-properties-error-code"></span><span id="response-network-one-of-1-remote-tasks-additional-properties-error-one-of-1-code"></span><span id="response-network-one-of-1-remote-tasks-error-code"></span><span id="response-network-one-of-1-remote-tasks-error-one-of-1-code"></span><span id="response-network-remote-tasks-additional-properties-error-code"></span><span id="response-network-remote-tasks-additional-properties-error-one-of-1-code"></span><span id="response-network-remote-tasks-error-code"></span><span id="response-network-remote-tasks-error-one-of-1-code"></span>        code:
          $ref: '#/components/schemas/Code'
          description: The error code.
<span id="response-error-one-of-1-type"></span><span id="response-error-type"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-additional-properties-error-one-of-1-type"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-additional-properties-error-type"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-error-one-of-1-type"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-error-type"></span><span id="response-network-one-of-1-remote-tasks-additional-properties-error-one-of-1-type"></span><span id="response-network-one-of-1-remote-tasks-additional-properties-error-type"></span><span id="response-network-one-of-1-remote-tasks-error-one-of-1-type"></span><span id="response-network-one-of-1-remote-tasks-error-type"></span><span id="response-network-remote-tasks-additional-properties-error-one-of-1-type"></span><span id="response-network-remote-tasks-additional-properties-error-type"></span><span id="response-network-remote-tasks-error-one-of-1-type"></span><span id="response-network-remote-tasks-error-type"></span>        type:
          $ref: '#/components/schemas/ErrorType'
          description: The error type.
<span id="response-error-link"></span><span id="response-error-one-of-1-link"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-additional-properties-error-link"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-additional-properties-error-one-of-1-link"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-error-link"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-error-one-of-1-link"></span><span id="response-network-one-of-1-remote-tasks-additional-properties-error-link"></span><span id="response-network-one-of-1-remote-tasks-additional-properties-error-one-of-1-link"></span><span id="response-network-one-of-1-remote-tasks-error-link"></span><span id="response-network-one-of-1-remote-tasks-error-one-of-1-link"></span><span id="response-network-remote-tasks-additional-properties-error-link"></span><span id="response-network-remote-tasks-additional-properties-error-one-of-1-link"></span><span id="response-network-remote-tasks-error-link"></span><span id="response-network-remote-tasks-error-one-of-1-link"></span>        link:
          type: string
          description: A link to the documentation about this specific error.
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
    DbTaskNetwork:
      oneOf:
<span id="response-network-one-of-1-one-of-0"></span>        - type: object
          description: Tasks that were duplicated from `origin`
          required:
            - origin
          properties:
<span id="response-network-one-of-0-origin"></span><span id="response-network-one-of-1-one-of-0-origin"></span><span id="response-network-one-of-1-origin"></span><span id="response-network-origin"></span>            origin:
              $ref: '#/components/schemas/Origin'
<span id="response-network-one-of-1-one-of-1"></span>        - type: object
          description: Tasks that were duplicated as `remote_tasks`
          required:
            - remote_tasks
          properties:
<span id="response-network-one-of-1-one-of-1-remote-tasks"></span><span id="response-network-one-of-1-remote-tasks"></span><span id="response-network-remote-tasks"></span>            remote_tasks:
              type: object
<span id="response-network-one-of-1-one-of-1-remote-tasks-additional-properties"></span><span id="response-network-one-of-1-remote-tasks-additional-properties"></span><span id="response-network-remote-tasks-additional-properties"></span>              additionalProperties:
                $ref: '#/components/schemas/RemoteTask'
              propertyNames:
                type: string
<span id="response-network-network-version"></span><span id="response-network-one-of-1-network-version"></span><span id="response-network-one-of-1-one-of-1-network-version"></span>            network_version:
              type: string
              format: uuid
<span id="response-network-one-of-1-one-of-2"></span><span id="response-network-one-of-2"></span>        - type: object
          description: Document import tasks sent in the context of `network_change`
          required:
            - import_from
            - network_change
          properties:
<span id="response-network-import-from"></span><span id="response-network-one-of-1-import-from"></span><span id="response-network-one-of-1-one-of-2-import-from"></span><span id="response-network-one-of-2-import-from"></span>            import_from:
              $ref: '#/components/schemas/ImportData'
<span id="response-network-network-change"></span><span id="response-network-one-of-1-network-change"></span><span id="response-network-one-of-1-one-of-2-network-change"></span><span id="response-network-one-of-2-network-change"></span>            network_change:
              $ref: '#/components/schemas/Origin'
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
    Origin:
      type: object
      description: |-
        Information about the origin of a task in a distributed Meilisearch
        deployment. This tracks where a task was originally created before being
        replicated to other nodes.
      required:
        - remoteName
        - taskUid
      properties:
<span id="response-network-network-change-remote-name"></span><span id="response-network-network-change-remotename"></span><span id="response-network-one-of-0-origin-remote-name"></span><span id="response-network-one-of-0-origin-remotename"></span><span id="response-network-one-of-1-network-change-remote-name"></span><span id="response-network-one-of-1-network-change-remotename"></span><span id="response-network-one-of-1-one-of-0-origin-remote-name"></span><span id="response-network-one-of-1-one-of-0-origin-remotename"></span><span id="response-network-one-of-1-one-of-2-network-change-remote-name"></span><span id="response-network-one-of-1-one-of-2-network-change-remotename"></span><span id="response-network-one-of-1-origin-remote-name"></span><span id="response-network-one-of-1-origin-remotename"></span><span id="response-network-one-of-2-network-change-remote-name"></span><span id="response-network-one-of-2-network-change-remotename"></span><span id="response-network-origin-remote-name"></span><span id="response-network-origin-remotename"></span>        remoteName:
          type: string
          description: &gt;-
            The name of the remote Meilisearch instance where this task
            originated.

            This corresponds to a remote defined in the network configuration.
<span id="response-network-network-change-task-uid"></span><span id="response-network-network-change-taskuid"></span><span id="response-network-one-of-0-origin-task-uid"></span><span id="response-network-one-of-0-origin-taskuid"></span><span id="response-network-one-of-1-network-change-task-uid"></span><span id="response-network-one-of-1-network-change-taskuid"></span><span id="response-network-one-of-1-one-of-0-origin-task-uid"></span><span id="response-network-one-of-1-one-of-0-origin-taskuid"></span><span id="response-network-one-of-1-one-of-2-network-change-task-uid"></span><span id="response-network-one-of-1-one-of-2-network-change-taskuid"></span><span id="response-network-one-of-1-origin-task-uid"></span><span id="response-network-one-of-1-origin-taskuid"></span><span id="response-network-one-of-2-network-change-task-uid"></span><span id="response-network-one-of-2-network-change-taskuid"></span><span id="response-network-origin-task-uid"></span><span id="response-network-origin-taskuid"></span>        taskUid:
          type: integer
          format: u-int32
          description: |-
            The unique task identifier on the originating remote. This allows
            tracking the same task across different nodes in the network.
          minimum: 0
<span id="response-network-network-change-network-version"></span><span id="response-network-network-change-networkversion"></span><span id="response-network-one-of-0-origin-network-version"></span><span id="response-network-one-of-0-origin-networkversion"></span><span id="response-network-one-of-1-network-change-network-version"></span><span id="response-network-one-of-1-network-change-networkversion"></span><span id="response-network-one-of-1-one-of-0-origin-network-version"></span><span id="response-network-one-of-1-one-of-0-origin-networkversion"></span><span id="response-network-one-of-1-one-of-2-network-change-network-version"></span><span id="response-network-one-of-1-one-of-2-network-change-networkversion"></span><span id="response-network-one-of-1-origin-network-version"></span><span id="response-network-one-of-1-origin-networkversion"></span><span id="response-network-one-of-2-network-change-network-version"></span><span id="response-network-one-of-2-network-change-networkversion"></span><span id="response-network-origin-network-version"></span><span id="response-network-origin-networkversion"></span>        networkVersion:
          type: string
          format: uuid
          description: &gt;-
            The version of the network topology when this task was created. Used
            to

            ensure consistent task routing during network topology changes.
    RemoteTask:
      type: object
      description: |-
        Represents a task that was replicated to a remote Meilisearch instance.
        Contains either the remote task UID on success, or an error if
        replication failed.
      properties:
<span id="response-network-one-of-1-one-of-1-remote-tasks-additional-properties-task-uid"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-additional-properties-taskuid"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-task-uid"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-taskuid"></span><span id="response-network-one-of-1-remote-tasks-additional-properties-task-uid"></span><span id="response-network-one-of-1-remote-tasks-additional-properties-taskuid"></span><span id="response-network-one-of-1-remote-tasks-task-uid"></span><span id="response-network-one-of-1-remote-tasks-taskuid"></span><span id="response-network-remote-tasks-additional-properties-task-uid"></span><span id="response-network-remote-tasks-additional-properties-taskuid"></span><span id="response-network-remote-tasks-task-uid"></span><span id="response-network-remote-tasks-taskuid"></span>        taskUid:
          type:
            - integer
            - 'null'
          format: u-int32
          description: &gt;-
            The unique task identifier assigned by the remote Meilisearch
            instance.

            Present when the task was successfully replicated to the remote.
          minimum: 0
<span id="response-network-one-of-1-one-of-1-remote-tasks-additional-properties-error"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-error"></span><span id="response-network-one-of-1-remote-tasks-additional-properties-error"></span><span id="response-network-one-of-1-remote-tasks-error"></span><span id="response-network-remote-tasks-additional-properties-error"></span><span id="response-network-remote-tasks-error"></span>        error:
          oneOf:
<span id="response-network-one-of-1-one-of-1-remote-tasks-additional-properties-error-one-of-0"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-error-one-of-0"></span><span id="response-network-one-of-1-remote-tasks-additional-properties-error-one-of-0"></span><span id="response-network-one-of-1-remote-tasks-error-one-of-0"></span><span id="response-network-remote-tasks-additional-properties-error-one-of-0"></span><span id="response-network-remote-tasks-error-one-of-0"></span>            - type: 'null'
<span id="response-network-one-of-1-one-of-1-remote-tasks-additional-properties-error-one-of-1"></span><span id="response-network-one-of-1-one-of-1-remote-tasks-error-one-of-1"></span><span id="response-network-one-of-1-remote-tasks-additional-properties-error-one-of-1"></span><span id="response-network-one-of-1-remote-tasks-error-one-of-1"></span><span id="response-network-remote-tasks-additional-properties-error-one-of-1"></span><span id="response-network-remote-tasks-error-one-of-1"></span>            - $ref: '#/components/schemas/ResponseError'
              description: &gt;-
                Error details if the task failed to replicate to this remote.
                Contains

                the error message, code, and type from the remote instance.
    ImportData:
      type: object
      description: Import data stored in a task
      required:
        - remoteName
        - documentCount
      properties:
<span id="response-network-import-from-remote-name"></span><span id="response-network-import-from-remotename"></span><span id="response-network-one-of-1-import-from-remote-name"></span><span id="response-network-one-of-1-import-from-remotename"></span><span id="response-network-one-of-1-one-of-2-import-from-remote-name"></span><span id="response-network-one-of-1-one-of-2-import-from-remotename"></span><span id="response-network-one-of-2-import-from-remote-name"></span><span id="response-network-one-of-2-import-from-remotename"></span>        remoteName:
          type: string
          description: Remote that this task is imported from
<span id="response-network-import-from-index-name"></span><span id="response-network-import-from-indexname"></span><span id="response-network-one-of-1-import-from-index-name"></span><span id="response-network-one-of-1-import-from-indexname"></span><span id="response-network-one-of-1-one-of-2-import-from-index-name"></span><span id="response-network-one-of-1-one-of-2-import-from-indexname"></span><span id="response-network-one-of-2-import-from-index-name"></span><span id="response-network-one-of-2-import-from-indexname"></span>        indexName:
          type:
            - string
            - 'null'
          description: Index relevant to this task
<span id="response-network-import-from-document-count"></span><span id="response-network-import-from-documentcount"></span><span id="response-network-one-of-1-import-from-document-count"></span><span id="response-network-one-of-1-import-from-documentcount"></span><span id="response-network-one-of-1-one-of-2-import-from-document-count"></span><span id="response-network-one-of-1-one-of-2-import-from-documentcount"></span><span id="response-network-one-of-2-import-from-document-count"></span><span id="response-network-one-of-2-import-from-documentcount"></span>        documentCount:
          type: integer
          format: u-int64
          description: Number of documents in this task
          minimum: 0
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