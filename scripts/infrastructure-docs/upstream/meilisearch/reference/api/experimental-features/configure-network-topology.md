# Configure network topology

> Add or remove remote nodes from the network. Changes apply to the current instance’s view of the cluster.



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
  /network:
    patch:
      tags:
        - Experimental features
      summary: Configure network topology
      description: &gt;-
        Add or remove remote nodes from the network. Changes apply to the
        current instance’s view of the cluster.
      operationId: patch_network
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/Network'
        required: true
      responses:
        '200':
          description: New network state is returned.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Network'
              example:
                self: ms-0
                remotes:
                  ms-0:
                    url: http://localhost:7700
                    searchApiKey: null
                    writeApiKey: null
                  ms-1:
                    url: http://localhost:7701
                    searchApiKey: foo
                    writeApiKey: bar
                  ms-2:
                    url: http://localhost:7702
                    searchApiKey: bar
                    writeApiKey: foo
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
            - network.update
            - '*'
      x-codeSamples:
        - lang: cURL
          source: |-
            curl \
              -X PATCH 'MEILISEARCH_URL/network' \
              -H 'Authorization: Bearer API_KEY' \
              -H 'Content-Type: application/json' \
              --data-binary '{
                "self": "ms-00",
                "remotes": {
                  "ms-00": {
                    "url": "http://INSTANCE_URL",
                    "searchApiKey": "INSTANCE_API_KEY"
                  },
                  "ms-01": {
                    "url": "http://ANOTHER_INSTANCE_URL",
                    "searchApiKey": "ANOTHER_INSTANCE_API_KEY"
                  }
                }
              }'
        - lang: JS
          source: |-
            client.initializeNetwork({
              self: 'https://instance1.meilisearch.com',
              remotes: {
                instance2: {
                  url: 'https://instance2.meilisearch.com',
                  searchApiKey: 'SEARCH_API_KEY',
                  writeApiKey: 'WRITE_API_KEY'
                }
              }
            })
        - lang: PHP
          source: |-
            $client-&gt;updateNetwork([
              'self' =&gt; 'ms-00',
              'leader' =&gt; 'ms-00',
              'remotes' =&gt; [
                'ms-00' =&gt; [
                  'url' =&gt; 'http://INSTANCE_URL',
                  'searchApiKey' =&gt; 'INSTANCE_API_KEY',
                  'writeApiKey' =&gt; 'INSTANCE_WRITE_API_KEY'
                ],
                'ms-01' =&gt; [
                  'url' =&gt; 'http://ANOTHER_INSTANCE_URL',
                  'searchApiKey' =&gt; 'ANOTHER_INSTANCE_API_KEY',
                  'writeApiKey' =&gt; 'ANOTHER_INSTANCE_WRITE_API_KEY'
                ]
              ]
            ]);
        - lang: Python
          source: |-
            client.add_or_update_networks({
                "remotes": {
                    "MEILISEARCH_URL": {
                        "searchApiKey": "masterKey"
                    }
                },
                "leader": None
            })
        - lang: Go
          source: |-
            client.UpdateNetwork(&amp;meilisearch.UpdateNetworkRequest{
              Self: meilisearch.String("ms-00"),
              Leader: meilisearch.String("ms-00"),
              Remotes: meilisearch.NewOpt(map[string]meilisearch.Opt[meilisearch.UpdateRemote]{
                "ms-00": meilisearch.NewOpt(meilisearch.UpdateRemote{
                  URL:          meilisearch.String("https://meilisearch.com"),
                  SearchAPIKey: meilisearch.String("ReadKey"),
                  WriteAPIKey:  meilisearch.String("WriteKey"),
                }),
              }),
              Shards: meilisearch.NewOpt(map[string]meilisearch.Opt[meilisearch.UpdateRemoteShard]{
                "ms-00": meilisearch.NewOpt(meilisearch.UpdateRemoteShard{
                  Remotes: meilisearch.NewOpt([]string{"ms-00"}),
                }),
              }),
            })
        - lang: Rust
          source: &gt;-
            let mut remotes = std::collections::HashMap::new();

            remotes.insert(String::from("ms-01"),
            Some(meilisearch_sdk::network::RemoteConfig {
              url: "https://ms-01.enterprise.meilisearch.com".to_string(),
              search_api_key: "SEARCH_API_KEY".to_string(),
              write_api_key: Some("WRITE_API_KEY".to_string()),
            }));

            // Remove ms-00 from the topology

            remotes.insert(String::from("ms-00"), None);


            let update = meilisearch_sdk::network::NetworkUpdate {
              leader: Some("ms-01".to_string()),
              remotes: Some(remotes),
              ..meilisearch_sdk::network::NetworkUpdate::default()
            };


            let task: TaskInfo = client
              .update_network_state(&amp;update)
              .await
              .unwrap();
components:
  schemas:
    Network:
      type: object
      description: Network topology configuration for distributed Meilisearch
      properties:
<span id="body-remotes"></span><span id="response-remotes"></span>        remotes:
          type:
            - object
            - 'null'
          description: &gt;-
            Map of remote instance names to their configurations


            - Pass `null` as a value for a remote to remove it from the
            configuration.

            - Removing a remote will also remove it from all shards.

            - Remotes that don't appear in this list will be unmodified by the
            network call.
<span id="body-remotes-additional-properties"></span><span id="response-remotes-additional-properties"></span>          additionalProperties:
            $ref: '#/components/schemas/Remote'
          propertyNames:
            type: string
          example:
            ms-00:
              url: http://localhost:7700
            ms-01:
              url: http://localhost:7701
<span id="body-shards"></span><span id="response-shards"></span>        shards:
          type:
            - object
            - 'null'
          description: &gt;-
            Map of shard names to their configurations.


            - Pass `null` as a value for a shard to remove it from the
            configuration.

            - Shards that don't appear in this list will be unmodified by the
            network call.
<span id="body-shards-additional-properties"></span><span id="response-shards-additional-properties"></span>          additionalProperties:
            $ref: '#/components/schemas/Shard'
          propertyNames:
            type: string
          example:
            shard-00:
              remotes:
                - ms-00
                - ms-01
<span id="body-previous-shards"></span><span id="body-previousshards"></span><span id="response-previous-shards"></span><span id="response-previousshards"></span>        previousShards:
          type:
            - object
            - 'null'
          description: &gt;-
            Previous shard configurations


            This field should not be passed by end-users. It is used in internal
            communications between Meilisearch instances
<span id="body-previous-shards-additional-properties"></span><span id="response-previous-shards-additional-properties"></span>          additionalProperties:
            $ref: '#/components/schemas/Shard'
          propertyNames:
            type: string
          example:
            shard-00:
              remotes:
                - ms-00
                - ms-01
<span id="body-self"></span><span id="response-self"></span>        self:
          type:
            - string
            - 'null'
          description: Name of this instance in the network
          example: ms-00
<span id="body-leader"></span><span id="response-leader"></span>        leader:
          type:
            - string
            - 'null'
          description: Name of the leader instance in the network
          example: ms-00
<span id="body-previous-remotes"></span><span id="body-previousremotes"></span><span id="response-previous-remotes"></span><span id="response-previousremotes"></span>        previousRemotes:
          type:
            - object
            - 'null'
          description: &gt;-
            Previous remote configurations


            This field should not be passed by end-users. It is used in internal
            communications between Meilisearch instances
<span id="body-previous-remotes-additional-properties"></span><span id="response-previous-remotes-additional-properties"></span>          additionalProperties:
            $ref: '#/components/schemas/Remote'
          propertyNames:
            type: string
          example:
            ms-00:
              url: http://localhost:7700
            ms-01:
              url: http://localhost:7701
      additionalProperties: false
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
    Remote:
      type: object
      description: Configuration for a remote Meilisearch instance
      properties:
<span id="body-previous-remotes-additional-properties-url"></span><span id="body-previous-remotes-url"></span><span id="body-remotes-additional-properties-url"></span><span id="body-remotes-url"></span><span id="response-previous-remotes-additional-properties-url"></span><span id="response-previous-remotes-url"></span><span id="response-remotes-additional-properties-url"></span><span id="response-remotes-url"></span>        url:
          type:
            - string
            - 'null'
          description: &gt;-
            URL of the remote instance


            - If the URL of the remote instance is resolving to a non-global IP,
            make sure that
              `--experimental-allowed-ip-networks` allows it. For details on how use this parameter,
              refer to [this documentation](https://www.meilisearch.com/docs/learn/self_hosted/configure_meilisearch_at_launch#allow-requests-to-private-networks).
          example: http://localhost:7700
<span id="body-previous-remotes-additional-properties-search-api-key"></span><span id="body-previous-remotes-additional-properties-searchapikey"></span><span id="body-previous-remotes-search-api-key"></span><span id="body-previous-remotes-searchapikey"></span><span id="body-remotes-additional-properties-search-api-key"></span><span id="body-remotes-additional-properties-searchapikey"></span><span id="body-remotes-search-api-key"></span><span id="body-remotes-searchapikey"></span><span id="response-previous-remotes-additional-properties-search-api-key"></span><span id="response-previous-remotes-additional-properties-searchapikey"></span><span id="response-previous-remotes-search-api-key"></span><span id="response-previous-remotes-searchapikey"></span><span id="response-remotes-additional-properties-search-api-key"></span><span id="response-remotes-additional-properties-searchapikey"></span><span id="response-remotes-search-api-key"></span><span id="response-remotes-searchapikey"></span>        searchApiKey:
          type:
            - string
            - 'null'
          description: API key for search operations on this remote
          example: XWnBI8QHUc-4IlqbKPLUDuhftNq19mQtjc6JvmivzJU
<span id="body-previous-remotes-additional-properties-write-api-key"></span><span id="body-previous-remotes-additional-properties-writeapikey"></span><span id="body-previous-remotes-write-api-key"></span><span id="body-previous-remotes-writeapikey"></span><span id="body-remotes-additional-properties-write-api-key"></span><span id="body-remotes-additional-properties-writeapikey"></span><span id="body-remotes-write-api-key"></span><span id="body-remotes-writeapikey"></span><span id="response-previous-remotes-additional-properties-write-api-key"></span><span id="response-previous-remotes-additional-properties-writeapikey"></span><span id="response-previous-remotes-write-api-key"></span><span id="response-previous-remotes-writeapikey"></span><span id="response-remotes-additional-properties-write-api-key"></span><span id="response-remotes-additional-properties-writeapikey"></span><span id="response-remotes-write-api-key"></span><span id="response-remotes-writeapikey"></span>        writeApiKey:
          type:
            - string
            - 'null'
          description: API key for write operations on this remote
          example: XWnBI8QHUc-4IlqbKPLUDuhftNq19mQtjc6JvmivzJU
      additionalProperties: false
    Shard:
      type: object
      description: &gt;-
        Configuration for a named shard of the network


        Documents are distributed among the various declared shards.

        Remotes own shards, meaning they own the documents that belong to these
        shards.
      properties:
<span id="body-previous-shards-additional-properties-remotes"></span><span id="body-previous-shards-remotes"></span><span id="body-shards-additional-properties-remotes"></span><span id="body-shards-remotes"></span><span id="response-previous-shards-additional-properties-remotes"></span><span id="response-previous-shards-remotes"></span><span id="response-shards-additional-properties-remotes"></span><span id="response-shards-remotes"></span>        remotes:
          type:
            - array
            - 'null'
<span id="body-previous-shards-additional-properties-remotes-items"></span><span id="body-previous-shards-remotes-items"></span><span id="body-shards-additional-properties-remotes-items"></span><span id="body-shards-remotes-items"></span><span id="response-previous-shards-additional-properties-remotes-items"></span><span id="response-previous-shards-remotes-items"></span><span id="response-shards-additional-properties-remotes-items"></span><span id="response-shards-remotes-items"></span>          items:
            type: string
          description: &gt;-
            List of remotes that own this shard.


            - The remotes must be part of the network's configuration

            - Setting this to a non-`null` value will replace all existing
            remotes for this shard.

            - `addRemotes` and `removeRemotes` are applied after `remotes` if
            multiple options are present.
          uniqueItems: true
<span id="body-previous-shards-add-remotes"></span><span id="body-previous-shards-additional-properties-add-remotes"></span><span id="body-previous-shards-additional-properties-addremotes"></span><span id="body-previous-shards-addremotes"></span><span id="body-shards-add-remotes"></span><span id="body-shards-additional-properties-add-remotes"></span><span id="body-shards-additional-properties-addremotes"></span><span id="body-shards-addremotes"></span><span id="response-previous-shards-add-remotes"></span><span id="response-previous-shards-additional-properties-add-remotes"></span><span id="response-previous-shards-additional-properties-addremotes"></span><span id="response-previous-shards-addremotes"></span><span id="response-shards-add-remotes"></span><span id="response-shards-additional-properties-add-remotes"></span><span id="response-shards-additional-properties-addremotes"></span><span id="response-shards-addremotes"></span>        addRemotes:
          type:
            - array
            - 'null'
<span id="body-previous-shards-add-remotes-items"></span><span id="body-previous-shards-additional-properties-add-remotes-items"></span><span id="body-shards-add-remotes-items"></span><span id="body-shards-additional-properties-add-remotes-items"></span><span id="response-previous-shards-add-remotes-items"></span><span id="response-previous-shards-additional-properties-add-remotes-items"></span><span id="response-shards-add-remotes-items"></span><span id="response-shards-additional-properties-add-remotes-items"></span>          items:
            type: string
          description: &gt;-
            Remotes to add to the list of owners of this shard.


            - The remotes must be part of the network's configuration

            - Setting this to non-`null` will append the listed remotes to the
            list of owners of this shard.

            - `remotes` is applied before `addRemotes`

            - `removeRemotes` is applied after `addRemotes`
          uniqueItems: true
<span id="body-previous-shards-additional-properties-remove-remotes"></span><span id="body-previous-shards-additional-properties-removeremotes"></span><span id="body-previous-shards-remove-remotes"></span><span id="body-previous-shards-removeremotes"></span><span id="body-shards-additional-properties-remove-remotes"></span><span id="body-shards-additional-properties-removeremotes"></span><span id="body-shards-remove-remotes"></span><span id="body-shards-removeremotes"></span><span id="response-previous-shards-additional-properties-remove-remotes"></span><span id="response-previous-shards-additional-properties-removeremotes"></span><span id="response-previous-shards-remove-remotes"></span><span id="response-previous-shards-removeremotes"></span><span id="response-shards-additional-properties-remove-remotes"></span><span id="response-shards-additional-properties-removeremotes"></span><span id="response-shards-remove-remotes"></span><span id="response-shards-removeremotes"></span>        removeRemotes:
          type:
            - array
            - 'null'
<span id="body-previous-shards-additional-properties-remove-remotes-items"></span><span id="body-previous-shards-remove-remotes-items"></span><span id="body-shards-additional-properties-remove-remotes-items"></span><span id="body-shards-remove-remotes-items"></span><span id="response-previous-shards-additional-properties-remove-remotes-items"></span><span id="response-previous-shards-remove-remotes-items"></span><span id="response-shards-additional-properties-remove-remotes-items"></span><span id="response-shards-remove-remotes-items"></span>          items:
            type: string
          description: &gt;-
            Remotes to remove from the list of owners of this shard.


            - The remotes may or may not be part of the network's configuration

            - Setting this to non-`null` will remove the listed remotes from the
            list of owners of this shard.

            - `remotes` and `addRemotes` are applied before `removeRemotes`

            - Remotes removed from the configuration are automatically removed
            from all shards and it is not necessary
              to explicitly pass them as `removeRemotes`.
          uniqueItems: true
      additionalProperties: false
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