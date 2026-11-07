# Search with GET

> Search for documents matching a query in the given index.

> Equivalent to the [search with POST route](/docs/reference/api/search/search-with-post) in the Meilisearch API.

**Note:** By default this endpoint returns at most 1000 results. Configure `pagination.maxTotalHits` in index settings to change this limit.

**Note:** The GET route only accepts string filter expressions. Use the POST route if you need array-of-array filter syntax.



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
  /indexes/{index_uid}/search:
    get:
      tags:
        - Search
      summary: Search with GET
      description: &gt;-
        Search for documents matching a query in the given index.


        &gt; Equivalent to the [search with POST
        route](/docs/reference/api/search/search-with-post) in the Meilisearch
        API.


        **Note:** By default this endpoint returns at most 1000 results.
        Configure `pagination.maxTotalHits` in index settings to change this
        limit.


        **Note:** The GET route only accepts string filter expressions. Use the
        POST route if you need array-of-array filter syntax.
      operationId: search_with_url_query
      parameters:
        - name: index_uid
<span id="parameter-index-uid"></span>          in: path
          description: Unique identifier of the index.
          required: true
          schema:
            type: string
          example: movies
        - name: q
<span id="parameter-q"></span>          in: query
          description: &gt;-
            Sets the search terms.


            Meilisearch returns documents that match this query.


            The query supports [prefix
            search](https://www.meilisearch.com/docs/learn/engine/prefix) and
            [typo
            tolerance](https://www.meilisearch.com/docs/learn/relevancy/typo_tolerance_settings).


            Meilisearch only considers the first ten words; terms are normalized
            (lowercase, accents ignored).


            Omit or leave empty for a placeholder search: no query terms are
            applied, so Meilisearch returns all searchable documents in the
            index, ordered by [ranking
            rules](https://www.meilisearch.com/docs/learn/relevancy/ranking_rules).


            Enclose terms in double quotes (`"`) for phrase search: only
            documents containing that exact sequence of words are returned (e.g.
            `"Winter Feast"`).


            Use a minus sign (`-`) before a word or phrase to exclude it from
            results.
          required: false
          schema:
            type: string
        - name: offset
<span id="parameter-offset"></span>          in: query
          description: &gt;-
            Number of documents to skip at the start of the results.


            Use together with `limit` for
            [pagination](https://www.meilisearch.com/docs/guides/front_end/pagination)
            (e.g. offset=20 and limit=20 returns results 21–40).


            This parameter is ignored when `page` or `hitsPerPage` is set; in
            that case the response includes `totalHits` and `totalPages` instead
            of `estimatedTotalHits`.
          required: false
          schema:
            type: integer
            default: 0
            minimum: 0
        - name: limit
<span id="parameter-limit"></span>          in: query
          description: &gt;-
            Maximum number of documents to return in the response.


            Use with `offset` for
            [pagination](https://www.meilisearch.com/docs/guides/front_end/pagination).


            This parameter is ignored when `page` or `hitsPerPage` is set.


            The value cannot exceed the index
            [maxTotalHits](https://www.meilisearch.com/docs/reference/api/settings/update-pagination#body-max-total-hits-one-of-0)
            setting.
          required: false
          schema:
            type: integer
            default: 20
            minimum: 0
        - name: page
<span id="parameter-page"></span>          in: query
          description: &gt;-
            Request a specific results page (1-indexed).


            Use together with `hitsPerPage`.


            When this parameter is set, the response includes `totalHits` and
            `totalPages` instead of `estimatedTotalHits`.


            `page` and `hitsPerPage` take precedence over `offset` and `limit`.
          required: false
          schema:
            type: integer
            minimum: 0
        - name: hitsPerPage
<span id="parameter-hits-per-page"></span>          in: query
          description: &gt;-
            Maximum number of documents per page for
            [pagination](https://www.meilisearch.com/docs/guides/front_end/pagination).


            This value determines `totalPages`; use it together with `page`.


            When set, the response includes `totalHits` and `totalPages`.


            Set to 0 to obtain the exhaustive `totalHits` count without
            returning any documents.
          required: false
          schema:
            type: integer
            minimum: 0
        - name: attributesToRetrieve
<span id="parameter-attributes-to-retrieve"></span>          in: query
          description: &gt;-
            List of attributes to include in each returned document.


            Use `["*"]` to return all attributes; if not set, the index
            [displayed
            attributes](https://www.meilisearch.com/docs/learn/relevancy/displayed_searchable_attributes)
            list is used.


            Attributes that are not in
            [displayedAttributes](https://www.meilisearch.com/docs/reference/api/settings/update-all-settings#body-displayed-attributes-one-of-0)
            are omitted from the response.
          required: false
          schema:
            type: array
<span id="parameter-attributes-to-retrieve-items"></span>            items:
              type: string
          explode: false
        - name: attributesToCrop
<span id="parameter-attributes-to-crop"></span>          in: query
          description: &gt;-
            Attributes whose values should be cropped to a short excerpt.


            The cropped text appears in each hit's `_formatted` object.


            Length is controlled by `cropLength`, or you can override it per
            attribute with the `attribute:length` syntax.


            Use `["*"]` to crop all attributes in `attributesToRetrieve`.


            When possible, the crop is centered around the matching terms.
          required: false
          schema:
            type: array
<span id="parameter-attributes-to-crop-items"></span>            items:
              type: string
          explode: false
        - name: cropLength
<span id="parameter-crop-length"></span>          in: query
          description: &gt;-
            Maximum number of words to include in cropped values.


            This parameter only applies when `attributesToCrop` is set.


            Both query terms and [stop
            words](https://www.meilisearch.com/docs/reference/api/settings/update-all-settings#body-stop-words-one-of-0)
            count toward this length.
          required: false
          schema:
            type: integer
            default: 10
            minimum: 0
        - name: cropMarker
<span id="parameter-crop-marker"></span>          in: query
          description: |-
            String used to mark crop boundaries in cropped text.

            If null or empty, no markers are inserted.

            Markers are only added where content was actually removed.
          required: false
          schema:
            type: string
            default: …
        - name: attributesToHighlight
<span id="parameter-attributes-to-highlight"></span>          in: query
          description: &gt;-
            Attributes in which matching query terms should be highlighted.


            The highlighted text appears in each hit's `_formatted` object.


            Use `["*"]` to highlight in all attributes from
            `attributesToRetrieve`.


            By default, matches are wrapped in `&lt;em&gt;` and `&lt;/em&gt;`; you can
            override this with `highlightPreTag` and `highlightPostTag`.


            Highlighting also applies to
            [synonyms](https://www.meilisearch.com/docs/learn/relevancy/synonyms)
            and [stop
            words](https://www.meilisearch.com/docs/reference/api/settings/update-all-settings#body-stop-words-one-of-0).


            Supported value types are string, number, array, and object.


            Note: highlights matches within all listed attributes, even those
            not in `searchableAttributes`.
          required: false
          schema:
            type: array
<span id="parameter-attributes-to-highlight-items"></span>            items:
              type: string
          explode: false
        - name: highlightPreTag
<span id="parameter-highlight-pre-tag"></span>          in: query
          description: |-
            String to insert before each highlighted term.

            Can be any string (e.g. `&lt;strong&gt;`, `*`).

            If null or empty, nothing is inserted at the start of a match.
          required: false
          schema:
            type: string
            default: &lt;em&gt;
        - name: highlightPostTag
<span id="parameter-highlight-post-tag"></span>          in: query
          description: &gt;-
            String to insert after each highlighted term.


            Should be used together with `highlightPreTag` to avoid malformed
            output (e.g. unclosed HTML tags).
          required: false
          schema:
            type: string
            default: &lt;/em&gt;
        - name: showMatchesPosition
<span id="parameter-show-matches-position"></span>          in: query
          description: &gt;-
            When true, each hit includes a `_matchesPosition` object with the
            byte offset (`start` and `length`) of each matched term.


            This is useful when you need custom highlighting.


            Note: reports match positions in all attributes, even non-searchable
            ones. Positions are measured in bytes, not characters.
          required: false
          schema:
            type: boolean
        - name: filter
<span id="parameter-filter"></span>          in: query
          description: &gt;-
            A
            [filter](https://www.meilisearch.com/docs/learn/filtering_and_sorting/filter_search_results)
            expression to narrow results.


            All attributes used in the expression must be in
            [filterableAttributes](https://www.meilisearch.com/docs/reference/api/settings/update-all-settings#body-filterable-attributes-one-of-0).


            Pass a string (e.g. `"(genres = horror OR genres = mystery) AND
            director = 'Jordan Peele'"`).


            For [geo
            search](https://www.meilisearch.com/docs/learn/filtering_and_sorting/geosearch),
            use `_geoRadius(lat, lng, distance_in_meters)`,
            `_geoBoundingBox([lat,lng],[lat,lng])`, or `_geoPolygon([lat,lng],
            ...)` (GeoJSON only for polygon).


            GET route accepts a string only; the value must be URL-encoded.
          required: false
          schema:
            type: string
        - name: sort
<span id="parameter-sort"></span>          in: query
          description: &gt;-
            Sort results by one or more attributes and their order.


            Use the format `["attribute:asc", "attribute:desc"]`; only
            attributes in
            [sortableAttributes](https://www.meilisearch.com/docs/reference/api/settings/update-all-settings#body-sortable-attributes-one-of-0)
            can be used.


            For [geo
            search](https://www.meilisearch.com/docs/learn/filtering_and_sorting/geosearch),
            use `_geoPoint(lat,lng):asc` or `:desc`; the response then includes
            `_geoDistance` in meters.


            The first attribute in the list has precedence.


            See [sorting search
            results](https://www.meilisearch.com/docs/learn/filtering_and_sorting/sort_search_results).
          required: false
          schema:
            type: string
        - name: distinct
<span id="parameter-distinct"></span>          in: query
          description: &gt;-
            Return only one document per distinct value of the given attribute
            (e.g. deduplicate by product_id).


            The attribute must be in
            [filterableAttributes](https://www.meilisearch.com/docs/reference/api/settings/update-all-settings#body-filterable-attributes-one-of-0).


            This overrides the index
            [distinctAttribute](https://www.meilisearch.com/docs/reference/api/settings/update-all-settings#body-distinct-attribute-one-of-0)
            setting for this request.


            See [distinct
            attribute](https://www.meilisearch.com/docs/learn/relevancy/distinct_attribute).
          required: false
          schema:
            type: string
        - name: facets
<span id="parameter-facets"></span>          in: query
          description: &gt;-
            Return the count of matches per facet value for the listed
            attributes.


            The response includes `facetDistribution` and, for numeric facets,
            `facetStats` (min/max).


            This route also supports patterns, i.e., "title", "dogs.*", "*",
            which can match over the
            [filterableAttributes](https://www.meilisearch.com/docs/reference/api/settings/update-all-settings#body-filterable-attributes-one-of-0).


            The number of values returned per facet is limited by the index
            [maxValuesPerFacet](https://www.meilisearch.com/docs/reference/api/settings/update-faceting#body-max-values-per-facet-one-of-0)
            setting; attributes not in filterableAttributes are ignored.


            More info:
            [faceting](https://www.meilisearch.com/docs/learn/filtering_and_sorting/search_with_facet_filters).
          required: false
          schema:
            type: array
<span id="parameter-facets-items"></span>            items:
              type: string
          explode: false
        - name: matchingStrategy
<span id="parameter-matching-strategy"></span>          in: query
          description: &gt;-
            How to match query terms when there are not enough results to
            satisfy `limit`.


            **`last`**: Returns documents containing all query terms first. If
            there are not enough such results, Meilisearch removes one query
            term at a time, starting from the end of the query (e.g. for "big
            fat cat", then "big fat", then "big").


            **`all`**: Only returns documents that contain all query terms.
            Meilisearch does not relax the query even if fewer than `limit`
            documents match.


            **`frequency`**: Returns documents containing all query terms first.
            If there are not enough, removes one term at a time starting with
            the word that is most frequent in the dataset, giving more weight to
            rarer terms (e.g. in "white cotton shirt", prioritizes documents
            containing "white" if "shirt" is very common).


            Default: `last`.
          required: false
          schema:
            $ref: '#/components/schemas/MatchingStrategy'
        - name: attributesToSearchOn
<span id="parameter-attributes-to-search-on"></span>          in: query
          description: &gt;-
            Restrict the search to the listed attributes only.


            Each attribute must be in the index [searchable
            attributes](https://www.meilisearch.com/docs/learn/relevancy/displayed_searchable_attributes)
            list.


            The order of attributes in this parameter does not affect relevancy.
          required: false
          schema:
            type: array
<span id="parameter-attributes-to-search-on-items"></span>            items:
              type: string
          explode: false
        - name: rankingScoreThreshold
<span id="parameter-ranking-score-threshold"></span>          in: query
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
          required: false
          schema:
            type: number
            format: float
        - name: locales
<span id="parameter-locales"></span>          in: query
          description: &gt;-
            Explicitly specify the language(s) of the query.


            Pass an array of [supported ISO-639
            locales](https://www.meilisearch.com/docs/reference/api/settings/update-all-settings#body-localized-attributes-one-of-0).


            This overrides auto-detection; use it when auto-detection is wrong
            for the query or the documents.


            See also the
            [localizedAttributes](https://www.meilisearch.com/docs/reference/api/settings/list-all-settings#response-localized-attributes-one-of-0)
            settings and
            [Language](https://www.meilisearch.com/docs/learn/resources/language).
          required: false
          schema:
            type: array
<span id="parameter-locales-items"></span>            items:
              $ref: '#/components/schemas/Locale'
          explode: false
        - name: hybridEmbedder
<span id="parameter-hybrid-embedder"></span>          in: query
          description: &gt;-
            Name of the embedder for [hybrid
            search](https://www.meilisearch.com/docs/learn/ai_powered_search/getting_started_with_ai_search),
            which combines keyword and semantic search.


            Must match an embedder configured in the index settings.


            Required when `vector` or `hybridSemanticRatio` is set.
          required: false
          schema:
            type: string
        - name: hybridSemanticRatio
<span id="parameter-hybrid-semantic-ratio"></span>          in: query
          description: &gt;-
            Balance between keyword and semantic search: 0.0 means keyword-only
            results, 1.0 means semantic-only.


            When `q` is empty and this value is greater than 0, Meilisearch
            performs a pure semantic search.


            Requires `hybridEmbedder` when set.
          required: false
          schema:
            type: number
            format: float
        - name: vector
<span id="parameter-vector"></span>          in: query
          description: &gt;-
            Custom query vector for [vector or hybrid
            search](https://www.meilisearch.com/docs/learn/ai_powered_search/getting_started_with_ai_search).


            The array length must match the dimensions of the embedder
            configured in the index.


            This parameter is mandatory when using a [user-provided
            embedder](https://www.meilisearch.com/docs/learn/ai_powered_search/search_with_user_provided_embeddings).


            When used with `hybrid`, documents are ranked by vector similarity.


            You can also use it to override an embedder's automatic vector
            generation.
          required: false
          schema:
            type: array
<span id="parameter-vector-items"></span>            items:
              type: number
              format: float
          explode: false
        - name: retrieveVectors
<span id="parameter-retrieve-vectors"></span>          in: query
          description: &gt;-
            When true, the response includes document and query embeddings in
            each hit's `_vectors` field.


            The `_vectors` field must be listed in
            [displayedAttributes](https://www.meilisearch.com/docs/reference/api/settings/update-all-settings#body-displayed-attributes-one-of-0)
            for it to appear.
          required: false
          schema:
            type: boolean
        - name: personalizeUserContext
<span id="parameter-personalize-user-context"></span>          in: query
          description: &gt;-
            For [personalized
            search](https://www.meilisearch.com/docs/learn/personalization/making_personalized_search_queries):
            a string describing the user (e.g. preferences or behavior).


            Results are then tailored to that profile.


            Personalization must be
            [enabled](https://www.meilisearch.com/docs/reference/api/experimental-features/configure-experimental-features)
            (e.g. Cohere key for self-hosted instances).
          required: false
          schema:
            type: string
        - name: useNetwork
<span id="parameter-use-network"></span>          in: query
          description: &gt;-
            When `true`, runs the query on the whole network (all shards
            covered, documents deduplicated across remotes).


            When `false` or omitted, the query runs locally.


            It also requires the `network` [experimental
            feature](http://localhost:3000/reference/api/experimental-features/configure-experimental-features).


            Values: `true` = use the whole network; `false` or omitted = local
            (default).


            When using the network, the index must exist with compatible
            settings on all remotes.


            Documents with the same id are assumed identical for deduplication.
          required: false
          schema:
            type: boolean
        - name: showRankingScore
<span id="parameter-show-ranking-score"></span>          in: query
          description: &gt;-
            When true, each document includes a `_rankingScore` between 0.0 and
            1.0; a higher value means the document is more relevant.


            See [ranking
            score](https://www.meilisearch.com/docs/learn/relevancy/ranking_score).


            The `sort` ranking rule does not affect the value of
            `_rankingScore`.
          required: false
          schema:
            type: boolean
        - name: showRankingScoreDetails
<span id="parameter-show-ranking-score-details"></span>          in: query
          description: &gt;-
            When true, each document includes `_rankingScoreDetails`, which
            breaks down the score contribution of each [ranking
            rule](https://www.meilisearch.com/docs/learn/relevancy/ranking_rules).


            Useful for debugging relevancy.
          required: false
          schema:
            type: boolean
        - name: showPerformanceDetails
<span id="parameter-show-performance-details"></span>          in: query
          description: &gt;-
            When true, the response includes a `performanceDetails` object with
            a timing breakdown of the query processing.
          required: false
          schema:
            type: boolean
      responses:
        '200':
          description: The documents are returned.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/SearchResult'
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
                offset: 0
                limit: 2
                estimatedTotalHits: 976
                processingTimeMs: 35
                query: 'american '
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
              -X GET 'MEILISEARCH_URL/indexes/movies/search?q=american%20ninja' \
              -H 'Authorization: Bearer API_KEY'
        - lang: JS
          source: client.index('movies').searchGet('American ninja')
        - lang: Dart
          source: await client.index('movies').search('American ninja');
components:
  schemas:
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
    SearchResult:
      allOf:
<span id="response-all-of-0"></span>        - $ref: '#/components/schemas/HitsInfo'
          description: &gt;-
            Pagination info.


            Either offset/limit with `estimatedTotalHits`, or page/hitsPerPage
            with `totalHits` and `totalPages`.
<span id="response-all-of-1"></span>        - type: object
          required:
            - hits
            - query
            - processingTimeMs
          properties:
<span id="response-all-of-1-hits"></span><span id="response-hits"></span>            hits:
              type: array
<span id="response-all-of-1-hits-items"></span><span id="response-hits-items"></span>              items:
                $ref: '#/components/schemas/SearchHit'
              description: &gt;-
                Matching documents.


                Each hit contains document fields and, when requested,
                `_formatted`, `_matchesPosition`, `_rankingScore`,
                `_rankingScoreDetails`, `_geoDistance`.
<span id="response-all-of-1-query"></span><span id="response-query"></span>            query:
              type: string
              description: Query string that produced this response.
<span id="response-all-of-1-query-vector"></span><span id="response-all-of-1-queryvector"></span><span id="response-query-vector"></span><span id="response-queryvector"></span>            queryVector:
              type:
                - array
                - 'null'
<span id="response-all-of-1-query-vector-items"></span><span id="response-query-vector-items"></span>              items:
                type: number
                format: float
              description: |-
                Query embedding used for the search.

                Present when vector or hybrid search was used.
<span id="response-all-of-1-processing-time-ms"></span><span id="response-all-of-1-processingtimems"></span><span id="response-processing-time-ms"></span><span id="response-processingtimems"></span>            processingTimeMs:
              type: integer
              description: Time taken to process the query, in milliseconds.
              minimum: 0
<span id="response-all-of-1-facet-distribution"></span><span id="response-all-of-1-facetdistribution"></span><span id="response-facet-distribution"></span><span id="response-facetdistribution"></span>            facetDistribution:
              type:
                - object
                - 'null'
              description: &gt;-
                Count of matching documents per facet value for each requested
                facet.


                Present when `facets` was set.
<span id="response-all-of-1-facet-distribution-additional-properties"></span><span id="response-facet-distribution-additional-properties"></span>              additionalProperties: {}
              propertyNames:
                type: string
<span id="response-all-of-1-facet-stats"></span><span id="response-all-of-1-facetstats"></span><span id="response-facet-stats"></span><span id="response-facetstats"></span>            facetStats:
              type:
                - object
                - 'null'
              description: |-
                Minimum and maximum numeric values per facet.

                Present for numeric facets when `facets` was set.
<span id="response-all-of-1-facet-stats-additional-properties"></span><span id="response-facet-stats-additional-properties"></span>              additionalProperties:
                $ref: '#/components/schemas/FacetStats'
              propertyNames:
                type: string
<span id="response-all-of-1-request-uid"></span><span id="response-all-of-1-requestuid"></span><span id="response-request-uid"></span><span id="response-requestuid"></span>            requestUid:
              type:
                - string
                - 'null'
              format: uuid
              description: UUID v7 identifying this search request.
<span id="response-all-of-1-metadata"></span><span id="response-metadata"></span>            metadata:
              oneOf:
<span id="response-all-of-1-metadata-one-of-0"></span><span id="response-metadata-one-of-0"></span>                - type: 'null'
<span id="response-all-of-1-metadata-one-of-1"></span><span id="response-metadata-one-of-1"></span>                - $ref: '#/components/schemas/SearchMetadata'
                  description: &gt;-
                    Query and index metadata.


                    Present when requested via the `Meili-Include-Metadata`
                    header.
<span id="response-all-of-1-performance-details"></span><span id="response-all-of-1-performancedetails"></span><span id="response-performance-details"></span><span id="response-performancedetails"></span>            performanceDetails:
              description: |-
                Timing breakdown per processing step.

                Present when `showPerformanceDetails` was true.
<span id="response-all-of-1-remote-errors"></span><span id="response-all-of-1-remoteerrors"></span><span id="response-remote-errors"></span><span id="response-remoteerrors"></span>            remoteErrors:
              type:
                - object
                - 'null'
              description: Errors from remote shards. Federated search only.
<span id="response-all-of-1-remote-errors-additional-properties"></span><span id="response-remote-errors-additional-properties"></span>              additionalProperties:
                $ref: '#/components/schemas/ResponseError'
              propertyNames:
                type: string
<span id="response-all-of-1-semantic-hit-count"></span><span id="response-all-of-1-semantichitcount"></span><span id="response-semantic-hit-count"></span><span id="response-semantichitcount"></span>            semanticHitCount:
              type:
                - integer
                - 'null'
              format: u-int32
              description: |-
                Exhaustive number of semantic search matches.

                AI-powered (hybrid/semantic) searches only.
              minimum: 0
      description: Search response containing matching documents and metadata.
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
<span id="response-all-of-1-facet-stats-additional-properties-min"></span><span id="response-all-of-1-facet-stats-min"></span><span id="response-facet-stats-additional-properties-min"></span><span id="response-facet-stats-min"></span>        min:
          type: number
          format: double
          description: Lowest numeric value for this facet among matching documents.
<span id="response-all-of-1-facet-stats-additional-properties-max"></span><span id="response-all-of-1-facet-stats-max"></span><span id="response-facet-stats-additional-properties-max"></span><span id="response-facet-stats-max"></span>        max:
          type: number
          format: double
          description: Highest numeric value for this facet among matching documents.
    SearchMetadata:
      type: object
      description: Metadata about a search query (included when requested via header).
      required:
        - queryUid
        - indexUid
      properties:
<span id="response-all-of-1-metadata-one-of-1-query"></span><span id="response-all-of-1-metadata-query"></span><span id="response-metadata-one-of-1-query"></span><span id="response-metadata-query"></span>        query:
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
<span id="response-all-of-1-metadata-one-of-1-query-uid"></span><span id="response-all-of-1-metadata-one-of-1-queryuid"></span><span id="response-all-of-1-metadata-query-uid"></span><span id="response-all-of-1-metadata-queryuid"></span><span id="response-metadata-one-of-1-query-uid"></span><span id="response-metadata-one-of-1-queryuid"></span><span id="response-metadata-query-uid"></span><span id="response-metadata-queryuid"></span>        queryUid:
          type: string
          format: uuid
          description: Unique identifier for the query.
<span id="response-all-of-1-metadata-index-uid"></span><span id="response-all-of-1-metadata-indexuid"></span><span id="response-all-of-1-metadata-one-of-1-index-uid"></span><span id="response-all-of-1-metadata-one-of-1-indexuid"></span><span id="response-metadata-index-uid"></span><span id="response-metadata-indexuid"></span><span id="response-metadata-one-of-1-index-uid"></span><span id="response-metadata-one-of-1-indexuid"></span>        indexUid:
          type: string
          description: UID of the index that was searched.
<span id="response-all-of-1-metadata-one-of-1-primary-key"></span><span id="response-all-of-1-metadata-one-of-1-primarykey"></span><span id="response-all-of-1-metadata-primary-key"></span><span id="response-all-of-1-metadata-primarykey"></span><span id="response-metadata-one-of-1-primary-key"></span><span id="response-metadata-one-of-1-primarykey"></span><span id="response-metadata-primary-key"></span><span id="response-metadata-primarykey"></span>        primaryKey:
          type:
            - string
            - 'null'
          description: &gt;-
            [Primary
            key](https://www.meilisearch.com/docs/learn/getting_started/primary_key)
            of the index.
<span id="response-all-of-1-metadata-one-of-1-remote"></span><span id="response-all-of-1-metadata-remote"></span><span id="response-metadata-one-of-1-remote"></span><span id="response-metadata-remote"></span>        remote:
          type:
            - string
            - 'null'
          description: Remote that processed the query (federated search only).
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes"></span><span id="response-all-of-1-metadata-inlineruleconditionoutcomes"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes"></span><span id="response-all-of-1-metadata-one-of-1-inlineruleconditionoutcomes"></span><span id="response-metadata-inline-rule-condition-outcomes"></span><span id="response-metadata-inlineruleconditionoutcomes"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes"></span><span id="response-metadata-one-of-1-inlineruleconditionoutcomes"></span>        inlineRuleConditionOutcomes:
          oneOf:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-0"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-0"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-0"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-0"></span>            - type: 'null'
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1"></span>            - $ref: '#/components/schemas/ConditionOutcomes'
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
    ConditionOutcomes:
      type: object
      description: Outcome of applying conditions to a query.
      required:
        - satisfiesActiveCondition
      properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-active-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfiesactivecondition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-active-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfiesactivecondition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-active-condition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfiesactivecondition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-active-condition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfiesactivecondition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-active-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfiesactivecondition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-active-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfiesactivecondition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-active-condition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfiesactivecondition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-active-condition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfiesactivecondition"></span>        satisfiesActiveCondition:
          type: boolean
          description: Whether the rule was active
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-time-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfiestimecondition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-time-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfiestimecondition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-time-condition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfiestimecondition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-time-condition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfiestimecondition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-time-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfiestimecondition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-time-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfiestimecondition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-time-condition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfiestimecondition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-time-condition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfiestimecondition"></span>        satisfiesTimeCondition:
          $ref: '#/components/schemas/TimeConditionOutcome'
          description: &gt;-
            Whether the current time falls in the range of time specified by the
            condition
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-empty-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfiesqueryemptycondition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-empty-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfiesqueryemptycondition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-empty-condition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfiesqueryemptycondition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-empty-condition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfiesqueryemptycondition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-empty-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfiesqueryemptycondition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-empty-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfiesqueryemptycondition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-empty-condition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfiesqueryemptycondition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-empty-condition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfiesqueryemptycondition"></span>        satisfiesQueryEmptyCondition:
          $ref: '#/components/schemas/QueryEmptyConditionOutcome'
          description: Whether the query emptiness condition aligns with the actual query
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfiesquerywordscondition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfiesquerywordscondition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfiesquerywordscondition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfiesquerywordscondition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfiesquerywordscondition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfiesquerywordscondition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfiesquerywordscondition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfiesquerywordscondition"></span>        satisfiesQueryWordsCondition:
          $ref: '#/components/schemas/QueryWordsConditionOutcome'
          description: Whether the query words meet the condition
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfiesfiltercondition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfiesfiltercondition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfiesfiltercondition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfiesfiltercondition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfiesfiltercondition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfiesfiltercondition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfiesfiltercondition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfiesfiltercondition"></span>        satisfiesFilterCondition:
          $ref: '#/components/schemas/FilterConditionOutcome'
          description: Whether the filter meets the condition
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
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-0"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-0"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-0"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-0"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-0"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-0"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-0"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-0"></span>        - type: string
          enum:
            - satisfied
            - noConstraint
          description: |-
            - `satisfied`: The query contains all the words of the condition
            - `noConstraint`: No query words condition
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1"></span>        - type: object
          description: At least the specified word is missing in the query
          required:
            - missingWord
          properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missing-word"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missingword"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missing-word"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missingword"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-missing-word"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-missingword"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missing-word"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missingword"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missing-word"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missingword"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missing-word"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missingword"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition-missing-word"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition-missingword"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missing-word"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missingword"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missing-word"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missingword"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missing-word"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missingword"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-missing-word"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-missingword"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missing-word"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missingword"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missing-word"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missingword"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missing-word"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missingword"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition-missing-word"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition-missingword"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missing-word"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missingword"></span>            missingWord:
              type: object
              description: At least the specified word is missing in the query
              required:
                - word
              properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missing-word-word"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missing-word-word"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-missing-word-word"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missing-word-word"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missing-word-word"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missing-word-word"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition-missing-word-word"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missing-word-word"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missing-word-word"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missing-word-word"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-missing-word-word"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missing-word-word"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-missing-word-word"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-query-words-condition-one-of-1-missing-word-word"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition-missing-word-word"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-query-words-condition-one-of-1-missing-word-word"></span>                word:
                  type: string
                  description: Missing word in query
      description: Outcome of the query words condition
    FilterConditionOutcome:
      oneOf:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-0"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-0"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-0"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-0"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-0"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-0"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-0"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-0"></span>        - type: string
          enum:
            - satisfied
            - noConstraint
          description: |-
            - `satisfied`: The filter satifies all constraints in the condition
            - `noConstraint`: No filter condition
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1"></span>        - type: object
          description: At least one field from the condition is missing from the filter
          required:
            - missingConstraintOnField
          properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missing-constraint-on-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missingconstraintonfield"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missing-constraint-on-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missingconstraintonfield"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-missing-constraint-on-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-missingconstraintonfield"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missing-constraint-on-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missingconstraintonfield"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missing-constraint-on-field"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missingconstraintonfield"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missing-constraint-on-field"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missingconstraintonfield"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-missing-constraint-on-field"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-missingconstraintonfield"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missing-constraint-on-field"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missingconstraintonfield"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missing-constraint-on-field"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missingconstraintonfield"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missing-constraint-on-field"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missingconstraintonfield"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-missing-constraint-on-field"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-missingconstraintonfield"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missing-constraint-on-field"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missingconstraintonfield"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missing-constraint-on-field"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missingconstraintonfield"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missing-constraint-on-field"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missingconstraintonfield"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-missing-constraint-on-field"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-missingconstraintonfield"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missing-constraint-on-field"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missingconstraintonfield"></span>            missingConstraintOnField:
              type: object
              description: At least one field from the condition is missing from the filter
              required:
                - field
              properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missing-constraint-on-field-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missing-constraint-on-field-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-missing-constraint-on-field-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missing-constraint-on-field-field"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missing-constraint-on-field-field"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missing-constraint-on-field-field"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-missing-constraint-on-field-field"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missing-constraint-on-field-field"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missing-constraint-on-field-field"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missing-constraint-on-field-field"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-missing-constraint-on-field-field"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missing-constraint-on-field-field"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-missing-constraint-on-field-field"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-1-missing-constraint-on-field-field"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-missing-constraint-on-field-field"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-1-missing-constraint-on-field-field"></span>                field:
                  type: string
                  description: Missing field in filter
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2"></span>        - type: object
          description: At least one field has a condition not met in the filter
          required:
            - unmetConstraintOnField
          properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmet-constraint-on-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmetconstraintonfield"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmet-constraint-on-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmetconstraintonfield"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmet-constraint-on-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmetconstraintonfield"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-unmet-constraint-on-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-unmetconstraintonfield"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmet-constraint-on-field"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmetconstraintonfield"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmet-constraint-on-field"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmetconstraintonfield"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmet-constraint-on-field"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmetconstraintonfield"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-unmet-constraint-on-field"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-unmetconstraintonfield"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmet-constraint-on-field"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmetconstraintonfield"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmet-constraint-on-field"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmetconstraintonfield"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmet-constraint-on-field"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmetconstraintonfield"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-unmet-constraint-on-field"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-unmetconstraintonfield"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmet-constraint-on-field"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmetconstraintonfield"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmet-constraint-on-field"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmetconstraintonfield"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmet-constraint-on-field"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmetconstraintonfield"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-unmet-constraint-on-field"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-unmetconstraintonfield"></span>            unmetConstraintOnField:
              type: object
              description: At least one field has a condition not met in the filter
              required:
                - field
              properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmet-constraint-on-field-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmet-constraint-on-field-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmet-constraint-on-field-field"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-unmet-constraint-on-field-field"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmet-constraint-on-field-field"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmet-constraint-on-field-field"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmet-constraint-on-field-field"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-unmet-constraint-on-field-field"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmet-constraint-on-field-field"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmet-constraint-on-field-field"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmet-constraint-on-field-field"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-unmet-constraint-on-field-field"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-2-unmet-constraint-on-field-field"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-unmet-constraint-on-field-field"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-2-unmet-constraint-on-field-field"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-unmet-constraint-on-field-field"></span>                field:
                  type: string
                  description: Field with unmet constraint in filter
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3"></span>        - type: object
          description: The filter does not constraint enough field to satisfy the condition
          required:
            - notEnoughFields
          properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-notenoughfields"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-notenoughfields"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-notenoughfields"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-notenoughfields"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-notenoughfields"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-notenoughfields"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-notenoughfields"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-notenoughfields"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-notenoughfields"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-notenoughfields"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-notenoughfields"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-notenoughfields"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-notenoughfields"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-notenoughfields"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-notenoughfields"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-notenoughfields"></span>            notEnoughFields:
              type: object
              description: &gt;-
                The filter does not constraint enough field to satisfy the
                condition
              required:
                - fieldCountInFilter
                - fieldCountInCondition
              properties:
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-field-count-in-filter"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-fieldcountinfilter"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-filter"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountinfilter"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-field-count-in-filter"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-fieldcountinfilter"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-filter"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountinfilter"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-field-count-in-filter"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-fieldcountinfilter"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-filter"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountinfilter"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-field-count-in-filter"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-fieldcountinfilter"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-filter"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountinfilter"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-field-count-in-filter"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-fieldcountinfilter"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-filter"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountinfilter"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-field-count-in-filter"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-fieldcountinfilter"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-filter"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountinfilter"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-field-count-in-filter"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-fieldcountinfilter"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-filter"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountinfilter"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-field-count-in-filter"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-fieldcountinfilter"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-filter"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountinfilter"></span>                fieldCountInFilter:
                  type: integer
                  description: Number of fields in the filter
                  minimum: 0
<span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-field-count-in-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-fieldcountincondition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountincondition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-field-count-in-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-fieldcountincondition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-condition"></span><span id="response-all-of-1-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountincondition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-field-count-in-condition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-fieldcountincondition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-condition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountincondition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-field-count-in-condition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-fieldcountincondition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-condition"></span><span id="response-all-of-1-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountincondition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-field-count-in-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-fieldcountincondition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountincondition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-field-count-in-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-fieldcountincondition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-condition"></span><span id="response-metadata-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountincondition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-field-count-in-condition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-not-enough-fields-fieldcountincondition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-condition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-one-of-1-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountincondition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-field-count-in-condition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-not-enough-fields-fieldcountincondition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-field-count-in-condition"></span><span id="response-metadata-one-of-1-inline-rule-condition-outcomes-satisfies-filter-condition-one-of-3-not-enough-fields-fieldcountincondition"></span>                fieldCountInCondition:
                  type: integer
                  description: Number of fields in the condition
                  minimum: 0
      description: Outcome of the filter condition
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