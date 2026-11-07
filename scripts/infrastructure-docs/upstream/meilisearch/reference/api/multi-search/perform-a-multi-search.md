# Perform a multi-search

> Run multiple search queries in a single API request.

Each query can target a different index, so you can search across several indexes at once and get one combined response.

**Warning:** If Meilisearch encounters an error processing any query in the request, it immediately
stops and returns an error message for the first error encountered. Partial results are not returned.



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
  /multi-search:
    post:
      tags:
        - Multi-search
      summary: Perform a multi-search
      description: &gt;-
        Run multiple search queries in a single API request.


        Each query can target a different index, so you can search across
        several indexes at once and get one combined response.


        **Warning:** If Meilisearch encounters an error processing any query in
        the request, it immediately

        stops and returns an error message for the first error encountered.
        Partial results are not returned.
      operationId: multi_search_with_post
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/FederatedSearch'
        required: true
      responses:
        '200':
          description: Federated multi-search.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/FederatedSearchResult'
              example:
                hits:
                  - id: 42
                    title: Batman returns
                    overview: The overview of batman returns
                    _federation:
                      indexUid: movies
                      queriesPosition: 0
                  - comicsId: batman-killing-joke
                    description: This comic is really awesome
                    title: 'Batman: the killing joke'
                    _federation:
                      indexUid: comics
                      queriesPosition: 1
                processingTimeMs: 0
                limit: 20
                offset: 0
                estimatedTotalHits: 2
                semanticHitCount: 0
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
            - search
            - '*'
      x-codeSamples:
        - lang: cURL
          source: |-
            curl \
              -X POST 'MEILISEARCH_URL/multi-search' \
              -H 'Authorization: Bearer API_KEY' \
              -H 'Content-Type: application/json' \
              --data-binary '{
                "queries": [
                  {
                    "indexUid": "movies",
                    "q": "pooh",
                    "limit": 5
                  },
                  {
                    "indexUid": "movies",
                    "q": "nemo",
                    "limit": 5
                  },
                  {
                    "indexUid": "movie_ratings",
                    "q": "us"
                  }
                ]
              }'
        - lang: JS
          source: |-
            client.multiSearch({ queries: [
              {
                indexUid: 'movies',
                q: 'pooh',
                limit: 5,
              },
              {
                indexUid: 'movies',
                q: 'nemo',
                limit: 5,
              },
              {
                indexUid: 'movie_ratings',
                q: 'us',
              },
            ]})
        - lang: PHP
          source: |-
            $client-&gt;multiSearch([
                (new SearchQuery())
                    -&gt;setIndexUid('movies')
                    -&gt;setQuery('pooh')
                    -&gt;setLimit(5),
                (new SearchQuery())
                    -&gt;setIndexUid('movies')
                    -&gt;setQuery('nemo')
                    -&gt;setLimit(5),
                (new SearchQuery())
                    -&gt;setIndexUid('movie_ratings')
                    -&gt;setQuery('us')
              ]);
        - lang: Python
          source: |-
            client.multi_search(
              [
                {'indexUid': 'movies', 'q': 'pooh', 'limit': 5},
                {'indexUid': 'movies', 'q': 'nemo', 'limit': 5},
                {'indexUid': 'movie_ratings', 'q': 'us'}
              ]
            )
        - lang: Java
          source: &gt;-
            MultiSearchRequest multiSearchRequest = new MultiSearchRequest();

            multiIndexSearch.addQuery(new
            IndexSearchRequest("movies").setQuery("pooh").setLimit(5));

            multiIndexSearch.addQuery(new
            IndexSearchRequest("movies").setQuery("nemo").setLimit(5));

            multiIndexSearch.addQuery(new
            IndexSearchRequest("movie_ratings").setQuery("us"));


            client.multiSearch(multiSearchRequest);
        - lang: Ruby
          source: |-
            client.multi_search([
              { index_uid: 'books', q: 'prince' },
              { index_uid: 'movies', q: 'pooh', limit: 5 }
              { index_uid: 'movies', q: 'nemo', limit: 5 }
              { index_uid: 'movie_ratings', q: 'us' }
            ])
        - lang: Go
          source: |-
            client.MultiSearch(&amp;MultiSearchRequest{
              Queries: []SearchRequest{
                {
                  IndexUID: "movies",
                  Query:    "pooh",
                  Limit:    5,
                },
                {
                  IndexUID: "movies",
                  Query:    "nemo",
                  Limit:    5,
                },
                {
                  IndexUID: "movie_ratings",
                  Query:    "us",
                },
              },
            })
        - lang: C#
          source: |-
            await client.MultiSearchAsync(new MultiSearchQuery()
            {
                Queries = new System.Collections.Generic.List&lt;SearchQuery&gt;()
                {
                    new SearchQuery() {
                      IndexUid = "movies",
                      Q = "booh",
                      Limit = 5
                    },
                    new SearchQuery() {
                      IndexUid = "movies",
                      Q = "nemo",
                      Limit = 5
                    },
                    new SearchQuery() {
                      IndexUid = "movie_ratings",
                      Q = "us",
                    },
                }
            });
        - lang: Rust
          source: |-
            let movie = client.index("movie");
            let movie_ratings = client.index("movie_ratings");

            let search_query_1 = SearchQuery::new(&amp;movie)
                .with_query("pooh")
                .with_limit(5)
                .build();
            let search_query_2 = SearchQuery::new(&amp;movie)
                .with_query("nemo")
                .with_limit(5)
                .build();
            let search_query_3 = SearchQuery::new(&amp;movie_ratings)
                .with_query("us")
                .build();

            let response = client
                .multi_search()
                .with_search_query(search_query_1)
                .with_search_query(search_query_2)
                .with_search_query(search_query_3)
                .execute::&lt;Document&gt;()
                .await
                .unwrap();
        - lang: Dart
          source: |-
            await client.multiSearch(MultiSearchQuery(queries: [
              IndexSearchQuery(query: 'pooh', indexUid: 'movies', limit: 5),
              IndexSearchQuery(query: 'nemo', indexUid: 'movies', limit: 5),
              IndexSearchQuery(query: 'us', indexUid: 'movies_ratings'),
            ]));
components:
  schemas:
    FederatedSearch:
      type: object
      description: |-
        Request body for federated multi-search across multiple indexes. This
        allows you to execute multiple search queries in a single request and
        optionally combine their results into a unified response. Use this for
        cross-index search scenarios or to reduce network round-trips.
      required:
        - queries
      properties:
<span id="body-queries"></span>        queries:
          type: array
<span id="body-queries-items"></span>          items:
            $ref: '#/components/schemas/SearchQueryWithIndex'
          description: &gt;-
            An array of search queries to execute. Each query can target a

            different index and have its own parameters. When `federation` is

            `null`, results are returned separately for each query. When

            `federation` is set, results are merged.


            Each query object must include `indexUid` to specify which index to
            search.
<span id="body-federation"></span>        federation:
          oneOf:
<span id="body-federation-one-of-0"></span>            - type: 'null'
<span id="body-federation-one-of-1"></span>            - $ref: '#/components/schemas/Federation'
              description: &gt;-
                Configuration for combining results from multiple queries into a

                single response. When set, results are merged and ranked
                together.

                When `null`, each query's results are returned separately in an

                array.
      additionalProperties: false
    FederatedSearchResult:
      allOf:
<span id="response-all-of-0"></span>        - $ref: '#/components/schemas/HitsInfo'
          description: Pagination information
<span id="response-all-of-1"></span>        - type: object
          required:
            - hits
            - processingTimeMs
          properties:
<span id="response-all-of-1-hits"></span><span id="response-hits"></span>            hits:
              type: array
<span id="response-all-of-1-hits-items"></span><span id="response-hits-items"></span>              items:
                $ref: '#/components/schemas/SearchHit'
              description: Combined search results from all queries
<span id="response-all-of-1-query-vectors"></span><span id="response-all-of-1-queryvectors"></span><span id="response-query-vectors"></span><span id="response-queryvectors"></span>            queryVectors:
              type:
                - object
                - 'null'
              description: List of vectors per query, if `retrieveVectors` is set to `true`
<span id="response-all-of-1-query-vectors-additional-properties"></span><span id="response-query-vectors-additional-properties"></span>              additionalProperties:
                $ref: '#/components/schemas/Vec'
              propertyNames:
                type: integer
                minimum: 0
<span id="response-all-of-1-processing-time-ms"></span><span id="response-all-of-1-processingtimems"></span><span id="response-processing-time-ms"></span><span id="response-processingtimems"></span>            processingTimeMs:
              type: integer
              description: Total processing time in milliseconds
              minimum: 0
<span id="response-all-of-1-facet-distribution"></span><span id="response-all-of-1-facetdistribution"></span><span id="response-facet-distribution"></span><span id="response-facetdistribution"></span>            facetDistribution:
              type:
                - object
                - 'null'
<span id="response-all-of-1-facet-distribution-additional-properties"></span><span id="response-facet-distribution-additional-properties"></span>              additionalProperties:
                type: object
<span id="response-all-of-1-facet-distribution-additional-properties-additional-properties"></span><span id="response-facet-distribution-additional-properties-additional-properties"></span>                additionalProperties:
                  type: integer
                  format: u-int64
                  minimum: 0
                propertyNames:
                  type: string
              propertyNames:
                type: string
<span id="response-all-of-1-facet-stats"></span><span id="response-all-of-1-facetstats"></span><span id="response-facet-stats"></span><span id="response-facetstats"></span>            facetStats:
              type:
                - object
                - 'null'
              description: Merged facet statistics across all indexes
<span id="response-all-of-1-facet-stats-additional-properties"></span><span id="response-facet-stats-additional-properties"></span>              additionalProperties:
                $ref: '#/components/schemas/FacetStats'
              propertyNames:
                type: string
<span id="response-all-of-1-facets-by-index"></span><span id="response-all-of-1-facetsbyindex"></span><span id="response-facets-by-index"></span><span id="response-facetsbyindex"></span>            facetsByIndex:
              $ref: '#/components/schemas/FederatedFacets'
              description: Facets grouped by index
<span id="response-all-of-1-request-uid"></span><span id="response-all-of-1-requestuid"></span><span id="response-request-uid"></span><span id="response-requestuid"></span>            requestUid:
              type:
                - string
                - 'null'
              format: uuid
              description: Unique identifier for the request
<span id="response-all-of-1-metadata"></span><span id="response-metadata"></span>            metadata:
              type:
                - array
                - 'null'
<span id="response-all-of-1-metadata-items"></span><span id="response-metadata-items"></span>              items:
                $ref: '#/components/schemas/SearchMetadata'
              description: Metadata for each query
<span id="response-all-of-1-remote-errors"></span><span id="response-all-of-1-remoteerrors"></span><span id="response-remote-errors"></span><span id="response-remoteerrors"></span>            remoteErrors:
              type:
                - object
                - 'null'
              description: Errors from remote servers
<span id="response-all-of-1-remote-errors-additional-properties"></span><span id="response-remote-errors-additional-properties"></span>              additionalProperties:
                $ref: '#/components/schemas/ResponseError'
              propertyNames:
                type: string
<span id="response-all-of-1-semantic-hit-count"></span><span id="response-all-of-1-semantichitcount"></span><span id="response-semantic-hit-count"></span><span id="response-semantichitcount"></span>            semanticHitCount:
              type:
                - integer
                - 'null'
              format: u-int32
              minimum: 0
<span id="response-all-of-1-performance-details"></span><span id="response-all-of-1-performancedetails"></span><span id="response-performance-details"></span><span id="response-performancedetails"></span>            performanceDetails: {}
      description: Response from a federated multi-search query
    ResponseError:
      type: object
      required:
        - message
        - code
        - type
        - link
      properties:
<span id="response-all-of-1-remote-errors-additional-properties-message"></span><span id="response-all-of-1-remote-errors-message"></span><span id="response-remote-errors-additional-properties-message"></span><span id="response-remote-errors-message"></span>        message:
          type: string
          description: The error message.
<span id="response-all-of-1-remote-errors-additional-properties-code"></span><span id="response-all-of-1-remote-errors-code"></span><span id="response-remote-errors-additional-properties-code"></span><span id="response-remote-errors-code"></span>        code:
          $ref: '#/components/schemas/Code'
          description: The error code.
<span id="response-all-of-1-remote-errors-additional-properties-type"></span><span id="response-all-of-1-remote-errors-type"></span><span id="response-remote-errors-additional-properties-type"></span><span id="response-remote-errors-type"></span>        type:
          $ref: '#/components/schemas/ErrorType'
          description: The error type.
<span id="response-all-of-1-remote-errors-additional-properties-link"></span><span id="response-all-of-1-remote-errors-link"></span><span id="response-remote-errors-additional-properties-link"></span><span id="response-remote-errors-link"></span>        link:
          type: string
          description: A link to the documentation about this specific error.
    SearchQueryWithIndex:
      type: object
      description: A `SearchQuery` + an index UID and optional FederationOptions.
      required:
        - indexUid
      properties:
<span id="body-queries-index-uid"></span><span id="body-queries-indexuid"></span><span id="body-queries-items-index-uid"></span><span id="body-queries-items-indexuid"></span>        indexUid:
          $ref: '#/components/schemas/IndexUid'
          description: Index unique identifier
<span id="body-queries-items-q"></span><span id="body-queries-q"></span>        q:
          type:
            - string
            - 'null'
          description: Query string
<span id="body-queries-items-offset"></span><span id="body-queries-offset"></span>        offset:
          type:
            - integer
            - 'null'
          description: Number of documents to skip
          minimum: 0
<span id="body-queries-items-limit"></span><span id="body-queries-limit"></span>        limit:
          type:
            - integer
            - 'null'
          description: Maximum number of documents returned
          minimum: 0
<span id="body-queries-items-page"></span><span id="body-queries-page"></span>        page:
          type:
            - integer
            - 'null'
          description: Request a specific page of results
          minimum: 0
<span id="body-queries-hits-per-page"></span><span id="body-queries-hitsperpage"></span><span id="body-queries-items-hits-per-page"></span><span id="body-queries-items-hitsperpage"></span>        hitsPerPage:
          type:
            - integer
            - 'null'
          description: Maximum number of documents returned for a page
          minimum: 0
<span id="body-queries-attributes-to-retrieve"></span><span id="body-queries-attributestoretrieve"></span><span id="body-queries-items-attributes-to-retrieve"></span><span id="body-queries-items-attributestoretrieve"></span>        attributesToRetrieve:
          type:
            - array
            - 'null'
<span id="body-queries-attributes-to-retrieve-items"></span><span id="body-queries-items-attributes-to-retrieve-items"></span>          items:
            type: string
          description: Attributes to display in the returned documents
          uniqueItems: true
<span id="body-queries-attributes-to-crop"></span><span id="body-queries-attributestocrop"></span><span id="body-queries-items-attributes-to-crop"></span><span id="body-queries-items-attributestocrop"></span>        attributesToCrop:
          type:
            - array
            - 'null'
<span id="body-queries-attributes-to-crop-items"></span><span id="body-queries-items-attributes-to-crop-items"></span>          items:
            type: string
          description: Attributes whose values have to be cropped
<span id="body-queries-crop-length"></span><span id="body-queries-croplength"></span><span id="body-queries-items-crop-length"></span><span id="body-queries-items-croplength"></span>        cropLength:
          type: integer
          description: Maximum length of cropped value in words
          minimum: 0
<span id="body-queries-crop-marker"></span><span id="body-queries-cropmarker"></span><span id="body-queries-items-crop-marker"></span><span id="body-queries-items-cropmarker"></span>        cropMarker:
          type: string
          description: String marking crop boundaries
<span id="body-queries-attributes-to-highlight"></span><span id="body-queries-attributestohighlight"></span><span id="body-queries-items-attributes-to-highlight"></span><span id="body-queries-items-attributestohighlight"></span>        attributesToHighlight:
          type:
            - array
            - 'null'
<span id="body-queries-attributes-to-highlight-items"></span><span id="body-queries-items-attributes-to-highlight-items"></span>          items:
            type: string
          description: Highlight matching terms contained in an attribute
          uniqueItems: true
<span id="body-queries-highlight-pre-tag"></span><span id="body-queries-highlightpretag"></span><span id="body-queries-items-highlight-pre-tag"></span><span id="body-queries-items-highlightpretag"></span>        highlightPreTag:
          type: string
          description: String inserted at the start of a highlighted term
<span id="body-queries-highlight-post-tag"></span><span id="body-queries-highlightposttag"></span><span id="body-queries-items-highlight-post-tag"></span><span id="body-queries-items-highlightposttag"></span>        highlightPostTag:
          type: string
          description: String inserted at the end of a highlighted term
<span id="body-queries-items-show-matches-position"></span><span id="body-queries-items-showmatchesposition"></span><span id="body-queries-show-matches-position"></span><span id="body-queries-showmatchesposition"></span>        showMatchesPosition:
          type: boolean
          description: Return matching terms location
<span id="body-queries-filter"></span><span id="body-queries-items-filter"></span>        filter:
          description: Filter queries by an attribute's value
<span id="body-queries-items-sort"></span><span id="body-queries-sort"></span>        sort:
          type:
            - array
            - 'null'
<span id="body-queries-items-sort-items"></span><span id="body-queries-sort-items"></span>          items:
            type: string
          description: Sort search results by an attribute's value
<span id="body-queries-distinct"></span><span id="body-queries-items-distinct"></span>        distinct:
          type:
            - string
            - 'null'
          description: |-
            Restrict search to documents with unique values of specified
            attribute
<span id="body-queries-facets"></span><span id="body-queries-items-facets"></span>        facets:
          oneOf:
<span id="body-queries-facets-one-of-0"></span><span id="body-queries-items-facets-one-of-0"></span>            - type: 'null'
<span id="body-queries-facets-one-of-1"></span><span id="body-queries-items-facets-one-of-1"></span>            - $ref: '#/components/schemas/AttributePatterns'
              description: Display the count of matches per facet
<span id="body-queries-items-matching-strategy"></span><span id="body-queries-items-matchingstrategy"></span><span id="body-queries-matching-strategy"></span><span id="body-queries-matchingstrategy"></span>        matchingStrategy:
          oneOf:
<span id="body-queries-items-matching-strategy-one-of-0"></span><span id="body-queries-matching-strategy-one-of-0"></span>            - $ref: '#/components/schemas/MatchingStrategy'
              description: Strategy used to match query terms within documents
<span id="body-queries-attributes-to-search-on"></span><span id="body-queries-attributestosearchon"></span><span id="body-queries-items-attributes-to-search-on"></span><span id="body-queries-items-attributestosearchon"></span>        attributesToSearchOn:
          type:
            - array
            - 'null'
<span id="body-queries-attributes-to-search-on-items"></span><span id="body-queries-items-attributes-to-search-on-items"></span>          items:
            type: string
          description: Restrict search to the specified attributes
<span id="body-queries-items-ranking-score-threshold"></span><span id="body-queries-items-rankingscorethreshold"></span><span id="body-queries-ranking-score-threshold"></span><span id="body-queries-rankingscorethreshold"></span>        rankingScoreThreshold:
          type:
            - number
            - 'null'
          format: double
          description: &gt;-
            Stop the search at the first document whose [ranking
            score](https://www.meilisearch.com/docs/learn/relevancy/ranking_score)
            is below this value (between 0.0 and 1.0).

            All the documents after this document are excluded from the hits.


            Excluded hits do not count toward `estimatedTotalHits`, `totalHits`,
            or facet distribution.


            When used together with `page` and `hitsPerPage`, this parameter may
            reduce performance because Meilisearch must score all matching
            documents.
<span id="body-queries-items-locales"></span><span id="body-queries-locales"></span>        locales:
          type:
            - array
            - 'null'
<span id="body-queries-items-locales-items"></span><span id="body-queries-locales-items"></span>          items:
            $ref: '#/components/schemas/Locale'
          description: Languages to use for query tokenization
<span id="body-queries-hybrid"></span><span id="body-queries-items-hybrid"></span>        hybrid:
          oneOf:
<span id="body-queries-hybrid-one-of-0"></span><span id="body-queries-items-hybrid-one-of-0"></span>            - type: 'null'
<span id="body-queries-hybrid-one-of-1"></span><span id="body-queries-items-hybrid-one-of-1"></span>            - $ref: '#/components/schemas/HybridQuery'
              description: &gt;-
                Hybrid search configuration combining keyword and semantic
                search.

                Set `semanticRatio` to balance between keyword matching (0.0)
                and

                semantic similarity (1.0). Requires an embedder to be
                configured.
<span id="body-queries-items-vector"></span><span id="body-queries-vector"></span>        vector:
          type:
            - array
            - 'null'
<span id="body-queries-items-vector-items"></span><span id="body-queries-vector-items"></span>          items:
            type: number
            format: float
          description: Search using a custom query vector
<span id="body-queries-items-retrieve-vectors"></span><span id="body-queries-items-retrievevectors"></span><span id="body-queries-retrieve-vectors"></span><span id="body-queries-retrievevectors"></span>        retrieveVectors:
          type: boolean
          description: Return document and query vector data
<span id="body-queries-items-media"></span><span id="body-queries-media"></span>        media:
          description: Perform AI-powered search queries with multimodal content
<span id="body-queries-items-personalize"></span><span id="body-queries-personalize"></span>        personalize:
          oneOf:
<span id="body-queries-items-personalize-one-of-0"></span><span id="body-queries-personalize-one-of-0"></span>            - type: 'null'
<span id="body-queries-items-personalize-one-of-1"></span><span id="body-queries-personalize-one-of-1"></span>            - $ref: '#/components/schemas/Personalize'
              description: Personalize search results
<span id="body-queries-items-use-network"></span><span id="body-queries-items-usenetwork"></span><span id="body-queries-use-network"></span><span id="body-queries-usenetwork"></span>        useNetwork:
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
<span id="body-queries-items-show-ranking-score"></span><span id="body-queries-items-showrankingscore"></span><span id="body-queries-show-ranking-score"></span><span id="body-queries-showrankingscore"></span>        showRankingScore:
          type: boolean
          description: Display the global ranking score of a document
<span id="body-queries-items-show-ranking-score-details"></span><span id="body-queries-items-showrankingscoredetails"></span><span id="body-queries-show-ranking-score-details"></span><span id="body-queries-showrankingscoredetails"></span>        showRankingScoreDetails:
          type: boolean
          description: Adds a detailed global ranking score field
<span id="body-queries-items-show-performance-details"></span><span id="body-queries-items-showperformancedetails"></span><span id="body-queries-show-performance-details"></span><span id="body-queries-showperformancedetails"></span>        showPerformanceDetails:
          type:
            - boolean
            - 'null'
          description: Adds a detailed performance details field
<span id="body-queries-federation-options"></span><span id="body-queries-federationoptions"></span><span id="body-queries-items-federation-options"></span><span id="body-queries-items-federationoptions"></span>        federationOptions:
          oneOf:
<span id="body-queries-federation-options-one-of-0"></span><span id="body-queries-items-federation-options-one-of-0"></span>            - type: 'null'
<span id="body-queries-federation-options-one-of-1"></span><span id="body-queries-items-federation-options-one-of-1"></span>            - $ref: '#/components/schemas/FederationOptions'
              description: Federation options for multi-index search
<span id="body-queries-inline-rule"></span><span id="body-queries-inlinerule"></span><span id="body-queries-items-inline-rule"></span><span id="body-queries-items-inlinerule"></span>        inlineRule:
          oneOf:
<span id="body-queries-inline-rule-one-of-0"></span><span id="body-queries-items-inline-rule-one-of-0"></span>            - type: 'null'
<span id="body-queries-inline-rule-one-of-1"></span><span id="body-queries-items-inline-rule-one-of-1"></span>            - $ref: '#/components/schemas/InlineDynamicSearchRule'
              description: &gt;-
                An inline [search rule](/capabilities/search_rules/overview) to
                apply to this query.


                - Requires the `dynamicSearchRule` experimental feature

                - Inline rules require the Enterprise Edition of Meilisearch.
      additionalProperties: false
    Federation:
      type: object
      description: Configuration for federated multi-search
      properties:
<span id="body-federation-limit"></span><span id="body-federation-one-of-1-limit"></span>        limit:
          type: integer
          description: Maximum number of results to return across all queries
          minimum: 0
<span id="body-federation-offset"></span><span id="body-federation-one-of-1-offset"></span>        offset:
          type: integer
          description: Number of results to skip
          minimum: 0
<span id="body-federation-one-of-1-page"></span><span id="body-federation-page"></span>        page:
          type:
            - integer
            - 'null'
          minimum: 0
<span id="body-federation-hits-per-page"></span><span id="body-federation-hitsperpage"></span><span id="body-federation-one-of-1-hits-per-page"></span><span id="body-federation-one-of-1-hitsperpage"></span>        hitsPerPage:
          type:
            - integer
            - 'null'
          minimum: 0
<span id="body-federation-facets-by-index"></span><span id="body-federation-facetsbyindex"></span><span id="body-federation-one-of-1-facets-by-index"></span><span id="body-federation-one-of-1-facetsbyindex"></span>        facetsByIndex:
          type: object
          description: Facets to retrieve per index
<span id="body-federation-facets-by-index-additional-properties"></span><span id="body-federation-one-of-1-facets-by-index-additional-properties"></span>          additionalProperties:
            $ref: '#/components/schemas/AttributePatterns'
          propertyNames:
            type: string
            example: movies
<span id="body-federation-merge-facets"></span><span id="body-federation-mergefacets"></span><span id="body-federation-one-of-1-merge-facets"></span><span id="body-federation-one-of-1-mergefacets"></span>        mergeFacets:
          oneOf:
<span id="body-federation-merge-facets-one-of-0"></span><span id="body-federation-one-of-1-merge-facets-one-of-0"></span>            - type: 'null'
<span id="body-federation-merge-facets-one-of-1"></span><span id="body-federation-one-of-1-merge-facets-one-of-1"></span>            - $ref: '#/components/schemas/MergeFacets'
              description: Options for merging facets from multiple indexes
<span id="body-federation-distinct"></span><span id="body-federation-one-of-1-distinct"></span>        distinct:
          type:
            - string
            - 'null'
          description: &gt;-
            Restrict search to documents with unique values of the specified
            attribute across all queries.


            1. This applies across all queries in the request, regardless of
            index or remote of origin.

            2. This overrides the index-level distinct attribute.

            3. This requires that the passed field can be set as distinct in all
            the participating indexes.
<span id="body-federation-one-of-1-show-performance-details"></span><span id="body-federation-one-of-1-showperformancedetails"></span><span id="body-federation-show-performance-details"></span><span id="body-federation-showperformancedetails"></span>        showPerformanceDetails:
          type: boolean
          description: Whether to include performance details in the response
<span id="body-federation-one-of-1-personalize"></span><span id="body-federation-personalize"></span>        personalize:
          oneOf:
<span id="body-federation-one-of-1-personalize-one-of-0"></span><span id="body-federation-personalize-one-of-0"></span>            - type: 'null'
<span id="body-federation-one-of-1-personalize-one-of-1"></span><span id="body-federation-personalize-one-of-1"></span>            - $ref: '#/components/schemas/Personalize'
              description: Personalize search results
      additionalProperties: false
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
    Vec:
      type: array
<span id="response-all-of-1-query-vectors-additional-properties-items"></span><span id="response-all-of-1-query-vectors-items"></span><span id="response-query-vectors-additional-properties-items"></span><span id="response-query-vectors-items"></span>      items:
        type: number
        format: float
    FacetStats:
      type: object
      description: &gt;-
        Minimum and maximum numeric values for a facet.


        Present when the facet attribute has numeric values among matching
        documents.
      required:
        - min
        - max
      properties:
<span id="response-all-of-1-facet-stats-additional-properties-min"></span><span id="response-all-of-1-facet-stats-min"></span><span id="response-all-of-1-facets-by-index-additional-properties-stats-additional-properties-min"></span><span id="response-all-of-1-facets-by-index-additional-properties-stats-min"></span><span id="response-all-of-1-facets-by-index-stats-additional-properties-min"></span><span id="response-all-of-1-facets-by-index-stats-min"></span><span id="response-facet-stats-additional-properties-min"></span><span id="response-facet-stats-min"></span><span id="response-facets-by-index-additional-properties-stats-additional-properties-min"></span><span id="response-facets-by-index-additional-properties-stats-min"></span><span id="response-facets-by-index-stats-additional-properties-min"></span><span id="response-facets-by-index-stats-min"></span>        min:
          type: number
          format: double
          description: Lowest numeric value for this facet among matching documents.
<span id="response-all-of-1-facet-stats-additional-properties-max"></span><span id="response-all-of-1-facet-stats-max"></span><span id="response-all-of-1-facets-by-index-additional-properties-stats-additional-properties-max"></span><span id="response-all-of-1-facets-by-index-additional-properties-stats-max"></span><span id="response-all-of-1-facets-by-index-stats-additional-properties-max"></span><span id="response-all-of-1-facets-by-index-stats-max"></span><span id="response-facet-stats-additional-properties-max"></span><span id="response-facet-stats-max"></span><span id="response-facets-by-index-additional-properties-stats-additional-properties-max"></span><span id="response-facets-by-index-additional-properties-stats-max"></span><span id="response-facets-by-index-stats-additional-properties-max"></span><span id="response-facets-by-index-stats-max"></span>        max:
          type: number
          format: double
          description: Highest numeric value for this facet among matching documents.
    FederatedFacets:
      type: object
<span id="response-all-of-1-facets-by-index-additional-properties"></span><span id="response-facets-by-index-additional-properties"></span>      additionalProperties:
        $ref: '#/components/schemas/ComputedFacets'
      propertyNames:
        type: string
    SearchMetadata:
      type: object
      description: Metadata about a search query (included when requested via header).
      required:
        - queryUid
        - indexUid
      properties:
<span id="response-all-of-1-metadata-items-query"></span><span id="response-all-of-1-metadata-query"></span><span id="response-metadata-items-query"></span><span id="response-metadata-query"></span>        query:
          type:
            - string
            - 'null'
          description: &gt;-
            Query string set for the query, if not already present in the
            response.


            Only set if:


            1. the metadata corresponds to a query in a federated search
            request, **and**

            2. that federated search request query is not a placeholder search
            (empty or missing `q`)


            Never set for regular search and non-federated multi-search because

            the query(ies) are already present in the response.
<span id="response-all-of-1-metadata-items-query-uid"></span><span id="response-all-of-1-metadata-items-queryuid"></span><span id="response-all-of-1-metadata-query-uid"></span><span id="response-all-of-1-metadata-queryuid"></span><span id="response-metadata-items-query-uid"></span><span id="response-metadata-items-queryuid"></span><span id="response-metadata-query-uid"></span><span id="response-metadata-queryuid"></span>        queryUid:
          type: string
          format: uuid
          description: Unique identifier for the query.
<span id="response-all-of-1-metadata-index-uid"></span><span id="response-all-of-1-metadata-indexuid"></span><span id="response-all-of-1-metadata-items-index-uid"></span><span id="response-all-of-1-metadata-items-indexuid"></span><span id="response-metadata-index-uid"></span><span id="response-metadata-indexuid"></span><span id="response-metadata-items-index-uid"></span><span id="response-metadata-items-indexuid"></span>        indexUid:
          type: string
          description: UID of the index that was searched.
<span id="response-all-of-1-metadata-items-primary-key"></span><span id="response-all-of-1-metadata-items-primarykey"></span><span id="response-all-of-1-metadata-primary-key"></span><span id="response-all-of-1-metadata-primarykey"></span><span id="response-metadata-items-primary-key"></span><span id="response-metadata-items-primarykey"></span><span id="response-metadata-primary-key"></span><span id="response-metadata-primarykey"></span>        primaryKey:
          type:
            - string
            - 'null'
          description: &gt;-
            [Primary
            key](https://www.meilisearch.com/docs/learn/getting_started/primary_key)
            of the index.
<span id="response-all-of-1-metadata-items-remote"></span><span id="response-all-of-1-metadata-remote"></span><span id="response-metadata-items-remote"></span><span id="response-metadata-remote"></span>        remote:
          type:
            - string
            - 'null'
          description: Remote that processed the query (federated search only).
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes"></span><span id="response-all-of-1-metadata-inlineruleconditionoutcomes"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes"></span><span id="response-all-of-1-metadata-items-inlineruleconditionoutcomes"></span><span id="response-metadata-inline-rule-condition-outcomes"></span><span id="response-metadata-inlineruleconditionoutcomes"></span><span id="response-metadata-items-inline-rule-condition-outcomes"></span><span id="response-metadata-items-inlineruleconditionoutcomes"></span>        inlineRuleConditionOutcomes:
          oneOf:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-0"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-0"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-0"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-0"></span>            - type: 'null'
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1"></span>            - $ref: '#/components/schemas/ConditionOutcomes'
              description: &gt;-
                Outcome of checking the conditions of the dynamic search rule
                passed in `inlineRule`.


                Only present in response if `inlineRule` was set in query.
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
    IndexUid:
      type: string
      example: movies
    AttributePatterns:
      type: array
<span id="body-federation-facets-by-index-additional-properties-items"></span><span id="body-federation-facets-by-index-items"></span><span id="body-federation-one-of-1-facets-by-index-additional-properties-items"></span><span id="body-federation-one-of-1-facets-by-index-items"></span><span id="body-queries-facets-items"></span><span id="body-queries-facets-one-of-1-items"></span><span id="body-queries-items-facets-items"></span><span id="body-queries-items-facets-one-of-1-items"></span>      items:
        type: string
      description: &gt;-
        A collection of patterns used to match attribute names. Patterns can
        include wildcards (`*`) for flexible matching. For example, `title`
        matches exactly, `overview_*` matches any attribute starting with
        `overview_`, and `*_date` matches any attribute ending with `_date`.
      examples:
        - '["*", "title", "description_*", "*_date", "*content*"]'
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
    HybridQuery:
      type: object
      description: 'Hybrid search: balance between keyword and semantic search.'
      required:
        - embedder
      properties:
<span id="body-queries-hybrid-one-of-1-semantic-ratio"></span><span id="body-queries-hybrid-one-of-1-semanticratio"></span><span id="body-queries-hybrid-semantic-ratio"></span><span id="body-queries-hybrid-semanticratio"></span><span id="body-queries-items-hybrid-one-of-1-semantic-ratio"></span><span id="body-queries-items-hybrid-one-of-1-semanticratio"></span><span id="body-queries-items-hybrid-semantic-ratio"></span><span id="body-queries-items-hybrid-semanticratio"></span>        semanticRatio:
          type: number
          format: float
          description: Balance between keyword (0.0) and semantic (1.0) search.
          example: 0.5
<span id="body-queries-hybrid-embedder"></span><span id="body-queries-hybrid-one-of-1-embedder"></span><span id="body-queries-items-hybrid-embedder"></span><span id="body-queries-items-hybrid-one-of-1-embedder"></span>        embedder:
          type: string
          description: &gt;-
            Name of the embedder from the index embedders setting (`embedder` in
            JSON). Used to vectorize the query.
          example: default
    Personalize:
      type: object
      description: &gt;-
        Configuration for [personalized
        search](https://www.meilisearch.com/docs/learn/personalization/making_personalized_search_queries)
        results.


        When enabled, results are tailored to the user profile described in
        `userContext`.
      required:
        - userContext
      properties:
<span id="body-federation-one-of-1-personalize-one-of-1-user-context"></span><span id="body-federation-one-of-1-personalize-one-of-1-usercontext"></span><span id="body-federation-one-of-1-personalize-user-context"></span><span id="body-federation-one-of-1-personalize-usercontext"></span><span id="body-federation-personalize-one-of-1-user-context"></span><span id="body-federation-personalize-one-of-1-usercontext"></span><span id="body-federation-personalize-user-context"></span><span id="body-federation-personalize-usercontext"></span><span id="body-queries-items-personalize-one-of-1-user-context"></span><span id="body-queries-items-personalize-one-of-1-usercontext"></span><span id="body-queries-items-personalize-user-context"></span><span id="body-queries-items-personalize-usercontext"></span><span id="body-queries-personalize-one-of-1-user-context"></span><span id="body-queries-personalize-one-of-1-usercontext"></span><span id="body-queries-personalize-user-context"></span><span id="body-queries-personalize-usercontext"></span>        userContext:
          type: string
          description: |-
            String describing the user (e.g. preferences, behavior).

            Used to return different results for different profiles.
      additionalProperties: false
    FederationOptions:
      type: object
      description: Options for federated multi-search queries
      properties:
<span id="body-queries-federation-options-one-of-1-weight"></span><span id="body-queries-federation-options-weight"></span><span id="body-queries-items-federation-options-one-of-1-weight"></span><span id="body-queries-items-federation-options-weight"></span>        weight:
          type: number
          format: double
          description: &gt;-
            A multiplicative factor applied to the ranking score for hits from
            this query.


            Values less than 1.0 make hits from this query less likely to appear
            in final results; values greater than 1.0 make them more likely.


            Must be a positive number. Default: `1.0`.
<span id="body-queries-federation-options-one-of-1-remote"></span><span id="body-queries-federation-options-remote"></span><span id="body-queries-items-federation-options-one-of-1-remote"></span><span id="body-queries-items-federation-options-remote"></span>        remote:
          type:
            - string
            - 'null'
          description: Remote server to send this query to
<span id="body-queries-federation-options-one-of-1-query-position"></span><span id="body-queries-federation-options-one-of-1-queryposition"></span><span id="body-queries-federation-options-query-position"></span><span id="body-queries-federation-options-queryposition"></span><span id="body-queries-items-federation-options-one-of-1-query-position"></span><span id="body-queries-items-federation-options-one-of-1-queryposition"></span><span id="body-queries-items-federation-options-query-position"></span><span id="body-queries-items-federation-options-queryposition"></span>        queryPosition:
          type:
            - integer
            - 'null'
          description: Position of this query in the list of queries
          minimum: 0
      additionalProperties: false
    InlineDynamicSearchRule:
      type: object
      required:
        - uid
      properties:
<span id="body-queries-inline-rule-one-of-1-uid"></span><span id="body-queries-inline-rule-uid"></span><span id="body-queries-items-inline-rule-one-of-1-uid"></span><span id="body-queries-items-inline-rule-uid"></span>        uid:
          $ref: '#/components/schemas/IndexUid'
<span id="body-queries-inline-rule-one-of-1-precedence"></span><span id="body-queries-inline-rule-precedence"></span><span id="body-queries-items-inline-rule-one-of-1-precedence"></span><span id="body-queries-items-inline-rule-precedence"></span>        precedence:
          type:
            - integer
            - 'null'
          format: u-int64
          minimum: 0
<span id="body-queries-inline-rule-active"></span><span id="body-queries-inline-rule-one-of-1-active"></span><span id="body-queries-items-inline-rule-active"></span><span id="body-queries-items-inline-rule-one-of-1-active"></span>        active:
          type: boolean
<span id="body-queries-inline-rule-conditions"></span><span id="body-queries-inline-rule-one-of-1-conditions"></span><span id="body-queries-items-inline-rule-conditions"></span><span id="body-queries-items-inline-rule-one-of-1-conditions"></span>        conditions:
          oneOf:
<span id="body-queries-inline-rule-conditions-one-of-0"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0"></span><span id="body-queries-items-inline-rule-conditions-one-of-0"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0"></span>            - $ref: '#/components/schemas/Conditions'
<span id="body-queries-inline-rule-actions"></span><span id="body-queries-inline-rule-one-of-1-actions"></span><span id="body-queries-items-inline-rule-actions"></span><span id="body-queries-items-inline-rule-one-of-1-actions"></span>        actions:
          oneOf:
<span id="body-queries-inline-rule-actions-one-of-0"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0"></span><span id="body-queries-items-inline-rule-actions-one-of-0"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0"></span>            - $ref: '#/components/schemas/RuleActions'
      additionalProperties: false
    MergeFacets:
      type: object
      description: |-
        Options for merging facets from multiple indexes in federated search.
        When multiple indexes are queried, this controls how their facet values
        are combined into a single facet distribution.
      properties:
<span id="body-federation-merge-facets-max-values-per-facet"></span><span id="body-federation-merge-facets-maxvaluesperfacet"></span><span id="body-federation-merge-facets-one-of-1-max-values-per-facet"></span><span id="body-federation-merge-facets-one-of-1-maxvaluesperfacet"></span><span id="body-federation-one-of-1-merge-facets-max-values-per-facet"></span><span id="body-federation-one-of-1-merge-facets-maxvaluesperfacet"></span><span id="body-federation-one-of-1-merge-facets-one-of-1-max-values-per-facet"></span><span id="body-federation-one-of-1-merge-facets-one-of-1-maxvaluesperfacet"></span>        maxValuesPerFacet:
          type:
            - integer
            - 'null'
          description: |-
            The maximum number of facet values to return for each facet after
            merging. Values from all indexes are combined and sorted before
            truncation. If not specified, uses the default limit from the index
            settings.
          minimum: 0
      additionalProperties: false
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
    ComputedFacets:
      type: object
      description: Computed facet data from a search
      required:
        - distribution
        - stats
      properties:
<span id="response-all-of-1-facets-by-index-additional-properties-distribution"></span><span id="response-all-of-1-facets-by-index-distribution"></span><span id="response-facets-by-index-additional-properties-distribution"></span><span id="response-facets-by-index-distribution"></span>        distribution:
          type: object
          description: Count of documents for each facet value
<span id="response-all-of-1-facets-by-index-additional-properties-distribution-additional-properties"></span><span id="response-all-of-1-facets-by-index-distribution-additional-properties"></span><span id="response-facets-by-index-additional-properties-distribution-additional-properties"></span><span id="response-facets-by-index-distribution-additional-properties"></span>          additionalProperties:
            type: object
<span id="response-all-of-1-facets-by-index-additional-properties-distribution-additional-properties-additional-properties"></span><span id="response-all-of-1-facets-by-index-distribution-additional-properties-additional-properties"></span><span id="response-facets-by-index-additional-properties-distribution-additional-properties-additional-properties"></span><span id="response-facets-by-index-distribution-additional-properties-additional-properties"></span>            additionalProperties:
              type: integer
              format: u-int64
              minimum: 0
            propertyNames:
              type: string
          propertyNames:
            type: string
<span id="response-all-of-1-facets-by-index-additional-properties-stats"></span><span id="response-all-of-1-facets-by-index-stats"></span><span id="response-facets-by-index-additional-properties-stats"></span><span id="response-facets-by-index-stats"></span>        stats:
          type: object
          description: Numeric statistics for each facet
<span id="response-all-of-1-facets-by-index-additional-properties-stats-additional-properties"></span><span id="response-all-of-1-facets-by-index-stats-additional-properties"></span><span id="response-facets-by-index-additional-properties-stats-additional-properties"></span><span id="response-facets-by-index-stats-additional-properties"></span>          additionalProperties:
            $ref: '#/components/schemas/FacetStats'
          propertyNames:
            type: string
    ConditionOutcomes:
      type: object
      description: Outcome of applying conditions to a query.
      required:
        - satisfiesActiveCondition
      properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-active-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfiesactivecondition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-active-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfiesactivecondition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-active-condition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfiesactivecondition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-active-condition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfiesactivecondition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-active-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfiesactivecondition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-active-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfiesactivecondition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-active-condition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfiesactivecondition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-active-condition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfiesactivecondition"></span>        satisfiesActiveCondition:
          type: boolean
          description: Whether the rule was active
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-time-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfiestimecondition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-time-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfiestimecondition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-time-condition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfiestimecondition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-time-condition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfiestimecondition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-time-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfiestimecondition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-time-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfiestimecondition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-time-condition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfiestimecondition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-time-condition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfiestimecondition"></span>        satisfiesTimeCondition:
          $ref: '#/components/schemas/TimeConditionOutcome'
          description: &gt;-
            Whether the current time falls in the range of time specified by the
            condition
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-empty-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfiesqueryemptycondition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-empty-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfiesqueryemptycondition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-empty-condition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfiesqueryemptycondition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-query-empty-condition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfiesqueryemptycondition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-empty-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfiesqueryemptycondition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-empty-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfiesqueryemptycondition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-empty-condition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfiesqueryemptycondition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-query-empty-condition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfiesqueryemptycondition"></span>        satisfiesQueryEmptyCondition:
          $ref: '#/components/schemas/QueryEmptyConditionOutcome'
          description: Whether the query emptiness condition aligns with the actual query
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfiesquerywordscondition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfiesquerywordscondition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfiesquerywordscondition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfiesquerywordscondition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfiesquerywordscondition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfiesquerywordscondition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfiesquerywordscondition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfiesquerywordscondition"></span>        satisfiesQueryWordsCondition:
          $ref: '#/components/schemas/QueryWordsConditionOutcome'
          description: Whether the query words meet the condition
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfiesfiltercondition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfiesfiltercondition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfiesfiltercondition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfiesfiltercondition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfiesfiltercondition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfiesfiltercondition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfiesfiltercondition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfiesfiltercondition"></span>        satisfiesFilterCondition:
          $ref: '#/components/schemas/FilterConditionOutcome'
          description: Whether the filter meets the condition
    Conditions:
      type: object
      properties:
<span id="body-queries-inline-rule-conditions-one-of-0-time"></span><span id="body-queries-inline-rule-conditions-time"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-time"></span><span id="body-queries-inline-rule-one-of-1-conditions-time"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-time"></span><span id="body-queries-items-inline-rule-conditions-time"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-time"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-time"></span>        time:
          oneOf:
<span id="body-queries-inline-rule-conditions-one-of-0-time-one-of-0"></span><span id="body-queries-inline-rule-conditions-time-one-of-0"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-time-one-of-0"></span><span id="body-queries-inline-rule-one-of-1-conditions-time-one-of-0"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-time-one-of-0"></span><span id="body-queries-items-inline-rule-conditions-time-one-of-0"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-time-one-of-0"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-time-one-of-0"></span>            - type: 'null'
<span id="body-queries-inline-rule-conditions-one-of-0-time-one-of-1"></span><span id="body-queries-inline-rule-conditions-time-one-of-1"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-time-one-of-1"></span><span id="body-queries-inline-rule-one-of-1-conditions-time-one-of-1"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-time-one-of-1"></span><span id="body-queries-items-inline-rule-conditions-time-one-of-1"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-time-one-of-1"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-time-one-of-1"></span>            - $ref: '#/components/schemas/TimeCondition'
              description: Time range where the rule is active
<span id="body-queries-inline-rule-conditions-one-of-0-query"></span><span id="body-queries-inline-rule-conditions-query"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-query"></span><span id="body-queries-inline-rule-one-of-1-conditions-query"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-query"></span><span id="body-queries-items-inline-rule-conditions-query"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-query"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-query"></span>        query:
          oneOf:
<span id="body-queries-inline-rule-conditions-one-of-0-query-one-of-0"></span><span id="body-queries-inline-rule-conditions-query-one-of-0"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-query-one-of-0"></span><span id="body-queries-inline-rule-one-of-1-conditions-query-one-of-0"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-query-one-of-0"></span><span id="body-queries-items-inline-rule-conditions-query-one-of-0"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-query-one-of-0"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-query-one-of-0"></span>            - type: 'null'
<span id="body-queries-inline-rule-conditions-one-of-0-query-one-of-1"></span><span id="body-queries-inline-rule-conditions-query-one-of-1"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-query-one-of-1"></span><span id="body-queries-inline-rule-one-of-1-conditions-query-one-of-1"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-query-one-of-1"></span><span id="body-queries-items-inline-rule-conditions-query-one-of-1"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-query-one-of-1"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-query-one-of-1"></span>            - $ref: '#/components/schemas/QueryCondition'
              description: &gt;-
                Conditions on the search query that determines whether the rule
                is active
<span id="body-queries-inline-rule-conditions-filter"></span><span id="body-queries-inline-rule-conditions-one-of-0-filter"></span><span id="body-queries-inline-rule-one-of-1-conditions-filter"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-filter"></span><span id="body-queries-items-inline-rule-conditions-filter"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-filter"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-filter"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-filter"></span>        filter:
          oneOf:
<span id="body-queries-inline-rule-conditions-filter-one-of-0"></span><span id="body-queries-inline-rule-conditions-one-of-0-filter-one-of-0"></span><span id="body-queries-inline-rule-one-of-1-conditions-filter-one-of-0"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-filter-one-of-0"></span><span id="body-queries-items-inline-rule-conditions-filter-one-of-0"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-filter-one-of-0"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-filter-one-of-0"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-filter-one-of-0"></span>            - type: 'null'
<span id="body-queries-inline-rule-conditions-filter-one-of-1"></span><span id="body-queries-inline-rule-conditions-one-of-0-filter-one-of-1"></span><span id="body-queries-inline-rule-one-of-1-conditions-filter-one-of-1"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-filter-one-of-1"></span><span id="body-queries-items-inline-rule-conditions-filter-one-of-1"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-filter-one-of-1"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-filter-one-of-1"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-filter-one-of-1"></span>            - $ref: '#/components/schemas/FilterCondition'
              description: &gt;-
                Conditions on the values matching the filter of the search query
                that determines whether the rule is active
    RuleActions:
      type: object
      description: List of actions to apply when this rule is active for the query.
      properties:
<span id="body-queries-inline-rule-actions-one-of-0-pin"></span><span id="body-queries-inline-rule-actions-pin"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-pin"></span><span id="body-queries-inline-rule-one-of-1-actions-pin"></span><span id="body-queries-items-inline-rule-actions-one-of-0-pin"></span><span id="body-queries-items-inline-rule-actions-pin"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-pin"></span><span id="body-queries-items-inline-rule-one-of-1-actions-pin"></span>        pin:
          type: array
<span id="body-queries-inline-rule-actions-one-of-0-pin-items"></span><span id="body-queries-inline-rule-actions-pin-items"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-pin-items"></span><span id="body-queries-inline-rule-one-of-1-actions-pin-items"></span><span id="body-queries-items-inline-rule-actions-one-of-0-pin-items"></span><span id="body-queries-items-inline-rule-actions-pin-items"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-pin-items"></span><span id="body-queries-items-inline-rule-one-of-1-actions-pin-items"></span>          items:
            $ref: '#/components/schemas/PinAction'
          description: Pins a selected document.
<span id="body-queries-inline-rule-actions-one-of-0-scale"></span><span id="body-queries-inline-rule-actions-scale"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-scale"></span><span id="body-queries-inline-rule-one-of-1-actions-scale"></span><span id="body-queries-items-inline-rule-actions-one-of-0-scale"></span><span id="body-queries-items-inline-rule-actions-scale"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-scale"></span><span id="body-queries-items-inline-rule-one-of-1-actions-scale"></span>        scale:
          type: array
<span id="body-queries-inline-rule-actions-one-of-0-scale-items"></span><span id="body-queries-inline-rule-actions-scale-items"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-scale-items"></span><span id="body-queries-inline-rule-one-of-1-actions-scale-items"></span><span id="body-queries-items-inline-rule-actions-one-of-0-scale-items"></span><span id="body-queries-items-inline-rule-actions-scale-items"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-scale-items"></span><span id="body-queries-items-inline-rule-one-of-1-actions-scale-items"></span>          items:
            $ref: '#/components/schemas/ScaleAction'
          description: Applies a multiplicative factor to the score of selected documents.
    TimeConditionOutcome:
      type: string
      description: Outcome of the time condition
      enum:
        - satisfied
        - noConstraint
        - tooEarly
        - tooLate
    QueryEmptyConditionOutcome:
      type: string
      description: Outcome of the query empty condition
      enum:
        - satisfied
        - noConstraint
        - queryNotEmpty
        - queryEmpty
    QueryWordsConditionOutcome:
      oneOf:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-0"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-0"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-0"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-0"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-0"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-0"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-0"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-0"></span>        - type: string
          enum:
            - satisfied
            - noConstraint
          description: |-
            - `satisfied`: The query contains all the words of the condition
            - `noConstraint`: No query words condition
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1"></span>        - type: object
          description: At least the specified word is missing in the query
          required:
            - missingWord
          properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missing-word"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missingword"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missing-word"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missingword"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-missing-word"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-missingword"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missing-word"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missingword"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missing-word"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missingword"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missing-word"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missingword"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition-missing-word"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition-missingword"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missing-word"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missingword"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missing-word"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missingword"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missing-word"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missingword"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-missing-word"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-missingword"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missing-word"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missingword"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missing-word"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missingword"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missing-word"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missingword"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition-missing-word"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition-missingword"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missing-word"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missingword"></span>            missingWord:
              type: object
              description: At least the specified word is missing in the query
              required:
                - word
              properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missing-word-word"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missing-word-word"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-missing-word-word"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missing-word-word"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missing-word-word"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missing-word-word"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition-missing-word-word"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missing-word-word"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missing-word-word"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missing-word-word"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-missing-word-word"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missing-word-word"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missing-word-word"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missing-word-word"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition-missing-word-word"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missing-word-word"></span>                word:
                  type: string
                  description: Missing word in query
      description: Outcome of the query words condition
    FilterConditionOutcome:
      oneOf:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-0"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-0"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-0"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-0"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-0"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-0"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-0"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-0"></span>        - type: string
          enum:
            - satisfied
            - noConstraint
          description: |-
            - `satisfied`: The filter satifies all constraints in the condition
            - `noConstraint`: No filter condition
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1"></span>        - type: object
          description: At least one field from the condition is missing from the filter
          required:
            - missingConstraintOnField
          properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missing-constraint-on-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missingconstraintonfield"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missing-constraint-on-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missingconstraintonfield"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-missing-constraint-on-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-missingconstraintonfield"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missing-constraint-on-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missingconstraintonfield"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missing-constraint-on-field"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missingconstraintonfield"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missing-constraint-on-field"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missingconstraintonfield"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-missing-constraint-on-field"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-missingconstraintonfield"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missing-constraint-on-field"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missingconstraintonfield"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missing-constraint-on-field"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missingconstraintonfield"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missing-constraint-on-field"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missingconstraintonfield"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-missing-constraint-on-field"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-missingconstraintonfield"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missing-constraint-on-field"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missingconstraintonfield"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missing-constraint-on-field"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missingconstraintonfield"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missing-constraint-on-field"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missingconstraintonfield"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-missing-constraint-on-field"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-missingconstraintonfield"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missing-constraint-on-field"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missingconstraintonfield"></span>            missingConstraintOnField:
              type: object
              description: At least one field from the condition is missing from the filter
              required:
                - field
              properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missing-constraint-on-field-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missing-constraint-on-field-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-missing-constraint-on-field-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missing-constraint-on-field-field"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missing-constraint-on-field-field"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missing-constraint-on-field-field"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-missing-constraint-on-field-field"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missing-constraint-on-field-field"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missing-constraint-on-field-field"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missing-constraint-on-field-field"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-missing-constraint-on-field-field"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missing-constraint-on-field-field"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missing-constraint-on-field-field"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missing-constraint-on-field-field"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-missing-constraint-on-field-field"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missing-constraint-on-field-field"></span>                field:
                  type: string
                  description: Missing field in filter
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2"></span>        - type: object
          description: At least one field has a condition not met in the filter
          required:
            - unmetConstraintOnField
          properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmet-constraint-on-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmetconstraintonfield"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmet-constraint-on-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmetconstraintonfield"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmet-constraint-on-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmetconstraintonfield"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-unmet-constraint-on-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-unmetconstraintonfield"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmet-constraint-on-field"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmetconstraintonfield"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmet-constraint-on-field"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmetconstraintonfield"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmet-constraint-on-field"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmetconstraintonfield"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-unmet-constraint-on-field"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-unmetconstraintonfield"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmet-constraint-on-field"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmetconstraintonfield"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmet-constraint-on-field"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmetconstraintonfield"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmet-constraint-on-field"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmetconstraintonfield"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-unmet-constraint-on-field"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-unmetconstraintonfield"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmet-constraint-on-field"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmetconstraintonfield"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmet-constraint-on-field"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmetconstraintonfield"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmet-constraint-on-field"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmetconstraintonfield"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-unmet-constraint-on-field"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-unmetconstraintonfield"></span>            unmetConstraintOnField:
              type: object
              description: At least one field has a condition not met in the filter
              required:
                - field
              properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmet-constraint-on-field-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmet-constraint-on-field-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmet-constraint-on-field-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-unmet-constraint-on-field-field"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmet-constraint-on-field-field"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmet-constraint-on-field-field"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmet-constraint-on-field-field"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-unmet-constraint-on-field-field"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmet-constraint-on-field-field"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmet-constraint-on-field-field"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmet-constraint-on-field-field"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-unmet-constraint-on-field-field"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmet-constraint-on-field-field"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmet-constraint-on-field-field"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmet-constraint-on-field-field"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-unmet-constraint-on-field-field"></span>                field:
                  type: string
                  description: Field with unmet constraint in filter
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3"></span>        - type: object
          description: The filter does not constraint enough field to satisfy the condition
          required:
            - notEnoughFields
          properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-notenoughfields"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-notenoughfields"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-notenoughfields"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-notenoughfields"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-notenoughfields"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-notenoughfields"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-notenoughfields"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-notenoughfields"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-notenoughfields"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-notenoughfields"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-notenoughfields"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-notenoughfields"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-notenoughfields"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-notenoughfields"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-notenoughfields"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-notenoughfields"></span>            notEnoughFields:
              type: object
              description: &gt;-
                The filter does not constraint enough field to satisfy the
                condition
              required:
                - fieldCountInFilter
                - fieldCountInCondition
              properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-field-count-in-filter"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-fieldcountinfilter"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-filter"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountinfilter"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-field-count-in-filter"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-fieldcountinfilter"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-filter"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountinfilter"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-field-count-in-filter"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-fieldcountinfilter"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-filter"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountinfilter"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-field-count-in-filter"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-fieldcountinfilter"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-filter"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountinfilter"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-field-count-in-filter"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-fieldcountinfilter"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-filter"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountinfilter"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-field-count-in-filter"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-fieldcountinfilter"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-filter"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountinfilter"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-field-count-in-filter"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-fieldcountinfilter"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-filter"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountinfilter"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-field-count-in-filter"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-fieldcountinfilter"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-filter"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountinfilter"></span>                fieldCountInFilter:
                  type: integer
                  description: Number of fields in the filter
                  minimum: 0
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-field-count-in-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-fieldcountincondition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountincondition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-field-count-in-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-fieldcountincondition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountincondition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-field-count-in-condition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-fieldcountincondition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-condition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountincondition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-field-count-in-condition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-fieldcountincondition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-condition"></span><span id="response-all-of-1-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountincondition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-field-count-in-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-fieldcountincondition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountincondition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-field-count-in-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-fieldcountincondition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountincondition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-field-count-in-condition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-fieldcountincondition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-condition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountincondition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-field-count-in-condition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-fieldcountincondition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-condition"></span><span id="response-metadata-items-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountincondition"></span>                fieldCountInCondition:
                  type: integer
                  description: Number of fields in the condition
                  minimum: 0
      description: Outcome of the filter condition
    TimeCondition:
      type: object
      properties:
<span id="body-queries-inline-rule-conditions-one-of-0-time-one-of-1-start"></span><span id="body-queries-inline-rule-conditions-one-of-0-time-start"></span><span id="body-queries-inline-rule-conditions-time-one-of-1-start"></span><span id="body-queries-inline-rule-conditions-time-start"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-time-one-of-1-start"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-time-start"></span><span id="body-queries-inline-rule-one-of-1-conditions-time-one-of-1-start"></span><span id="body-queries-inline-rule-one-of-1-conditions-time-start"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-time-one-of-1-start"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-time-start"></span><span id="body-queries-items-inline-rule-conditions-time-one-of-1-start"></span><span id="body-queries-items-inline-rule-conditions-time-start"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-time-one-of-1-start"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-time-start"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-time-one-of-1-start"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-time-start"></span>        start:
          type:
            - string
            - 'null'
          format: date-time
          description: |-
            Start of the time range where this rule can be considered active.

            Specify as a RFC3339 datetime.
<span id="body-queries-inline-rule-conditions-one-of-0-time-end"></span><span id="body-queries-inline-rule-conditions-one-of-0-time-one-of-1-end"></span><span id="body-queries-inline-rule-conditions-time-end"></span><span id="body-queries-inline-rule-conditions-time-one-of-1-end"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-time-end"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-time-one-of-1-end"></span><span id="body-queries-inline-rule-one-of-1-conditions-time-end"></span><span id="body-queries-inline-rule-one-of-1-conditions-time-one-of-1-end"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-time-end"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-time-one-of-1-end"></span><span id="body-queries-items-inline-rule-conditions-time-end"></span><span id="body-queries-items-inline-rule-conditions-time-one-of-1-end"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-time-end"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-time-one-of-1-end"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-time-end"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-time-one-of-1-end"></span>        end:
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
<span id="body-queries-inline-rule-conditions-one-of-0-query-is-empty"></span><span id="body-queries-inline-rule-conditions-one-of-0-query-isempty"></span><span id="body-queries-inline-rule-conditions-one-of-0-query-one-of-1-is-empty"></span><span id="body-queries-inline-rule-conditions-one-of-0-query-one-of-1-isempty"></span><span id="body-queries-inline-rule-conditions-query-is-empty"></span><span id="body-queries-inline-rule-conditions-query-isempty"></span><span id="body-queries-inline-rule-conditions-query-one-of-1-is-empty"></span><span id="body-queries-inline-rule-conditions-query-one-of-1-isempty"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-query-is-empty"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-query-isempty"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-query-one-of-1-is-empty"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-query-one-of-1-isempty"></span><span id="body-queries-inline-rule-one-of-1-conditions-query-is-empty"></span><span id="body-queries-inline-rule-one-of-1-conditions-query-isempty"></span><span id="body-queries-inline-rule-one-of-1-conditions-query-one-of-1-is-empty"></span><span id="body-queries-inline-rule-one-of-1-conditions-query-one-of-1-isempty"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-query-is-empty"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-query-isempty"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-query-one-of-1-is-empty"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-query-one-of-1-isempty"></span><span id="body-queries-items-inline-rule-conditions-query-is-empty"></span><span id="body-queries-items-inline-rule-conditions-query-isempty"></span><span id="body-queries-items-inline-rule-conditions-query-one-of-1-is-empty"></span><span id="body-queries-items-inline-rule-conditions-query-one-of-1-isempty"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-query-is-empty"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-query-isempty"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-query-one-of-1-is-empty"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-query-one-of-1-isempty"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-query-is-empty"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-query-isempty"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-query-one-of-1-is-empty"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-query-one-of-1-isempty"></span>        isEmpty:
          type:
            - boolean
            - 'null'
          description: &gt;-
            If present and non-null, specifies either:


            - That this rule can only be active when the search query is empty

            - That this rule can only be active when the search query is
            non-empty (contains at least one word)
<span id="body-queries-inline-rule-conditions-one-of-0-query-one-of-1-words"></span><span id="body-queries-inline-rule-conditions-one-of-0-query-words"></span><span id="body-queries-inline-rule-conditions-query-one-of-1-words"></span><span id="body-queries-inline-rule-conditions-query-words"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-query-one-of-1-words"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-query-words"></span><span id="body-queries-inline-rule-one-of-1-conditions-query-one-of-1-words"></span><span id="body-queries-inline-rule-one-of-1-conditions-query-words"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-query-one-of-1-words"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-query-words"></span><span id="body-queries-items-inline-rule-conditions-query-one-of-1-words"></span><span id="body-queries-items-inline-rule-conditions-query-words"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-query-one-of-1-words"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-query-words"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-query-one-of-1-words"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-query-words"></span>        words:
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
<span id="body-queries-inline-rule-conditions-filter-one-of-1-values"></span><span id="body-queries-inline-rule-conditions-filter-values"></span><span id="body-queries-inline-rule-conditions-one-of-0-filter-one-of-1-values"></span><span id="body-queries-inline-rule-conditions-one-of-0-filter-values"></span><span id="body-queries-inline-rule-one-of-1-conditions-filter-one-of-1-values"></span><span id="body-queries-inline-rule-one-of-1-conditions-filter-values"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-filter-one-of-1-values"></span><span id="body-queries-inline-rule-one-of-1-conditions-one-of-0-filter-values"></span><span id="body-queries-items-inline-rule-conditions-filter-one-of-1-values"></span><span id="body-queries-items-inline-rule-conditions-filter-values"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-filter-one-of-1-values"></span><span id="body-queries-items-inline-rule-conditions-one-of-0-filter-values"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-filter-one-of-1-values"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-filter-values"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-filter-one-of-1-values"></span><span id="body-queries-items-inline-rule-one-of-1-conditions-one-of-0-filter-values"></span>        values:
          type: object
          additionalProperties: {}
          propertyNames:
            type: string
    PinAction:
      type: object
      description: An action that pins a selected document.
      required:
        - id
        - position
      properties:
<span id="body-queries-inline-rule-actions-one-of-0-pin-index-uid"></span><span id="body-queries-inline-rule-actions-one-of-0-pin-indexuid"></span><span id="body-queries-inline-rule-actions-one-of-0-pin-items-index-uid"></span><span id="body-queries-inline-rule-actions-one-of-0-pin-items-indexuid"></span><span id="body-queries-inline-rule-actions-pin-index-uid"></span><span id="body-queries-inline-rule-actions-pin-indexuid"></span><span id="body-queries-inline-rule-actions-pin-items-index-uid"></span><span id="body-queries-inline-rule-actions-pin-items-indexuid"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-pin-index-uid"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-pin-indexuid"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-pin-items-index-uid"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-pin-items-indexuid"></span><span id="body-queries-inline-rule-one-of-1-actions-pin-index-uid"></span><span id="body-queries-inline-rule-one-of-1-actions-pin-indexuid"></span><span id="body-queries-inline-rule-one-of-1-actions-pin-items-index-uid"></span><span id="body-queries-inline-rule-one-of-1-actions-pin-items-indexuid"></span><span id="body-queries-items-inline-rule-actions-one-of-0-pin-index-uid"></span><span id="body-queries-items-inline-rule-actions-one-of-0-pin-indexuid"></span><span id="body-queries-items-inline-rule-actions-one-of-0-pin-items-index-uid"></span><span id="body-queries-items-inline-rule-actions-one-of-0-pin-items-indexuid"></span><span id="body-queries-items-inline-rule-actions-pin-index-uid"></span><span id="body-queries-items-inline-rule-actions-pin-indexuid"></span><span id="body-queries-items-inline-rule-actions-pin-items-index-uid"></span><span id="body-queries-items-inline-rule-actions-pin-items-indexuid"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-pin-index-uid"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-pin-indexuid"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-pin-items-index-uid"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-pin-items-indexuid"></span><span id="body-queries-items-inline-rule-one-of-1-actions-pin-index-uid"></span><span id="body-queries-items-inline-rule-one-of-1-actions-pin-indexuid"></span><span id="body-queries-items-inline-rule-one-of-1-actions-pin-items-index-uid"></span><span id="body-queries-items-inline-rule-one-of-1-actions-pin-items-indexuid"></span>        indexUid:
          type:
            - string
            - 'null'
          description: &gt;-
            Index name.


            For the action to select any document, when this parameter is
            provided,

            the index of the query must match the provided parameter.
<span id="body-queries-inline-rule-actions-one-of-0-pin-id"></span><span id="body-queries-inline-rule-actions-one-of-0-pin-items-id"></span><span id="body-queries-inline-rule-actions-pin-id"></span><span id="body-queries-inline-rule-actions-pin-items-id"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-pin-id"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-pin-items-id"></span><span id="body-queries-inline-rule-one-of-1-actions-pin-id"></span><span id="body-queries-inline-rule-one-of-1-actions-pin-items-id"></span><span id="body-queries-items-inline-rule-actions-one-of-0-pin-id"></span><span id="body-queries-items-inline-rule-actions-one-of-0-pin-items-id"></span><span id="body-queries-items-inline-rule-actions-pin-id"></span><span id="body-queries-items-inline-rule-actions-pin-items-id"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-pin-id"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-pin-items-id"></span><span id="body-queries-items-inline-rule-one-of-1-actions-pin-id"></span><span id="body-queries-items-inline-rule-one-of-1-actions-pin-items-id"></span>        id:
          type: string
          description: &gt;-
            Document ID of the document to select.


            Only the document whose [primary
            key](https://www.meilisearch.com/docs/learn/getting_started/primary_key)
            value

            matches the specified id will be selected by the action.


            If there is no such document in the index of the query, then no
            documents will be selected and no pinning will occur.
<span id="body-queries-inline-rule-actions-one-of-0-pin-items-position"></span><span id="body-queries-inline-rule-actions-one-of-0-pin-position"></span><span id="body-queries-inline-rule-actions-pin-items-position"></span><span id="body-queries-inline-rule-actions-pin-position"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-pin-items-position"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-pin-position"></span><span id="body-queries-inline-rule-one-of-1-actions-pin-items-position"></span><span id="body-queries-inline-rule-one-of-1-actions-pin-position"></span><span id="body-queries-items-inline-rule-actions-one-of-0-pin-items-position"></span><span id="body-queries-items-inline-rule-actions-one-of-0-pin-position"></span><span id="body-queries-items-inline-rule-actions-pin-items-position"></span><span id="body-queries-items-inline-rule-actions-pin-position"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-pin-items-position"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-pin-position"></span><span id="body-queries-items-inline-rule-one-of-1-actions-pin-items-position"></span><span id="body-queries-items-inline-rule-one-of-1-actions-pin-position"></span>        position:
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
<span id="body-queries-inline-rule-actions-one-of-0-scale-index-uid"></span><span id="body-queries-inline-rule-actions-one-of-0-scale-indexuid"></span><span id="body-queries-inline-rule-actions-one-of-0-scale-items-index-uid"></span><span id="body-queries-inline-rule-actions-one-of-0-scale-items-indexuid"></span><span id="body-queries-inline-rule-actions-scale-index-uid"></span><span id="body-queries-inline-rule-actions-scale-indexuid"></span><span id="body-queries-inline-rule-actions-scale-items-index-uid"></span><span id="body-queries-inline-rule-actions-scale-items-indexuid"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-scale-index-uid"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-scale-indexuid"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-scale-items-index-uid"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-scale-items-indexuid"></span><span id="body-queries-inline-rule-one-of-1-actions-scale-index-uid"></span><span id="body-queries-inline-rule-one-of-1-actions-scale-indexuid"></span><span id="body-queries-inline-rule-one-of-1-actions-scale-items-index-uid"></span><span id="body-queries-inline-rule-one-of-1-actions-scale-items-indexuid"></span><span id="body-queries-items-inline-rule-actions-one-of-0-scale-index-uid"></span><span id="body-queries-items-inline-rule-actions-one-of-0-scale-indexuid"></span><span id="body-queries-items-inline-rule-actions-one-of-0-scale-items-index-uid"></span><span id="body-queries-items-inline-rule-actions-one-of-0-scale-items-indexuid"></span><span id="body-queries-items-inline-rule-actions-scale-index-uid"></span><span id="body-queries-items-inline-rule-actions-scale-indexuid"></span><span id="body-queries-items-inline-rule-actions-scale-items-index-uid"></span><span id="body-queries-items-inline-rule-actions-scale-items-indexuid"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-scale-index-uid"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-scale-indexuid"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-scale-items-index-uid"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-scale-items-indexuid"></span><span id="body-queries-items-inline-rule-one-of-1-actions-scale-index-uid"></span><span id="body-queries-items-inline-rule-one-of-1-actions-scale-indexuid"></span><span id="body-queries-items-inline-rule-one-of-1-actions-scale-items-index-uid"></span><span id="body-queries-items-inline-rule-one-of-1-actions-scale-items-indexuid"></span>        indexUid:
          type:
            - string
            - 'null'
          description: &gt;-
            List of index patterns.


            For the action to select any document, when this parameter is
            provided,

            the index of the query must match the provided parameter.
<span id="body-queries-inline-rule-actions-one-of-0-scale-ids"></span><span id="body-queries-inline-rule-actions-one-of-0-scale-items-ids"></span><span id="body-queries-inline-rule-actions-scale-ids"></span><span id="body-queries-inline-rule-actions-scale-items-ids"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-scale-ids"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-scale-items-ids"></span><span id="body-queries-inline-rule-one-of-1-actions-scale-ids"></span><span id="body-queries-inline-rule-one-of-1-actions-scale-items-ids"></span><span id="body-queries-items-inline-rule-actions-one-of-0-scale-ids"></span><span id="body-queries-items-inline-rule-actions-one-of-0-scale-items-ids"></span><span id="body-queries-items-inline-rule-actions-scale-ids"></span><span id="body-queries-items-inline-rule-actions-scale-items-ids"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-scale-ids"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-scale-items-ids"></span><span id="body-queries-items-inline-rule-one-of-1-actions-scale-ids"></span><span id="body-queries-items-inline-rule-one-of-1-actions-scale-items-ids"></span>        ids:
          type:
            - array
            - 'null'
          items:
            type: string
          description: &gt;-
            Array of specific document IDs to select.


            Only documents whose [primary
            key](https://www.meilisearch.com/docs/learn/getting_started/primary_key)
            value

            matches the specified ids will be selected by the action.


            If `filter` is also specified,

            the documents must also satisfy the filter to be selected.
<span id="body-queries-inline-rule-actions-one-of-0-scale-filter"></span><span id="body-queries-inline-rule-actions-one-of-0-scale-items-filter"></span><span id="body-queries-inline-rule-actions-scale-filter"></span><span id="body-queries-inline-rule-actions-scale-items-filter"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-scale-filter"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-scale-items-filter"></span><span id="body-queries-inline-rule-one-of-1-actions-scale-filter"></span><span id="body-queries-inline-rule-one-of-1-actions-scale-items-filter"></span><span id="body-queries-items-inline-rule-actions-one-of-0-scale-filter"></span><span id="body-queries-items-inline-rule-actions-one-of-0-scale-items-filter"></span><span id="body-queries-items-inline-rule-actions-scale-filter"></span><span id="body-queries-items-inline-rule-actions-scale-items-filter"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-scale-filter"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-scale-items-filter"></span><span id="body-queries-items-inline-rule-one-of-1-actions-scale-filter"></span><span id="body-queries-items-inline-rule-one-of-1-actions-scale-items-filter"></span>        filter:
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
<span id="body-queries-inline-rule-actions-one-of-0-scale-items-weight"></span><span id="body-queries-inline-rule-actions-one-of-0-scale-weight"></span><span id="body-queries-inline-rule-actions-scale-items-weight"></span><span id="body-queries-inline-rule-actions-scale-weight"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-scale-items-weight"></span><span id="body-queries-inline-rule-one-of-1-actions-one-of-0-scale-weight"></span><span id="body-queries-inline-rule-one-of-1-actions-scale-items-weight"></span><span id="body-queries-inline-rule-one-of-1-actions-scale-weight"></span><span id="body-queries-items-inline-rule-actions-one-of-0-scale-items-weight"></span><span id="body-queries-items-inline-rule-actions-one-of-0-scale-weight"></span><span id="body-queries-items-inline-rule-actions-scale-items-weight"></span><span id="body-queries-items-inline-rule-actions-scale-weight"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-scale-items-weight"></span><span id="body-queries-items-inline-rule-one-of-1-actions-one-of-0-scale-weight"></span><span id="body-queries-items-inline-rule-one-of-1-actions-scale-items-weight"></span><span id="body-queries-items-inline-rule-one-of-1-actions-scale-weight"></span>        weight:
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