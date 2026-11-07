# Get health

> The health check endpoint enables you to periodically test the health of your Meilisearch instance. Returns a simple status indicating that the server is available.

The engine will return `available` status with a `200` status code when the instance is healthy.
It will return `mustRestart` status with a `500` status code if the instance requires a restart.
This restart is required after a compaction of the task queue for example.



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
  /health:
    get:
      tags:
        - Health
      summary: Get health
      description: &gt;-
        The health check endpoint enables you to periodically test the health of
        your Meilisearch instance. Returns a simple status indicating that the
        server is available.


        The engine will return `available` status with a `200` status code when
        the instance is healthy.

        It will return `mustRestart` status with a `500` status code if the
        instance requires a restart.

        This restart is required after a compaction of the task queue for
        example.
      operationId: get_health
      responses:
        '200':
          description: Instance is healthy.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HealthResponse'
              example:
                status: available
      security: []
      x-codeSamples:
        - lang: cURL
          source: |-
            curl \
              -X GET 'MEILISEARCH_URL/health' \
              -H 'Authorization: Bearer API_KEY'
        - lang: JS
          source: client.health()
        - lang: PHP
          source: $client-&gt;health();
        - lang: Python
          source: client.health()
        - lang: Java
          source: client.health();
        - lang: Ruby
          source: client.health
        - lang: Go
          source: client.Health()
        - lang: C#
          source: await client.HealthAsync();
        - lang: Rust
          source: &gt;-
            // health() return an Err() if the server is not healthy, so this
            example would panic due to the unwrap

            client
              .health()
              .await
              .unwrap();
        - lang: Dart
          source: await client.health();
        - lang: Swift
          source: |-
            client.health { (result) in
                switch result {
                case .success:
                    print("Healthy!")
                case .failure(let error):
                    print(error)
                }
            }
components:
  schemas:
    HealthResponse:
      type: object
      required:
        - status
      properties:
<span id="response-status"></span>        status:
          $ref: '#/components/schemas/HealthStatus'
          description: The status of the instance.
    HealthStatus:
      type: string
      enum:
        - available
        - mustRestart

</code></pre>

This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.