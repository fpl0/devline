# Request a chat completion

> Answer a conversational question with the OpenAI-compatible chat completions API,
using the documents of the authorized indexes as context.
Only streamed responses (`stream: true`) are supported.

This route is only available when the `chatCompletions` [experimental feature](https://www.meilisearch.com/docs/resources/help/experimental_features_overview) is enabled.



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
  /chats/{workspace_uid}/chat/completions:
    post:
      tags:
        - Chats
      summary: Request a chat completion
      description: &gt;-
        Answer a conversational question with the OpenAI-compatible chat
        completions API,

        using the documents of the authorized indexes as context.

        Only streamed responses (`stream: true`) are supported.


        This route is only available when the `chatCompletions` [experimental
        feature](https://www.meilisearch.com/docs/resources/help/experimental_features_overview)
        is enabled.
      operationId: chat
      parameters:
        - name: workspace_uid
<span id="parameter-workspace-uid"></span>          in: path
          description: The unique identifier of the chat workspace.
          required: true
          schema:
            type: string
          example: my-workspace
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateChatCompletionRequest'
        required: true
      responses:
        '200':
          description: Start a conversation.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/CreateChatCompletionResponse'
              example:
                id: chatcmpl-abc123
                choices:
                  - index: 0
                    message:
                      content: &gt;-
                        After searching the Steam database, here are some game
                        recommendations related to your query:


                        1. **Game Dev Tycoon**: This game might interest you, as
                        it involves developing games, which could resonate with
                        your work as a developer working on selling a search
                        engine. It is a casual, strategy, and simulation game
                        where you simulate a game development studio.


                        2. **Mad Games Tycoon**: Another game that could be
                        relevant is Mad Games Tycoon. In this game, you build up
                        your own games, similar to the process of creating and
                        selling software, which could provide insights and
                        inspiration for your work.


                        3. **Airline Tycoon 2**: While not directly related to
                        search engines, Airline Tycoon 2 involves strategic
                        decision-making and business management, which could
                        offer valuable lessons for selling a product like a
                        search engine.


                        These games provide a mix of strategic thinking,
                        simulation, and development aspects that might appeal to
                        you as a developer working on selling a search engine.
                      refusal: null
                      tool_calls: null
                      role: assistant
                      function_call: null
                      audio: null
                    finish_reason: stop
                    logprobs: null
                created: 1747922647
                model: gpt-3.5-turbo-0125
                service_tier: default
                system_fingerprint: null
                object: chat.completion
                usage:
                  prompt_tokens: 1515
                  completion_tokens: 197
                  total_tokens: 1712
                  prompt_tokens_details:
                    audio_tokens: 0
                    cached_tokens: 0
                  completion_tokens_details:
                    accepted_prediction_tokens: 0
                    audio_tokens: 0
                    reasoning_tokens: 0
                    rejected_prediction_tokens: 0
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
          description: Chat not found.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ResponseError'
              example:
                message: Chat :workspaceUid not found.
                code: chat_not_found
                type: invalid_request
                link: https://docs.meilisearch.com/errors#chat_not_found
      security:
        - Bearer:
            - chatsCompletions
            - '*'
      x-codeSamples:
        - lang: cURL
          source: |-
            curl -N \
              -X POST 'MEILISEARCH_URL/chats/WORKSPACE_NAME/chat/completions' \
              -H 'Authorization: Bearer MEILISEARCH_KEY' \
              -H 'Content-Type: application/json' \
              --data-binary '{
                "model": "PROVIDER_MODEL_UID",
                "messages": [
                  {
                    "role": "user",
                    "content": "USER_PROMPT"
                  }
                ],
                "tools": [
                  {
                    "type": "function",
                    "function": {
                      "name": "_meiliSearchProgress",
                      "description": "Reports real-time search progress to the user"
                    }
                  },
                  {
                    "type": "function",
                    "function": {
                      "name": "_meiliSearchSources",
                      "description": "Provides sources and references for the information"
                    }
                  }
                ]
              }'
components:
  schemas:
    CreateChatCompletionRequest:
      type: object
      description: A chat completion request compatible with the OpenAI API.
      required:
        - messages
        - model
      properties:
<span id="body-messages"></span>        messages:
          type: array
<span id="body-messages-items"></span>          items:
            $ref: '#/components/schemas/ChatCompletionRequestMessage'
          description: &gt;-
            A list of messages comprising the conversation so far. Depending on
            the [model](https://platform.openai.com/docs/models) you use,
            different message types (modalities) are supported, like
            [text](https://platform.openai.com/docs/guides/text-generation),
            [images](https://platform.openai.com/docs/guides/vision), and
            [audio](https://platform.openai.com/docs/guides/audio).
<span id="body-model"></span>        model:
          type: string
          description: &gt;-
            ID of the model to use.

            See the [model endpoint
            compatibility](https://platform.openai.com/docs/models#model-endpoint-compatibility)
            table for details on which models work with the Chat API.
<span id="body-store"></span>        store:
          type:
            - boolean
            - 'null'
          description: &gt;-
            Whether or not to store the output of this chat completion request


            for use in our [model
            distillation](https://platform.openai.com/docs/guides/distillation)
            or [evals](https://platform.openai.com/docs/guides/evals) products.
<span id="body-reasoning-effort"></span>        reasoning_effort:
          oneOf:
<span id="body-reasoning-effort-one-of-0"></span>            - type: 'null'
<span id="body-reasoning-effort-one-of-1"></span>            - $ref: '#/components/schemas/ReasoningEffort'
              description: &gt;-
                **o1 models only**


                Constrains effort on reasoning for

                [reasoning
                models](https://platform.openai.com/docs/guides/reasoning).


                Currently supported values are `low`, `medium`, and `high`.
                Reducing


                reasoning effort can result in faster responses and fewer tokens

                used on reasoning in a response.
<span id="body-metadata"></span>        metadata:
          description: &gt;-
            Developer-defined tags and values used for filtering completions in
            the [dashboard](https://platform.openai.com/chat-completions).
<span id="body-frequency-penalty"></span>        frequency_penalty:
          type:
            - number
            - 'null'
          format: float
          description: &gt;-
            Number between -2.0 and 2.0. Positive values penalize new tokens
            based on their existing frequency in the text so far, decreasing the
            model's likelihood to repeat the same line verbatim.
<span id="body-logit-bias"></span>        logit_bias:
          type:
            - object
            - 'null'
          description: &gt;-
            Modify the likelihood of specified tokens appearing in the
            completion.


            Accepts a json object that maps tokens (specified by their token ID
            in the tokenizer) to an associated bias value from -100 to 100.

            Mathematically, the bias is added to the logits generated by the
            model prior to sampling.

            The exact effect will vary per model, but values between -1 and 1
            should decrease or increase likelihood of selection;

            values like -100 or 100 should result in a ban or exclusive
            selection of the relevant token.
<span id="body-logit-bias-additional-properties"></span>          additionalProperties: {}
          propertyNames:
            type: string
<span id="body-logprobs"></span>        logprobs:
          type:
            - boolean
            - 'null'
          description: &gt;-
            Whether to return log probabilities of the output tokens or not. If
            true, returns the log probabilities of each output token returned in
            the `content` of `message`.
<span id="body-top-logprobs"></span>        top_logprobs:
          type:
            - integer
            - 'null'
          format: u-int8
          description: &gt;-
            An integer between 0 and 20 specifying the number of most likely
            tokens to return at each token position, each with an associated log
            probability. `logprobs` must be set to `true` if this parameter is
            used.
          minimum: 0
<span id="body-max-tokens"></span>        max_tokens:
          type:
            - integer
            - 'null'
          format: u-int32
          description: &gt;-
            The maximum number of
            [tokens](https://platform.openai.com/tokenizer) that can be
            generated in the chat completion.


            This value can be used to control
            [costs](https://openai.com/api/pricing/) for text generated via API.

            This value is now deprecated in favor of `max_completion_tokens`,
            and is

            not compatible with [o1 series
            models](https://platform.openai.com/docs/guides/reasoning).
          deprecated: true
          minimum: 0
<span id="body-max-completion-tokens"></span>        max_completion_tokens:
          type:
            - integer
            - 'null'
          format: u-int32
          description: &gt;-
            An upper bound for the number of tokens that can be generated for a
            completion, including visible output tokens and [reasoning
            tokens](https://platform.openai.com/docs/guides/reasoning).
          minimum: 0
<span id="body-n"></span>        'n':
          type:
            - integer
            - 'null'
          format: u-int8
          description: &gt;-
            How many chat completion choices to generate for each input message.
            Note that you will be charged based on the number of generated
            tokens across all of the choices. Keep `n` as `1` to minimize costs.
          minimum: 0
<span id="body-modalities"></span>        modalities:
          type:
            - array
            - 'null'
<span id="body-modalities-items"></span>          items:
            $ref: '#/components/schemas/ChatCompletionModalities'
          description: &gt;-
            Output types that you would like the model to generate for this
            request. Most models are capable of generating text, which is the
            default: `["text"]`.
<span id="body-prediction"></span>        prediction:
          oneOf:
<span id="body-prediction-one-of-0"></span>            - type: 'null'
<span id="body-prediction-one-of-1"></span>            - $ref: '#/components/schemas/PredictionContent'
              description: &gt;-
                Configuration for a [Predicted
                Output](https://platform.openai.com/docs/guides/predicted-outputs),which
                can greatly improve response times when large parts of the model
                response are known ahead of time. This is most common when you
                are regenerating a file with only minor changes to most of the
                content.
<span id="body-audio"></span>        audio:
          oneOf:
<span id="body-audio-one-of-0"></span>            - type: 'null'
<span id="body-audio-one-of-1"></span>            - $ref: '#/components/schemas/ChatCompletionAudio'
              description: &gt;-
                Parameters for audio output. Required when audio output is
                requested with `modalities: ["audio"]`. [Learn
                more](https://platform.openai.com/docs/guides/audio).
<span id="body-presence-penalty"></span>        presence_penalty:
          type:
            - number
            - 'null'
          format: float
          description: &gt;-
            Number between -2.0 and 2.0. Positive values penalize new tokens
            based on whether they appear in the text so far, increasing the
            model's likelihood to talk about new topics.
<span id="body-response-format"></span>        response_format:
          oneOf:
<span id="body-response-format-one-of-0"></span>            - type: 'null'
<span id="body-response-format-one-of-1"></span>            - $ref: '#/components/schemas/ResponseFormat'
              description: &gt;-
                An object specifying the format that the model must output.
                Compatible with
                [GPT-4o](https://platform.openai.com/docs/models/gpt-4o),
                [GPT-4o
                mini](https://platform.openai.com/docs/models/gpt-4o-mini),
                [GPT-4
                Turbo](https://platform.openai.com/docs/models/gpt-4-and-gpt-4-turbo)
                and all GPT-3.5 Turbo models newer than `gpt-3.5-turbo-1106`.


                Setting to `{ "type": "json_schema", "json_schema": {...} }`
                enables Structured Outputs which guarantees the model will match
                your supplied JSON schema. Learn more in the [Structured Outputs
                guide](https://platform.openai.com/docs/guides/structured-outputs).


                Setting to `{ "type": "json_object" }` enables JSON mode, which
                guarantees the message the model generates is valid JSON.


                **Important:** when using JSON mode, you **must** also instruct
                the model to produce JSON yourself via a system or user message.
                Without this, the model may generate an unending stream of
                whitespace until the generation reaches the token limit,
                resulting in a long-running and seemingly "stuck" request. Also
                note that the message content may be partially cut off if
                `finish_reason="length"`, which indicates the generation
                exceeded `max_tokens` or the conversation exceeded the max
                context length.
<span id="body-seed"></span>        seed:
          type:
            - integer
            - 'null'
          format: int64
          description: &gt;2-
             This feature is in Beta.
            If specified, our system will make a best effort to sample
            deterministically, such that repeated requests

            with the same `seed` and parameters should return the same result.

            Determinism is not guaranteed, and you should refer to the
            `system_fingerprint` response parameter to monitor changes in the
            backend.
<span id="body-service-tier"></span>        service_tier:
          oneOf:
<span id="body-service-tier-one-of-0"></span>            - type: 'null'
<span id="body-service-tier-one-of-1"></span>            - $ref: '#/components/schemas/ServiceTier'
              description: &gt;-
                Specifies the latency tier to use for processing the request.
                This parameter is relevant for customers subscribed to the scale
                tier service:

                - If set to 'auto', the system will utilize scale tier credits
                until they are exhausted.

                - If set to 'default', the request will be processed using the
                default service tier with a lower uptime SLA and no latency
                guarentee.

                - When not set, the default behavior is 'auto'.


                When this parameter is set, the response body will include the
                `service_tier` utilized.
<span id="body-stop"></span>        stop:
          oneOf:
<span id="body-stop-one-of-0"></span>            - type: 'null'
<span id="body-stop-one-of-1"></span>            - $ref: '#/components/schemas/Stop'
              description: &gt;-
                Up to 4 sequences where the API will stop generating further
                tokens.
<span id="body-stream"></span>        stream:
          type:
            - boolean
            - 'null'
          description: &gt;-
            If set, partial message deltas will be sent, like in ChatGPT.

            Tokens will be sent as data-only [server-sent
            events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#Event_stream_format)

            as they become available, with the stream terminated by a `data:
            [DONE]` message. [Example Python
            code](https://cookbook.openai.com/examples/how_to_stream_completions).
<span id="body-stream-options"></span>        stream_options:
          oneOf:
<span id="body-stream-options-one-of-0"></span>            - type: 'null'
<span id="body-stream-options-one-of-1"></span>            - $ref: '#/components/schemas/ChatCompletionStreamOptions'
              description: &gt;-
                Options for streaming response. Only set this when you set
                `stream: true`.
<span id="body-temperature"></span>        temperature:
          type:
            - number
            - 'null'
          format: float
          description: &gt;-
            What sampling temperature to use, between 0 and 2. Higher values
            like 0.8 will make the output more random,

            while lower values like 0.2 will make it more focused and
            deterministic.


            We generally recommend altering this or `top_p` but not both.
<span id="body-top-p"></span>        top_p:
          type:
            - number
            - 'null'
          format: float
          description: &gt;-
            An alternative to sampling with temperature, called nucleus
            sampling,

            where the model considers the results of the tokens with top_p
            probability mass.

            So 0.1 means only the tokens comprising the top 10% probability mass
            are considered.

             We generally recommend altering this or `temperature` but not both.
<span id="body-tools"></span>        tools:
          type:
            - array
            - 'null'
<span id="body-tools-items"></span>          items:
            $ref: '#/components/schemas/ChatCompletionTool'
          description: &gt;-
            A list of tools the model may call. Currently, only functions are
            supported as a tool.

            Use this to provide a list of functions the model may generate JSON
            inputs for. A max of 128 functions are supported.
<span id="body-tool-choice"></span>        tool_choice:
          oneOf:
<span id="body-tool-choice-one-of-0"></span>            - type: 'null'
<span id="body-tool-choice-one-of-1"></span>            - $ref: '#/components/schemas/ChatCompletionToolChoiceOption'
              description: &gt;-
                Controls which (if any) tool is called by the model. `none`
                means the model will not call any tool and instead generates a
                message. `auto` means the model can pick between generating a
                message or calling one or more tools. `required` means the model
                must call one or more tools. `none` is the default when no tools
                are present. `auto` is the default if tools are present.
<span id="body-parallel-tool-calls"></span>        parallel_tool_calls:
          type:
            - boolean
            - 'null'
          description: &gt;-
            Whether to enable [parallel function
            calling](https://platform.openai.com/docs/guides/function-calling/parallel-function-calling)
            during tool use.
<span id="body-user"></span>        user:
          type:
            - string
            - 'null'
          description: &gt;-
            A unique identifier representing your end-user, which can help
            OpenAI to monitor and detect abuse. [Learn
            more](https://platform.openai.com/docs/guides/safety-best-practices#end-user-ids).
<span id="body-function-call"></span>        function_call:
          oneOf:
<span id="body-function-call-one-of-0"></span>            - type: 'null'
<span id="body-function-call-one-of-1"></span>            - $ref: '#/components/schemas/ChatCompletionFunctionCall'
              description: &gt;-
                Deprecated in favor of `tool_choice`.


                Controls which (if any) function is called by the model.

                `none` means the model will not call a function and instead
                generates a message.

                `auto` means the model can pick between generating a message or
                calling a function.

                Specifying a particular function via `{"name": "my_function"}`
                forces the model to call that function.


                `none` is the default when no functions are present. `auto` is
                the default if functions are present.
<span id="body-functions"></span>        functions:
          type:
            - array
            - 'null'
<span id="body-functions-items"></span>          items:
            $ref: '#/components/schemas/ChatCompletionFunctions'
          description: |-
            Deprecated in favor of `tools`.

            A list of functions the model may generate JSON inputs for.
          deprecated: true
    CreateChatCompletionResponse:
      type: object
      description: &gt;-
        Represents a chat completion response returned by model, based on the
        provided input.
      required:
        - id
        - choices
        - created
        - model
        - object
      properties:
<span id="response-id"></span>        id:
          type: string
          description: A unique identifier for the chat completion.
<span id="response-choices"></span>        choices:
          type: array
<span id="response-choices-items"></span>          items:
            $ref: '#/components/schemas/ChatChoice'
          description: &gt;-
            A list of chat completion choices. Can be more than one if `n` is
            greater than 1.
<span id="response-created"></span>        created:
          type: integer
          format: u-int32
          description: &gt;-
            The Unix timestamp (in seconds) of when the chat completion was
            created.
          minimum: 0
<span id="response-model"></span>        model:
          type: string
          description: The model used for the chat completion.
<span id="response-service-tier"></span>        service_tier:
          oneOf:
<span id="response-service-tier-one-of-0"></span>            - type: 'null'
<span id="response-service-tier-one-of-1"></span>            - $ref: '#/components/schemas/ServiceTierResponse'
              description: &gt;-
                The service tier used for processing the request. This field is
                only included if the `service_tier` parameter is specified in
                the request.
<span id="response-system-fingerprint"></span>        system_fingerprint:
          type:
            - string
            - 'null'
          description: &gt;-
            This fingerprint represents the backend configuration that the model
            runs with.


            Can be used in conjunction with the `seed` request parameter to
            understand when backend changes have been made that might impact
            determinism.
<span id="response-object"></span>        object:
          type: string
          description: The object type, which is always `chat.completion`.
<span id="response-usage"></span>        usage:
          oneOf:
<span id="response-usage-one-of-0"></span>            - type: 'null'
<span id="response-usage-one-of-1"></span>            - $ref: '#/components/schemas/CompletionUsage'
              description: Usage statistics for the completion request.
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
    ChatCompletionRequestMessage:
      oneOf:
<span id="body-messages-items-one-of-0"></span><span id="body-messages-one-of-0"></span>        - allOf:
<span id="body-messages-all-of-0"></span><span id="body-messages-items-all-of-0"></span><span id="body-messages-items-one-of-0-all-of-0"></span><span id="body-messages-one-of-0-all-of-0"></span>            - $ref: '#/components/schemas/ChatCompletionRequestDeveloperMessage'
<span id="body-messages-all-of-1"></span><span id="body-messages-items-all-of-1"></span><span id="body-messages-items-one-of-0-all-of-1"></span><span id="body-messages-one-of-0-all-of-1"></span>            - type: object
              required:
                - role
              properties:
<span id="body-messages-all-of-1-role"></span><span id="body-messages-items-all-of-1-role"></span><span id="body-messages-items-one-of-0-all-of-1-role"></span><span id="body-messages-items-one-of-0-role"></span><span id="body-messages-items-role"></span><span id="body-messages-one-of-0-all-of-1-role"></span><span id="body-messages-one-of-0-role"></span><span id="body-messages-role"></span>                role:
                  type: string
                  enum:
                    - developer
<span id="body-messages-items-one-of-1"></span><span id="body-messages-one-of-1"></span>        - allOf:
<span id="body-messages-items-one-of-1-all-of-0"></span><span id="body-messages-one-of-1-all-of-0"></span>            - $ref: '#/components/schemas/ChatCompletionRequestSystemMessage'
<span id="body-messages-items-one-of-1-all-of-1"></span><span id="body-messages-one-of-1-all-of-1"></span>            - type: object
              required:
                - role
              properties:
<span id="body-messages-items-one-of-1-all-of-1-role"></span><span id="body-messages-items-one-of-1-role"></span><span id="body-messages-one-of-1-all-of-1-role"></span><span id="body-messages-one-of-1-role"></span>                role:
                  type: string
                  enum:
                    - system
<span id="body-messages-items-one-of-2"></span><span id="body-messages-one-of-2"></span>        - allOf:
<span id="body-messages-items-one-of-2-all-of-0"></span><span id="body-messages-one-of-2-all-of-0"></span>            - $ref: '#/components/schemas/ChatCompletionRequestUserMessage'
<span id="body-messages-items-one-of-2-all-of-1"></span><span id="body-messages-one-of-2-all-of-1"></span>            - type: object
              required:
                - role
              properties:
<span id="body-messages-items-one-of-2-all-of-1-role"></span><span id="body-messages-items-one-of-2-role"></span><span id="body-messages-one-of-2-all-of-1-role"></span><span id="body-messages-one-of-2-role"></span>                role:
                  type: string
                  enum:
                    - user
<span id="body-messages-items-one-of-3"></span><span id="body-messages-one-of-3"></span>        - allOf:
<span id="body-messages-items-one-of-3-all-of-0"></span><span id="body-messages-one-of-3-all-of-0"></span>            - $ref: '#/components/schemas/ChatCompletionRequestAssistantMessage'
<span id="body-messages-items-one-of-3-all-of-1"></span><span id="body-messages-one-of-3-all-of-1"></span>            - type: object
              required:
                - role
              properties:
<span id="body-messages-items-one-of-3-all-of-1-role"></span><span id="body-messages-items-one-of-3-role"></span><span id="body-messages-one-of-3-all-of-1-role"></span><span id="body-messages-one-of-3-role"></span>                role:
                  type: string
                  enum:
                    - assistant
<span id="body-messages-items-one-of-4"></span><span id="body-messages-one-of-4"></span>        - allOf:
<span id="body-messages-items-one-of-4-all-of-0"></span><span id="body-messages-one-of-4-all-of-0"></span>            - $ref: '#/components/schemas/ChatCompletionRequestToolMessage'
<span id="body-messages-items-one-of-4-all-of-1"></span><span id="body-messages-one-of-4-all-of-1"></span>            - type: object
              required:
                - role
              properties:
<span id="body-messages-items-one-of-4-all-of-1-role"></span><span id="body-messages-items-one-of-4-role"></span><span id="body-messages-one-of-4-all-of-1-role"></span><span id="body-messages-one-of-4-role"></span>                role:
                  type: string
                  enum:
                    - tool
<span id="body-messages-items-one-of-5"></span><span id="body-messages-one-of-5"></span>        - allOf:
<span id="body-messages-items-one-of-5-all-of-0"></span><span id="body-messages-one-of-5-all-of-0"></span>            - $ref: '#/components/schemas/ChatCompletionRequestFunctionMessage'
<span id="body-messages-items-one-of-5-all-of-1"></span><span id="body-messages-one-of-5-all-of-1"></span>            - type: object
              required:
                - role
              properties:
<span id="body-messages-items-one-of-5-all-of-1-role"></span><span id="body-messages-items-one-of-5-role"></span><span id="body-messages-one-of-5-all-of-1-role"></span><span id="body-messages-one-of-5-role"></span>                role:
                  type: string
                  enum:
                    - function
      description: &gt;-
        A message in the chat completion conversation. Tagged by `role` (system,
        user, assistant, tool, developer, or function).
    ReasoningEffort:
      type: string
      description: &gt;-
        Constrains effort on reasoning for reasoning models. Supported values
        are `low`, `medium`, and `high`.
      enum:
        - low
        - medium
        - high
    ChatCompletionModalities:
      type: string
      description: &gt;-
        Output types that you would like the model to generate for this request.


        Most models are capable of generating text, which is the default:
        `["text"]`


        The `gpt-4o-audio-preview` model can also be used to [generate

        audio](https://platform.openai.com/docs/guides/audio). To request that
        this model generate both text and audio responses, you can use:
        `["text", "audio"]`
      enum:
        - text
        - audio
    PredictionContent:
      oneOf:
<span id="body-prediction-one-of-1-one-of-0"></span>        - type: object
          description: |-
            The type of the predicted content you want to provide. This type is
            currently always `content`.
          required:
            - content
            - type
          properties:
<span id="body-prediction-content"></span><span id="body-prediction-one-of-0-content"></span><span id="body-prediction-one-of-1-content"></span><span id="body-prediction-one-of-1-one-of-0-content"></span>            content:
              $ref: '#/components/schemas/PredictionContentContent'
              description: &gt;-
                The type of the predicted content you want to provide. This type
                is

                currently always `content`.
<span id="body-prediction-one-of-0-type"></span><span id="body-prediction-one-of-1-one-of-0-type"></span><span id="body-prediction-one-of-1-type"></span><span id="body-prediction-type"></span>            type:
              type: string
              enum:
                - content
      description: &gt;-
        Static predicted output content, such as the content of a text file that
        is being regenerated.
    ChatCompletionAudio:
      type: object
      description: &gt;-
        Parameters for audio output. Required when audio output is requested
        with `modalities: ["audio"]`.
      required:
        - voice
        - format
      properties:
<span id="body-audio-one-of-1-voice"></span><span id="body-audio-voice"></span>        voice:
          $ref: '#/components/schemas/ChatCompletionAudioVoice'
          description: &gt;-
            The voice the model uses to respond. Supported voices are `ash`,
            `ballad`, `coral`, `sage`, and `verse` (also supported but not
            recommended are `alloy`, `echo`, and `shimmer`; these voices are
            less expressive).
<span id="body-audio-format"></span><span id="body-audio-one-of-1-format"></span>        format:
          $ref: '#/components/schemas/ChatCompletionAudioFormat'
          description: &gt;-
            Specifies the output audio format. Must be one of `wav`, `mp3`,
            `flac`, `opus`, or `pcm16`.
    ResponseFormat:
      oneOf:
<span id="body-response-format-one-of-1-one-of-0"></span>        - type: object
          description: 'The type of response format being defined: `text`'
          required:
            - type
          properties:
<span id="body-response-format-one-of-0-type"></span><span id="body-response-format-one-of-1-one-of-0-type"></span><span id="body-response-format-one-of-1-type"></span><span id="body-response-format-type"></span>            type:
              type: string
              enum:
                - text
<span id="body-response-format-one-of-1-one-of-1"></span>        - type: object
          description: 'The type of response format being defined: `json_object`'
          required:
            - type
          properties:
<span id="body-response-format-one-of-1-one-of-1-type"></span>            type:
              type: string
              enum:
                - json_object
<span id="body-response-format-one-of-1-one-of-2"></span><span id="body-response-format-one-of-2"></span>        - type: object
          description: 'The type of response format being defined: `json_schema`'
          required:
            - json_schema
            - type
          properties:
<span id="body-response-format-json-schema"></span><span id="body-response-format-one-of-1-json-schema"></span><span id="body-response-format-one-of-1-one-of-2-json-schema"></span><span id="body-response-format-one-of-2-json-schema"></span>            json_schema:
              $ref: '#/components/schemas/ResponseFormatJsonSchema'
<span id="body-response-format-one-of-1-one-of-2-type"></span><span id="body-response-format-one-of-2-type"></span>            type:
              type: string
              enum:
                - json_schema
      description: &gt;-
        An object specifying the format that the model must output (text, JSON
        object, or JSON schema).
    ServiceTier:
      type: string
      description: Specifies the latency tier to use for processing the request.
      enum:
        - auto
        - default
    Stop:
      oneOf:
<span id="body-stop-one-of-1-one-of-0"></span>        - type: string
<span id="body-stop-one-of-1-one-of-1"></span>        - type: array
<span id="body-stop-items"></span><span id="body-stop-one-of-1-items"></span><span id="body-stop-one-of-1-one-of-1-items"></span>          items:
            type: string
      description: Up to 4 sequences where the API will stop generating further tokens.
    ChatCompletionStreamOptions:
      type: object
      description: &gt;-
        Options for streaming response. Only set this when you set `stream:
        true`.
      required:
        - include_usage
      properties:
<span id="body-stream-options-include-usage"></span><span id="body-stream-options-one-of-1-include-usage"></span>        include_usage:
          type: boolean
          description: &gt;-
            If set, an additional chunk will be streamed before the `data:
            [DONE]` message. The `usage` field on this chunk shows the token
            usage statistics for the entire request, and the `choices` field
            will always be an empty array. All other chunks will also include a
            `usage` field, but with a null value.
    ChatCompletionTool:
      type: object
      description: &gt;-
        A tool the model may call. Currently, only functions are supported as a
        tool.
      required:
        - type
        - function
      properties:
<span id="body-tools-items-type"></span><span id="body-tools-type"></span>        type:
          $ref: '#/components/schemas/ChatCompletionToolType'
<span id="body-tools-function"></span><span id="body-tools-items-function"></span>        function:
          $ref: '#/components/schemas/FunctionObject'
    ChatCompletionToolChoiceOption:
      oneOf:
<span id="body-tool-choice-one-of-1-one-of-0"></span>        - type: string
          enum:
            - none
            - auto
            - required
<span id="body-tool-choice-one-of-1-one-of-1"></span>        - type: object
          required:
            - named
          properties:
<span id="body-tool-choice-named"></span><span id="body-tool-choice-one-of-1-named"></span><span id="body-tool-choice-one-of-1-one-of-1-named"></span>            named:
              $ref: '#/components/schemas/ChatCompletionNamedToolChoice'
      description: &gt;-
        Controls which (if any) tool is called by the model.

        `none` means the model will not call any tool and instead generates a
        message.

        `auto` means the model can pick between generating a message or calling
        one or more tools.

        `required` means the model must call one or more tools.

        Specifying a particular tool via `{"type": "function", "function":
        {"name": "my_function"}}` forces the model to call that tool.


        `none` is the default when no tools are present. `auto` is the default
        if tools are present.present.
    ChatCompletionFunctionCall:
      oneOf:
<span id="body-function-call-one-of-1-one-of-0"></span>        - type: string
          enum:
            - none
            - auto
          description: &gt;-
            - `none`: The model does not call a function, and responds to the
            end-user.

            - `auto`: The model can pick between an end-user or calling a
            function.
<span id="body-function-call-one-of-1-one-of-1"></span>        - type: object
          description: Forces the model to call the specified function.
          required:
            - Function
          properties:
<span id="body-function-call-function"></span><span id="body-function-call-one-of-1-function"></span><span id="body-function-call-one-of-1-one-of-1-function"></span>            Function:
              type: object
              description: Forces the model to call the specified function.
              required:
                - name
              properties:
<span id="body-function-call-function-name"></span><span id="body-function-call-one-of-1-function-name"></span><span id="body-function-call-one-of-1-one-of-1-function-name"></span>                name:
                  type: string
      description: &gt;-
        Deprecated in favor of `tool_choice`. Controls which (if any) function
        is called by the model.
    ChatCompletionFunctions:
      type: object
      description: &gt;-
        Deprecated: A function the model may generate JSON inputs for. Use
        `ChatCompletionTool` instead.
      required:
        - name
        - parameters
      properties:
<span id="body-functions-items-name"></span><span id="body-functions-name"></span>        name:
          type: string
          description: &gt;-
            The name of the function to be called. Must be a-z, A-Z, 0-9, or
            contain underscores and dashes, with a maximum length of 64.
<span id="body-functions-description"></span><span id="body-functions-items-description"></span>        description:
          type:
            - string
            - 'null'
          description: &gt;-
            A description of what the function does, used by the model to choose
            when and how to call the function.
<span id="body-functions-items-parameters"></span><span id="body-functions-parameters"></span>        parameters:
          description: &gt;-
            The parameters the functions accepts, described as a JSON Schema
            object. See the
            [guide](https://platform.openai.com/docs/guides/text-generation/function-calling)
            for examples, and the [JSON Schema
            reference](https://json-schema.org/understanding-json-schema/) for
            documentation about the format.


            Omitting `parameters` defines a function with an empty parameter
            list.
      deprecated: true
    ChatChoice:
      type: object
      description: A chat completion choice generated by the model.
      required:
        - index
        - message
      properties:
<span id="response-choices-index"></span><span id="response-choices-items-index"></span>        index:
          type: integer
          format: u-int32
          description: The index of the choice in the list of choices.
          minimum: 0
<span id="response-choices-items-message"></span><span id="response-choices-message"></span>        message:
          $ref: '#/components/schemas/ChatCompletionResponseMessage'
<span id="response-choices-finish-reason"></span><span id="response-choices-items-finish-reason"></span>        finish_reason:
          oneOf:
<span id="response-choices-finish-reason-one-of-0"></span><span id="response-choices-items-finish-reason-one-of-0"></span>            - type: 'null'
<span id="response-choices-finish-reason-one-of-1"></span><span id="response-choices-items-finish-reason-one-of-1"></span>            - $ref: '#/components/schemas/FinishReason'
              description: &gt;-
                The reason the model stopped generating tokens. This will be
                `stop` if the model hit a natural stop point or a provided stop
                sequence,

                `length` if the maximum number of tokens specified in the
                request was reached,

                `content_filter` if content was omitted due to a flag from our
                content filters,

                `tool_calls` if the model called a tool, or `function_call`
                (deprecated) if the model called a function.
<span id="response-choices-items-logprobs"></span><span id="response-choices-logprobs"></span>        logprobs:
          oneOf:
<span id="response-choices-items-logprobs-one-of-0"></span><span id="response-choices-logprobs-one-of-0"></span>            - type: 'null'
<span id="response-choices-items-logprobs-one-of-1"></span><span id="response-choices-logprobs-one-of-1"></span>            - $ref: '#/components/schemas/ChatChoiceLogprobs'
              description: Log probability information for the choice.
    ServiceTierResponse:
      type: string
      description: &gt;-
        The service tier used for processing the request, as returned in the
        response.
      enum:
        - scale
        - default
        - auto
    CompletionUsage:
      type: object
      description: Usage statistics for the completion request.
      required:
        - prompt_tokens
        - completion_tokens
        - total_tokens
      properties:
<span id="response-usage-one-of-1-prompt-tokens"></span><span id="response-usage-prompt-tokens"></span>        prompt_tokens:
          type: integer
          format: u-int32
          description: Number of tokens in the prompt.
          minimum: 0
<span id="response-usage-completion-tokens"></span><span id="response-usage-one-of-1-completion-tokens"></span>        completion_tokens:
          type: integer
          format: u-int32
          description: Number of tokens in the generated completion.
          minimum: 0
<span id="response-usage-one-of-1-total-tokens"></span><span id="response-usage-total-tokens"></span>        total_tokens:
          type: integer
          format: u-int32
          description: Total number of tokens used in the request (prompt + completion).
          minimum: 0
<span id="response-usage-one-of-1-prompt-tokens-details"></span><span id="response-usage-prompt-tokens-details"></span>        prompt_tokens_details:
          oneOf:
<span id="response-usage-one-of-1-prompt-tokens-details-one-of-0"></span><span id="response-usage-prompt-tokens-details-one-of-0"></span>            - type: 'null'
<span id="response-usage-one-of-1-prompt-tokens-details-one-of-1"></span><span id="response-usage-prompt-tokens-details-one-of-1"></span>            - $ref: '#/components/schemas/PromptTokensDetails'
              description: Breakdown of tokens used in the prompt.
<span id="response-usage-completion-tokens-details"></span><span id="response-usage-one-of-1-completion-tokens-details"></span>        completion_tokens_details:
          oneOf:
<span id="response-usage-completion-tokens-details-one-of-0"></span><span id="response-usage-one-of-1-completion-tokens-details-one-of-0"></span>            - type: 'null'
<span id="response-usage-completion-tokens-details-one-of-1"></span><span id="response-usage-one-of-1-completion-tokens-details-one-of-1"></span>            - $ref: '#/components/schemas/CompletionTokensDetails'
              description: Breakdown of tokens used in a completion.
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
    ChatCompletionRequestDeveloperMessage:
      type: object
      description: A developer message in the conversation.
      required:
        - content
      properties:
<span id="body-messages-all-of-0-content"></span><span id="body-messages-content"></span><span id="body-messages-items-all-of-0-content"></span><span id="body-messages-items-content"></span><span id="body-messages-items-one-of-0-all-of-0-content"></span><span id="body-messages-items-one-of-0-content"></span><span id="body-messages-one-of-0-all-of-0-content"></span><span id="body-messages-one-of-0-content"></span>        content:
          $ref: '#/components/schemas/ChatCompletionRequestDeveloperMessageContent'
          description: The contents of the developer message.
<span id="body-messages-all-of-0-name"></span><span id="body-messages-items-all-of-0-name"></span><span id="body-messages-items-name"></span><span id="body-messages-items-one-of-0-all-of-0-name"></span><span id="body-messages-items-one-of-0-name"></span><span id="body-messages-name"></span><span id="body-messages-one-of-0-all-of-0-name"></span><span id="body-messages-one-of-0-name"></span>        name:
          type:
            - string
            - 'null'
          description: &gt;-
            An optional name for the participant. Provides the model information
            to differentiate between participants of the same role.
    ChatCompletionRequestSystemMessage:
      type: object
      description: A system message setting the behavior of the assistant.
      required:
        - content
      properties:
<span id="body-messages-items-one-of-1-all-of-0-content"></span><span id="body-messages-items-one-of-1-content"></span><span id="body-messages-one-of-1-all-of-0-content"></span><span id="body-messages-one-of-1-content"></span>        content:
          $ref: '#/components/schemas/ChatCompletionRequestSystemMessageContent'
          description: The contents of the system message.
<span id="body-messages-items-one-of-1-all-of-0-name"></span><span id="body-messages-items-one-of-1-name"></span><span id="body-messages-one-of-1-all-of-0-name"></span><span id="body-messages-one-of-1-name"></span>        name:
          type:
            - string
            - 'null'
          description: &gt;-
            An optional name for the participant. Provides the model information
            to differentiate between participants of the same role.
    ChatCompletionRequestUserMessage:
      type: object
      description: A user message in the conversation.
      required:
        - content
      properties:
<span id="body-messages-items-one-of-2-all-of-0-content"></span><span id="body-messages-items-one-of-2-content"></span><span id="body-messages-one-of-2-all-of-0-content"></span><span id="body-messages-one-of-2-content"></span>        content:
          $ref: '#/components/schemas/ChatCompletionRequestUserMessageContent'
          description: The contents of the user message.
<span id="body-messages-items-one-of-2-all-of-0-name"></span><span id="body-messages-items-one-of-2-name"></span><span id="body-messages-one-of-2-all-of-0-name"></span><span id="body-messages-one-of-2-name"></span>        name:
          type:
            - string
            - 'null'
          description: &gt;-
            An optional name for the participant. Provides the model information
            to differentiate between participants of the same role.
    ChatCompletionRequestAssistantMessage:
      type: object
      description: An assistant message in the conversation, possibly including tool calls.
      properties:
<span id="body-messages-items-one-of-3-all-of-0-content"></span><span id="body-messages-items-one-of-3-content"></span><span id="body-messages-one-of-3-all-of-0-content"></span><span id="body-messages-one-of-3-content"></span>        content:
          oneOf:
<span id="body-messages-items-one-of-3-all-of-0-content-one-of-0"></span><span id="body-messages-items-one-of-3-content-one-of-0"></span><span id="body-messages-one-of-3-all-of-0-content-one-of-0"></span><span id="body-messages-one-of-3-content-one-of-0"></span>            - type: 'null'
<span id="body-messages-items-one-of-3-all-of-0-content-one-of-1"></span><span id="body-messages-items-one-of-3-content-one-of-1"></span><span id="body-messages-one-of-3-all-of-0-content-one-of-1"></span><span id="body-messages-one-of-3-content-one-of-1"></span>            - $ref: &gt;-
                #/components/schemas/ChatCompletionRequestAssistantMessageContent
              description: &gt;-
                The contents of the assistant message. Required unless
                `tool_calls` or `function_call` is specified.
<span id="body-messages-all-of-0-refusal"></span><span id="body-messages-items-all-of-0-refusal"></span><span id="body-messages-items-one-of-3-all-of-0-refusal"></span><span id="body-messages-items-one-of-3-refusal"></span><span id="body-messages-items-refusal"></span><span id="body-messages-one-of-3-all-of-0-refusal"></span><span id="body-messages-one-of-3-refusal"></span><span id="body-messages-refusal"></span>        refusal:
          type:
            - string
            - 'null'
          description: The refusal message by the assistant.
<span id="body-messages-items-one-of-3-all-of-0-name"></span><span id="body-messages-items-one-of-3-name"></span><span id="body-messages-one-of-3-all-of-0-name"></span><span id="body-messages-one-of-3-name"></span>        name:
          type:
            - string
            - 'null'
          description: &gt;-
            An optional name for the participant. Provides the model information
            to differentiate between participants of the same role.
<span id="body-messages-all-of-0-audio"></span><span id="body-messages-audio"></span><span id="body-messages-items-all-of-0-audio"></span><span id="body-messages-items-audio"></span><span id="body-messages-items-one-of-3-all-of-0-audio"></span><span id="body-messages-items-one-of-3-audio"></span><span id="body-messages-one-of-3-all-of-0-audio"></span><span id="body-messages-one-of-3-audio"></span>        audio:
          oneOf:
<span id="body-messages-all-of-0-audio-one-of-0"></span><span id="body-messages-audio-one-of-0"></span><span id="body-messages-items-all-of-0-audio-one-of-0"></span><span id="body-messages-items-audio-one-of-0"></span><span id="body-messages-items-one-of-3-all-of-0-audio-one-of-0"></span><span id="body-messages-items-one-of-3-audio-one-of-0"></span><span id="body-messages-one-of-3-all-of-0-audio-one-of-0"></span><span id="body-messages-one-of-3-audio-one-of-0"></span>            - type: 'null'
<span id="body-messages-all-of-0-audio-one-of-1"></span><span id="body-messages-audio-one-of-1"></span><span id="body-messages-items-all-of-0-audio-one-of-1"></span><span id="body-messages-items-audio-one-of-1"></span><span id="body-messages-items-one-of-3-all-of-0-audio-one-of-1"></span><span id="body-messages-items-one-of-3-audio-one-of-1"></span><span id="body-messages-one-of-3-all-of-0-audio-one-of-1"></span><span id="body-messages-one-of-3-audio-one-of-1"></span>            - $ref: '#/components/schemas/ChatCompletionRequestAssistantMessageAudio'
              description: |-
                Data about a previous audio response from the model.
                [Learn more](https://platform.openai.com/docs/guides/audio).
<span id="body-messages-all-of-0-tool-calls"></span><span id="body-messages-items-all-of-0-tool-calls"></span><span id="body-messages-items-one-of-3-all-of-0-tool-calls"></span><span id="body-messages-items-one-of-3-tool-calls"></span><span id="body-messages-items-tool-calls"></span><span id="body-messages-one-of-3-all-of-0-tool-calls"></span><span id="body-messages-one-of-3-tool-calls"></span><span id="body-messages-tool-calls"></span>        tool_calls:
          type:
            - array
            - 'null'
<span id="body-messages-all-of-0-tool-calls-items"></span><span id="body-messages-items-all-of-0-tool-calls-items"></span><span id="body-messages-items-one-of-3-all-of-0-tool-calls-items"></span><span id="body-messages-items-one-of-3-tool-calls-items"></span><span id="body-messages-items-tool-calls-items"></span><span id="body-messages-one-of-3-all-of-0-tool-calls-items"></span><span id="body-messages-one-of-3-tool-calls-items"></span><span id="body-messages-tool-calls-items"></span>          items:
            $ref: '#/components/schemas/ChatCompletionMessageToolCall'
<span id="body-messages-all-of-0-function-call"></span><span id="body-messages-function-call"></span><span id="body-messages-items-all-of-0-function-call"></span><span id="body-messages-items-function-call"></span><span id="body-messages-items-one-of-3-all-of-0-function-call"></span><span id="body-messages-items-one-of-3-function-call"></span><span id="body-messages-one-of-3-all-of-0-function-call"></span><span id="body-messages-one-of-3-function-call"></span>        function_call:
          oneOf:
<span id="body-messages-all-of-0-function-call-one-of-0"></span><span id="body-messages-function-call-one-of-0"></span><span id="body-messages-items-all-of-0-function-call-one-of-0"></span><span id="body-messages-items-function-call-one-of-0"></span><span id="body-messages-items-one-of-3-all-of-0-function-call-one-of-0"></span><span id="body-messages-items-one-of-3-function-call-one-of-0"></span><span id="body-messages-one-of-3-all-of-0-function-call-one-of-0"></span><span id="body-messages-one-of-3-function-call-one-of-0"></span>            - type: 'null'
<span id="body-messages-all-of-0-function-call-one-of-1"></span><span id="body-messages-function-call-one-of-1"></span><span id="body-messages-items-all-of-0-function-call-one-of-1"></span><span id="body-messages-items-function-call-one-of-1"></span><span id="body-messages-items-one-of-3-all-of-0-function-call-one-of-1"></span><span id="body-messages-items-one-of-3-function-call-one-of-1"></span><span id="body-messages-one-of-3-all-of-0-function-call-one-of-1"></span><span id="body-messages-one-of-3-function-call-one-of-1"></span>            - $ref: '#/components/schemas/FunctionCall'
              description: &gt;-
                Deprecated and replaced by `tool_calls`. The name and arguments
                of a function that should be called, as generated by the model.
    ChatCompletionRequestToolMessage:
      type: object
      description: Tool message
      required:
        - content
        - tool_call_id
      properties:
<span id="body-messages-items-one-of-4-all-of-0-content"></span><span id="body-messages-items-one-of-4-content"></span><span id="body-messages-one-of-4-all-of-0-content"></span><span id="body-messages-one-of-4-content"></span>        content:
          $ref: '#/components/schemas/ChatCompletionRequestToolMessageContent'
          description: The contents of the tool message.
<span id="body-messages-all-of-0-tool-call-id"></span><span id="body-messages-items-all-of-0-tool-call-id"></span><span id="body-messages-items-one-of-4-all-of-0-tool-call-id"></span><span id="body-messages-items-one-of-4-tool-call-id"></span><span id="body-messages-items-tool-call-id"></span><span id="body-messages-one-of-4-all-of-0-tool-call-id"></span><span id="body-messages-one-of-4-tool-call-id"></span><span id="body-messages-tool-call-id"></span>        tool_call_id:
          type: string
    ChatCompletionRequestFunctionMessage:
      type: object
      description: 'Deprecated: A function response message. Use tool messages instead.'
      required:
        - name
      properties:
<span id="body-messages-items-one-of-5-all-of-0-content"></span><span id="body-messages-items-one-of-5-content"></span><span id="body-messages-one-of-5-all-of-0-content"></span><span id="body-messages-one-of-5-content"></span>        content:
          type:
            - string
            - 'null'
          description: The return value from the function call, to return to the model.
<span id="body-messages-items-one-of-5-all-of-0-name"></span><span id="body-messages-items-one-of-5-name"></span><span id="body-messages-one-of-5-all-of-0-name"></span><span id="body-messages-one-of-5-name"></span>        name:
          type: string
          description: The name of the function to call.
    PredictionContentContent:
      oneOf:
<span id="body-prediction-content-one-of-0"></span><span id="body-prediction-one-of-0-content-one-of-0"></span><span id="body-prediction-one-of-1-content-one-of-0"></span><span id="body-prediction-one-of-1-one-of-0-content-one-of-0"></span>        - type: string
          description: &gt;-
            The content used for a Predicted Output. This is often the text of a
            file you are regenerating with minor changes.
<span id="body-prediction-content-one-of-1"></span><span id="body-prediction-one-of-0-content-one-of-1"></span><span id="body-prediction-one-of-1-content-one-of-1"></span><span id="body-prediction-one-of-1-one-of-0-content-one-of-1"></span>        - type: array
<span id="body-prediction-content-items"></span><span id="body-prediction-content-one-of-1-items"></span><span id="body-prediction-one-of-0-content-items"></span><span id="body-prediction-one-of-0-content-one-of-1-items"></span><span id="body-prediction-one-of-1-content-items"></span><span id="body-prediction-one-of-1-content-one-of-1-items"></span><span id="body-prediction-one-of-1-one-of-0-content-items"></span><span id="body-prediction-one-of-1-one-of-0-content-one-of-1-items"></span>          items:
            $ref: '#/components/schemas/ChatCompletionRequestMessageContentPartText'
          description: &gt;-
            An array of content parts with a defined type. Supported options
            differ based on the [model](https://platform.openai.com/docs/models)
            being used to generate the response. Can contain text inputs.
      description: &gt;-
        The content that should be matched when generating a model response. If
        generated tokens would match this content, the entire model response can
        be returned much more quickly.
    ChatCompletionAudioVoice:
      type: string
      description: The voice the model uses to respond in audio output.
      enum:
        - alloy
        - ash
        - ballad
        - coral
        - echo
        - sage
        - shimmer
        - verse
    ChatCompletionAudioFormat:
      type: string
      description: &gt;-
        Specifies the output audio format. Must be one of `wav`, `mp3`, `flac`,
        `opus`, or `pcm16`.
      enum:
        - wav
        - mp3
        - flac
        - opus
        - pcm16
    ResponseFormatJsonSchema:
      type: object
      description: &gt;-
        A JSON Schema-based response format specification for Structured
        Outputs.
      required:
        - name
      properties:
<span id="body-response-format-json-schema-description"></span><span id="body-response-format-one-of-1-json-schema-description"></span><span id="body-response-format-one-of-1-one-of-2-json-schema-description"></span><span id="body-response-format-one-of-2-json-schema-description"></span>        description:
          type:
            - string
            - 'null'
          description: &gt;-
            A description of what the response format is for, used by the model
            to determine how to respond in the format.
<span id="body-response-format-json-schema-name"></span><span id="body-response-format-one-of-1-json-schema-name"></span><span id="body-response-format-one-of-1-one-of-2-json-schema-name"></span><span id="body-response-format-one-of-2-json-schema-name"></span>        name:
          type: string
          description: &gt;-
            The name of the response format. Must be a-z, A-Z, 0-9, or contain
            underscores and dashes, with a maximum length of 64.
<span id="body-response-format-json-schema-schema"></span><span id="body-response-format-one-of-1-json-schema-schema"></span><span id="body-response-format-one-of-1-one-of-2-json-schema-schema"></span><span id="body-response-format-one-of-2-json-schema-schema"></span>        schema:
          description: &gt;-
            The schema for the response format, described as a JSON Schema
            object.
<span id="body-response-format-json-schema-strict"></span><span id="body-response-format-one-of-1-json-schema-strict"></span><span id="body-response-format-one-of-1-one-of-2-json-schema-strict"></span><span id="body-response-format-one-of-2-json-schema-strict"></span>        strict:
          type:
            - boolean
            - 'null'
          description: &gt;-
            Whether to enable strict schema adherence when generating the
            output. If set to true, the model will always follow the exact
            schema defined in the `schema` field. Only a subset of JSON Schema
            is supported when `strict` is `true`. To learn more, read the
            [Structured Outputs
            guide](https://platform.openai.com/docs/guides/structured-outputs).
    ChatCompletionToolType:
      type: string
      description: The type of a tool. Currently, only `function` is supported.
      enum:
        - function
    FunctionObject:
      type: object
      description: A function definition for tool use, described as a JSON Schema object.
      required:
        - name
      properties:
<span id="body-tools-function-name"></span><span id="body-tools-items-function-name"></span>        name:
          type: string
          description: &gt;-
            The name of the function to be called. Must be a-z, A-Z, 0-9, or
            contain underscores and dashes, with a maximum length of 64.
<span id="body-tools-function-description"></span><span id="body-tools-items-function-description"></span>        description:
          type:
            - string
            - 'null'
          description: &gt;-
            A description of what the function does, used by the model to choose
            when and how to call the function.
<span id="body-tools-function-parameters"></span><span id="body-tools-items-function-parameters"></span>        parameters:
          description: &gt;-
            The parameters the functions accepts, described as a JSON Schema
            object. See the
            [guide](https://platform.openai.com/docs/guides/text-generation/function-calling)
            for examples, and the [JSON Schema
            reference](https://json-schema.org/understanding-json-schema/) for
            documentation about the format.


            Omitting `parameters` defines a function with an empty parameter
            list.
<span id="body-tools-function-strict"></span><span id="body-tools-items-function-strict"></span>        strict:
          type:
            - boolean
            - 'null'
          description: &gt;-
            Whether to enable strict schema adherence when generating the
            function call. If set to true, the model will follow the exact
            schema defined in the `parameters` field. Only a subset of JSON
            Schema is supported when `strict` is `true`. Learn more about
            Structured Outputs in the [function calling
            guide](https://platform.openai.com/docs/guides/function-calling).
    ChatCompletionNamedToolChoice:
      type: object
      description: &gt;-
        Specifies a tool the model should use. Use to force the model to call a
        specific function.
      required:
        - type
        - function
      properties:
<span id="body-tool-choice-named-type"></span><span id="body-tool-choice-one-of-1-named-type"></span><span id="body-tool-choice-one-of-1-one-of-1-named-type"></span>        type:
          $ref: '#/components/schemas/ChatCompletionToolType'
          description: The type of the tool. Currently, only `function` is supported.
<span id="body-tool-choice-named-function"></span><span id="body-tool-choice-one-of-1-named-function"></span><span id="body-tool-choice-one-of-1-one-of-1-named-function"></span>        function:
          $ref: '#/components/schemas/FunctionName'
    ChatCompletionResponseMessage:
      type: object
      description: A chat completion message generated by the model.
      required:
        - role
      properties:
<span id="response-choices-items-message-content"></span><span id="response-choices-message-content"></span>        content:
          type:
            - string
            - 'null'
          description: The contents of the message.
<span id="response-choices-items-message-refusal"></span><span id="response-choices-message-refusal"></span>        refusal:
          type:
            - string
            - 'null'
          description: The refusal message generated by the model.
<span id="response-choices-items-message-tool-calls"></span><span id="response-choices-message-tool-calls"></span>        tool_calls:
          type:
            - array
            - 'null'
<span id="response-choices-items-message-tool-calls-items"></span><span id="response-choices-message-tool-calls-items"></span>          items:
            $ref: '#/components/schemas/ChatCompletionMessageToolCall'
          description: The tool calls generated by the model, such as function calls.
<span id="response-choices-items-message-role"></span><span id="response-choices-message-role"></span>        role:
          $ref: '#/components/schemas/Role'
          description: The role of the author of this message.
<span id="response-choices-items-message-function-call"></span><span id="response-choices-message-function-call"></span>        function_call:
          oneOf:
<span id="response-choices-items-message-function-call-one-of-0"></span><span id="response-choices-message-function-call-one-of-0"></span>            - type: 'null'
<span id="response-choices-items-message-function-call-one-of-1"></span><span id="response-choices-message-function-call-one-of-1"></span>            - $ref: '#/components/schemas/FunctionCall'
              description: &gt;-
                Deprecated and replaced by `tool_calls`.

                The name and arguments of a function that should be called, as
                generated by the model.
<span id="response-choices-items-message-audio"></span><span id="response-choices-message-audio"></span>        audio:
          oneOf:
<span id="response-choices-items-message-audio-one-of-0"></span><span id="response-choices-message-audio-one-of-0"></span>            - type: 'null'
<span id="response-choices-items-message-audio-one-of-1"></span><span id="response-choices-message-audio-one-of-1"></span>            - $ref: '#/components/schemas/ChatCompletionResponseMessageAudio'
              description: &gt;-
                If the audio output modality is requested, this object contains
                data about the audio response from the model. [Learn
                more](https://platform.openai.com/docs/guides/audio).
    FinishReason:
      type: string
      description: The reason the model stopped generating tokens.
      enum:
        - stop
        - length
        - tool_calls
        - content_filter
        - function_call
    ChatChoiceLogprobs:
      type: object
      description: Log probability information for a chat completion choice.
      properties:
<span id="response-choices-items-logprobs-content"></span><span id="response-choices-items-logprobs-one-of-1-content"></span><span id="response-choices-logprobs-content"></span><span id="response-choices-logprobs-one-of-1-content"></span>        content:
          type:
            - array
            - 'null'
<span id="response-choices-items-logprobs-content-items"></span><span id="response-choices-items-logprobs-one-of-1-content-items"></span><span id="response-choices-logprobs-content-items"></span><span id="response-choices-logprobs-one-of-1-content-items"></span>          items:
            $ref: '#/components/schemas/ChatCompletionTokenLogprob'
          description: A list of message content tokens with log probability information.
<span id="response-choices-items-logprobs-one-of-1-refusal"></span><span id="response-choices-items-logprobs-refusal"></span><span id="response-choices-logprobs-one-of-1-refusal"></span><span id="response-choices-logprobs-refusal"></span>        refusal:
          type:
            - array
            - 'null'
<span id="response-choices-items-logprobs-one-of-1-refusal-items"></span><span id="response-choices-items-logprobs-refusal-items"></span><span id="response-choices-logprobs-one-of-1-refusal-items"></span><span id="response-choices-logprobs-refusal-items"></span>          items:
            $ref: '#/components/schemas/ChatCompletionTokenLogprob'
    PromptTokensDetails:
      type: object
      description: Breakdown of tokens used in a completion.
      properties:
<span id="response-usage-one-of-1-prompt-tokens-details-audio-tokens"></span><span id="response-usage-one-of-1-prompt-tokens-details-one-of-1-audio-tokens"></span><span id="response-usage-prompt-tokens-details-audio-tokens"></span><span id="response-usage-prompt-tokens-details-one-of-1-audio-tokens"></span>        audio_tokens:
          type:
            - integer
            - 'null'
          format: u-int32
          description: Audio input tokens present in the prompt.
          minimum: 0
<span id="response-usage-one-of-1-prompt-tokens-details-cached-tokens"></span><span id="response-usage-one-of-1-prompt-tokens-details-one-of-1-cached-tokens"></span><span id="response-usage-prompt-tokens-details-cached-tokens"></span><span id="response-usage-prompt-tokens-details-one-of-1-cached-tokens"></span>        cached_tokens:
          type:
            - integer
            - 'null'
          format: u-int32
          description: Cached tokens present in the prompt.
          minimum: 0
    CompletionTokensDetails:
      type: object
      description: Breakdown of tokens used in a completion.
      properties:
<span id="response-usage-completion-tokens-details-accepted-prediction-tokens"></span><span id="response-usage-completion-tokens-details-one-of-1-accepted-prediction-tokens"></span><span id="response-usage-one-of-1-completion-tokens-details-accepted-prediction-tokens"></span><span id="response-usage-one-of-1-completion-tokens-details-one-of-1-accepted-prediction-tokens"></span>        accepted_prediction_tokens:
          type:
            - integer
            - 'null'
          format: u-int32
          minimum: 0
<span id="response-usage-completion-tokens-details-audio-tokens"></span><span id="response-usage-completion-tokens-details-one-of-1-audio-tokens"></span><span id="response-usage-one-of-1-completion-tokens-details-audio-tokens"></span><span id="response-usage-one-of-1-completion-tokens-details-one-of-1-audio-tokens"></span>        audio_tokens:
          type:
            - integer
            - 'null'
          format: u-int32
          description: Audio input tokens generated by the model.
          minimum: 0
<span id="response-usage-completion-tokens-details-one-of-1-reasoning-tokens"></span><span id="response-usage-completion-tokens-details-reasoning-tokens"></span><span id="response-usage-one-of-1-completion-tokens-details-one-of-1-reasoning-tokens"></span><span id="response-usage-one-of-1-completion-tokens-details-reasoning-tokens"></span>        reasoning_tokens:
          type:
            - integer
            - 'null'
          format: u-int32
          description: Tokens generated by the model for reasoning.
          minimum: 0
<span id="response-usage-completion-tokens-details-one-of-1-rejected-prediction-tokens"></span><span id="response-usage-completion-tokens-details-rejected-prediction-tokens"></span><span id="response-usage-one-of-1-completion-tokens-details-one-of-1-rejected-prediction-tokens"></span><span id="response-usage-one-of-1-completion-tokens-details-rejected-prediction-tokens"></span>        rejected_prediction_tokens:
          type:
            - integer
            - 'null'
          format: u-int32
          description: |2-
             When using Predicted Outputs, the number of tokens in the
            prediction that did not appear in the completion. However, like
            reasoning tokens, these tokens are still counted in the total
            completion tokens for purposes of billing, output, and context
            window limits.
          minimum: 0
    ChatCompletionRequestDeveloperMessageContent:
      oneOf:
<span id="body-messages-all-of-0-content-one-of-0"></span><span id="body-messages-content-one-of-0"></span><span id="body-messages-items-all-of-0-content-one-of-0"></span><span id="body-messages-items-content-one-of-0"></span><span id="body-messages-items-one-of-0-all-of-0-content-one-of-0"></span><span id="body-messages-items-one-of-0-content-one-of-0"></span><span id="body-messages-one-of-0-all-of-0-content-one-of-0"></span><span id="body-messages-one-of-0-content-one-of-0"></span>        - type: string
<span id="body-messages-all-of-0-content-one-of-1"></span><span id="body-messages-content-one-of-1"></span><span id="body-messages-items-all-of-0-content-one-of-1"></span><span id="body-messages-items-content-one-of-1"></span><span id="body-messages-items-one-of-0-all-of-0-content-one-of-1"></span><span id="body-messages-items-one-of-0-content-one-of-1"></span><span id="body-messages-one-of-0-all-of-0-content-one-of-1"></span><span id="body-messages-one-of-0-content-one-of-1"></span>        - type: array
<span id="body-messages-all-of-0-content-items"></span><span id="body-messages-all-of-0-content-one-of-1-items"></span><span id="body-messages-content-items"></span><span id="body-messages-content-one-of-1-items"></span><span id="body-messages-items-all-of-0-content-items"></span><span id="body-messages-items-all-of-0-content-one-of-1-items"></span><span id="body-messages-items-content-items"></span><span id="body-messages-items-content-one-of-1-items"></span><span id="body-messages-items-one-of-0-all-of-0-content-items"></span><span id="body-messages-items-one-of-0-all-of-0-content-one-of-1-items"></span><span id="body-messages-items-one-of-0-content-items"></span><span id="body-messages-items-one-of-0-content-one-of-1-items"></span><span id="body-messages-one-of-0-all-of-0-content-items"></span><span id="body-messages-one-of-0-all-of-0-content-one-of-1-items"></span><span id="body-messages-one-of-0-content-items"></span><span id="body-messages-one-of-0-content-one-of-1-items"></span>          items:
            $ref: '#/components/schemas/ChatCompletionRequestMessageContentPartText'
      description: &gt;-
        The contents of a developer message: either a text string or an array of
        text content parts.
    ChatCompletionRequestSystemMessageContent:
      oneOf:
<span id="body-messages-items-one-of-1-all-of-0-content-one-of-0"></span><span id="body-messages-items-one-of-1-content-one-of-0"></span><span id="body-messages-one-of-1-all-of-0-content-one-of-0"></span><span id="body-messages-one-of-1-content-one-of-0"></span>        - type: string
          description: The text contents of the system message.
<span id="body-messages-items-one-of-1-all-of-0-content-one-of-1"></span><span id="body-messages-items-one-of-1-content-one-of-1"></span><span id="body-messages-one-of-1-all-of-0-content-one-of-1"></span><span id="body-messages-one-of-1-content-one-of-1"></span>        - type: array
<span id="body-messages-items-one-of-1-all-of-0-content-items"></span><span id="body-messages-items-one-of-1-all-of-0-content-one-of-1-items"></span><span id="body-messages-items-one-of-1-content-items"></span><span id="body-messages-items-one-of-1-content-one-of-1-items"></span><span id="body-messages-one-of-1-all-of-0-content-items"></span><span id="body-messages-one-of-1-all-of-0-content-one-of-1-items"></span><span id="body-messages-one-of-1-content-items"></span><span id="body-messages-one-of-1-content-one-of-1-items"></span>          items:
            $ref: '#/components/schemas/ChatCompletionRequestSystemMessageContentPart'
          description: &gt;-
            An array of content parts with a defined type. For system messages,
            only type `text` is supported.
      description: &gt;-
        The contents of a system message: either a text string or an array of
        text content parts.
    ChatCompletionRequestUserMessageContent:
      oneOf:
<span id="body-messages-items-one-of-2-all-of-0-content-one-of-0"></span><span id="body-messages-items-one-of-2-content-one-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-0"></span><span id="body-messages-one-of-2-content-one-of-0"></span>        - type: string
          description: The text contents of the message.
<span id="body-messages-items-one-of-2-all-of-0-content-one-of-1"></span><span id="body-messages-items-one-of-2-content-one-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1"></span><span id="body-messages-one-of-2-content-one-of-1"></span>        - type: array
<span id="body-messages-items-one-of-2-all-of-0-content-items"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-items"></span><span id="body-messages-items-one-of-2-content-items"></span><span id="body-messages-items-one-of-2-content-one-of-1-items"></span><span id="body-messages-one-of-2-all-of-0-content-items"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-items"></span><span id="body-messages-one-of-2-content-items"></span><span id="body-messages-one-of-2-content-one-of-1-items"></span>          items:
            $ref: '#/components/schemas/ChatCompletionRequestUserMessageContentPart'
          description: &gt;-
            An array of content parts with a defined type. Supported options
            differ based on the [model](https://platform.openai.com/docs/models)
            being used to generate the response. Can contain text, image, or
            audio inputs.
      description: &gt;-
        The contents of a user message: either a text string or an array of
        content parts (text, image, or audio).
    ChatCompletionRequestAssistantMessageContent:
      oneOf:
<span id="body-messages-items-one-of-3-all-of-0-content-one-of-1-one-of-0"></span><span id="body-messages-items-one-of-3-content-one-of-1-one-of-0"></span><span id="body-messages-one-of-3-all-of-0-content-one-of-1-one-of-0"></span><span id="body-messages-one-of-3-content-one-of-1-one-of-0"></span>        - type: string
          description: The text contents of the message.
<span id="body-messages-items-one-of-3-all-of-0-content-one-of-1-one-of-1"></span><span id="body-messages-items-one-of-3-content-one-of-1-one-of-1"></span><span id="body-messages-one-of-3-all-of-0-content-one-of-1-one-of-1"></span><span id="body-messages-one-of-3-content-one-of-1-one-of-1"></span>        - type: array
<span id="body-messages-all-of-0-content-one-of-1-one-of-1-items"></span><span id="body-messages-content-one-of-1-one-of-1-items"></span><span id="body-messages-items-all-of-0-content-one-of-1-one-of-1-items"></span><span id="body-messages-items-content-one-of-1-one-of-1-items"></span><span id="body-messages-items-one-of-3-all-of-0-content-items"></span><span id="body-messages-items-one-of-3-all-of-0-content-one-of-1-items"></span><span id="body-messages-items-one-of-3-all-of-0-content-one-of-1-one-of-1-items"></span><span id="body-messages-items-one-of-3-content-items"></span><span id="body-messages-items-one-of-3-content-one-of-1-items"></span><span id="body-messages-items-one-of-3-content-one-of-1-one-of-1-items"></span><span id="body-messages-one-of-3-all-of-0-content-items"></span><span id="body-messages-one-of-3-all-of-0-content-one-of-1-items"></span><span id="body-messages-one-of-3-all-of-0-content-one-of-1-one-of-1-items"></span><span id="body-messages-one-of-3-content-items"></span><span id="body-messages-one-of-3-content-one-of-1-items"></span><span id="body-messages-one-of-3-content-one-of-1-one-of-1-items"></span>          items:
            $ref: &gt;-
              #/components/schemas/ChatCompletionRequestAssistantMessageContentPart
          description: &gt;-
            An array of content parts with a defined type. Can be one or more of
            type `text`, or exactly one of type `refusal`.
      description: &gt;-
        The contents of an assistant message: either a text string or an array
        of content parts (text or refusal).
    ChatCompletionRequestAssistantMessageAudio:
      type: object
      description: &gt;-
        Data about a previous audio response from the model, used in multi-turn
        conversations.
      required:
        - id
      properties:
<span id="body-messages-all-of-0-audio-id"></span><span id="body-messages-all-of-0-audio-one-of-1-id"></span><span id="body-messages-audio-id"></span><span id="body-messages-audio-one-of-1-id"></span><span id="body-messages-items-all-of-0-audio-id"></span><span id="body-messages-items-all-of-0-audio-one-of-1-id"></span><span id="body-messages-items-audio-id"></span><span id="body-messages-items-audio-one-of-1-id"></span><span id="body-messages-items-one-of-3-all-of-0-audio-id"></span><span id="body-messages-items-one-of-3-all-of-0-audio-one-of-1-id"></span><span id="body-messages-items-one-of-3-audio-id"></span><span id="body-messages-items-one-of-3-audio-one-of-1-id"></span><span id="body-messages-one-of-3-all-of-0-audio-id"></span><span id="body-messages-one-of-3-all-of-0-audio-one-of-1-id"></span><span id="body-messages-one-of-3-audio-id"></span><span id="body-messages-one-of-3-audio-one-of-1-id"></span>        id:
          type: string
          description: Unique identifier for a previous audio response from the model.
    ChatCompletionMessageToolCall:
      type: object
      description: A tool call generated by the model.
      required:
        - id
        - function
      properties:
<span id="body-messages-all-of-0-tool-calls-id"></span><span id="body-messages-all-of-0-tool-calls-items-id"></span><span id="body-messages-items-all-of-0-tool-calls-id"></span><span id="body-messages-items-all-of-0-tool-calls-items-id"></span><span id="body-messages-items-one-of-3-all-of-0-tool-calls-id"></span><span id="body-messages-items-one-of-3-all-of-0-tool-calls-items-id"></span><span id="body-messages-items-one-of-3-tool-calls-id"></span><span id="body-messages-items-one-of-3-tool-calls-items-id"></span><span id="body-messages-items-tool-calls-id"></span><span id="body-messages-items-tool-calls-items-id"></span><span id="body-messages-one-of-3-all-of-0-tool-calls-id"></span><span id="body-messages-one-of-3-all-of-0-tool-calls-items-id"></span><span id="body-messages-one-of-3-tool-calls-id"></span><span id="body-messages-one-of-3-tool-calls-items-id"></span><span id="body-messages-tool-calls-id"></span><span id="body-messages-tool-calls-items-id"></span><span id="response-choices-items-message-tool-calls-id"></span><span id="response-choices-items-message-tool-calls-items-id"></span><span id="response-choices-message-tool-calls-id"></span><span id="response-choices-message-tool-calls-items-id"></span>        id:
          type: string
          description: The ID of the tool call.
<span id="body-messages-all-of-0-tool-calls-items-type"></span><span id="body-messages-all-of-0-tool-calls-type"></span><span id="body-messages-items-all-of-0-tool-calls-items-type"></span><span id="body-messages-items-all-of-0-tool-calls-type"></span><span id="body-messages-items-one-of-3-all-of-0-tool-calls-items-type"></span><span id="body-messages-items-one-of-3-all-of-0-tool-calls-type"></span><span id="body-messages-items-one-of-3-tool-calls-items-type"></span><span id="body-messages-items-one-of-3-tool-calls-type"></span><span id="body-messages-items-tool-calls-items-type"></span><span id="body-messages-items-tool-calls-type"></span><span id="body-messages-one-of-3-all-of-0-tool-calls-items-type"></span><span id="body-messages-one-of-3-all-of-0-tool-calls-type"></span><span id="body-messages-one-of-3-tool-calls-items-type"></span><span id="body-messages-one-of-3-tool-calls-type"></span><span id="body-messages-tool-calls-items-type"></span><span id="body-messages-tool-calls-type"></span><span id="response-choices-items-message-tool-calls-items-type"></span><span id="response-choices-items-message-tool-calls-type"></span><span id="response-choices-message-tool-calls-items-type"></span><span id="response-choices-message-tool-calls-type"></span>        type:
          oneOf:
<span id="body-messages-all-of-0-tool-calls-items-type-one-of-0"></span><span id="body-messages-all-of-0-tool-calls-type-one-of-0"></span><span id="body-messages-items-all-of-0-tool-calls-items-type-one-of-0"></span><span id="body-messages-items-all-of-0-tool-calls-type-one-of-0"></span><span id="body-messages-items-one-of-3-all-of-0-tool-calls-items-type-one-of-0"></span><span id="body-messages-items-one-of-3-all-of-0-tool-calls-type-one-of-0"></span><span id="body-messages-items-one-of-3-tool-calls-items-type-one-of-0"></span><span id="body-messages-items-one-of-3-tool-calls-type-one-of-0"></span><span id="body-messages-items-tool-calls-items-type-one-of-0"></span><span id="body-messages-items-tool-calls-type-one-of-0"></span><span id="body-messages-one-of-3-all-of-0-tool-calls-items-type-one-of-0"></span><span id="body-messages-one-of-3-all-of-0-tool-calls-type-one-of-0"></span><span id="body-messages-one-of-3-tool-calls-items-type-one-of-0"></span><span id="body-messages-one-of-3-tool-calls-type-one-of-0"></span><span id="body-messages-tool-calls-items-type-one-of-0"></span><span id="body-messages-tool-calls-type-one-of-0"></span><span id="response-choices-items-message-tool-calls-items-type-one-of-0"></span><span id="response-choices-items-message-tool-calls-type-one-of-0"></span><span id="response-choices-message-tool-calls-items-type-one-of-0"></span><span id="response-choices-message-tool-calls-type-one-of-0"></span>            - type: 'null'
<span id="body-messages-all-of-0-tool-calls-items-type-one-of-1"></span><span id="body-messages-all-of-0-tool-calls-type-one-of-1"></span><span id="body-messages-items-all-of-0-tool-calls-items-type-one-of-1"></span><span id="body-messages-items-all-of-0-tool-calls-type-one-of-1"></span><span id="body-messages-items-one-of-3-all-of-0-tool-calls-items-type-one-of-1"></span><span id="body-messages-items-one-of-3-all-of-0-tool-calls-type-one-of-1"></span><span id="body-messages-items-one-of-3-tool-calls-items-type-one-of-1"></span><span id="body-messages-items-one-of-3-tool-calls-type-one-of-1"></span><span id="body-messages-items-tool-calls-items-type-one-of-1"></span><span id="body-messages-items-tool-calls-type-one-of-1"></span><span id="body-messages-one-of-3-all-of-0-tool-calls-items-type-one-of-1"></span><span id="body-messages-one-of-3-all-of-0-tool-calls-type-one-of-1"></span><span id="body-messages-one-of-3-tool-calls-items-type-one-of-1"></span><span id="body-messages-one-of-3-tool-calls-type-one-of-1"></span><span id="body-messages-tool-calls-items-type-one-of-1"></span><span id="body-messages-tool-calls-type-one-of-1"></span><span id="response-choices-items-message-tool-calls-items-type-one-of-1"></span><span id="response-choices-items-message-tool-calls-type-one-of-1"></span><span id="response-choices-message-tool-calls-items-type-one-of-1"></span><span id="response-choices-message-tool-calls-type-one-of-1"></span>            - $ref: '#/components/schemas/ChatCompletionToolType'
              description: The type of the tool. Currently, only `function` is supported.
<span id="body-messages-all-of-0-tool-calls-function"></span><span id="body-messages-all-of-0-tool-calls-items-function"></span><span id="body-messages-items-all-of-0-tool-calls-function"></span><span id="body-messages-items-all-of-0-tool-calls-items-function"></span><span id="body-messages-items-one-of-3-all-of-0-tool-calls-function"></span><span id="body-messages-items-one-of-3-all-of-0-tool-calls-items-function"></span><span id="body-messages-items-one-of-3-tool-calls-function"></span><span id="body-messages-items-one-of-3-tool-calls-items-function"></span><span id="body-messages-items-tool-calls-function"></span><span id="body-messages-items-tool-calls-items-function"></span><span id="body-messages-one-of-3-all-of-0-tool-calls-function"></span><span id="body-messages-one-of-3-all-of-0-tool-calls-items-function"></span><span id="body-messages-one-of-3-tool-calls-function"></span><span id="body-messages-one-of-3-tool-calls-items-function"></span><span id="body-messages-tool-calls-function"></span><span id="body-messages-tool-calls-items-function"></span><span id="response-choices-items-message-tool-calls-function"></span><span id="response-choices-items-message-tool-calls-items-function"></span><span id="response-choices-message-tool-calls-function"></span><span id="response-choices-message-tool-calls-items-function"></span>        function:
          $ref: '#/components/schemas/FunctionCall'
          description: The function that the model called.
    FunctionCall:
      type: object
      description: &gt;-
        The name and arguments of a function that should be called, as generated
        by the model.
      required:
        - name
        - arguments
      properties:
<span id="body-messages-all-of-0-function-call-name"></span><span id="body-messages-all-of-0-function-call-one-of-1-name"></span><span id="body-messages-all-of-0-tool-calls-function-name"></span><span id="body-messages-all-of-0-tool-calls-items-function-name"></span><span id="body-messages-function-call-name"></span><span id="body-messages-function-call-one-of-1-name"></span><span id="body-messages-items-all-of-0-function-call-name"></span><span id="body-messages-items-all-of-0-function-call-one-of-1-name"></span><span id="body-messages-items-all-of-0-tool-calls-function-name"></span><span id="body-messages-items-all-of-0-tool-calls-items-function-name"></span><span id="body-messages-items-function-call-name"></span><span id="body-messages-items-function-call-one-of-1-name"></span><span id="body-messages-items-one-of-3-all-of-0-function-call-name"></span><span id="body-messages-items-one-of-3-all-of-0-function-call-one-of-1-name"></span><span id="body-messages-items-one-of-3-all-of-0-tool-calls-function-name"></span><span id="body-messages-items-one-of-3-all-of-0-tool-calls-items-function-name"></span><span id="body-messages-items-one-of-3-function-call-name"></span><span id="body-messages-items-one-of-3-function-call-one-of-1-name"></span><span id="body-messages-items-one-of-3-tool-calls-function-name"></span><span id="body-messages-items-one-of-3-tool-calls-items-function-name"></span><span id="body-messages-items-tool-calls-function-name"></span><span id="body-messages-items-tool-calls-items-function-name"></span><span id="body-messages-one-of-3-all-of-0-function-call-name"></span><span id="body-messages-one-of-3-all-of-0-function-call-one-of-1-name"></span><span id="body-messages-one-of-3-all-of-0-tool-calls-function-name"></span><span id="body-messages-one-of-3-all-of-0-tool-calls-items-function-name"></span><span id="body-messages-one-of-3-function-call-name"></span><span id="body-messages-one-of-3-function-call-one-of-1-name"></span><span id="body-messages-one-of-3-tool-calls-function-name"></span><span id="body-messages-one-of-3-tool-calls-items-function-name"></span><span id="body-messages-tool-calls-function-name"></span><span id="body-messages-tool-calls-items-function-name"></span><span id="response-choices-items-message-function-call-name"></span><span id="response-choices-items-message-function-call-one-of-1-name"></span><span id="response-choices-items-message-tool-calls-function-name"></span><span id="response-choices-items-message-tool-calls-items-function-name"></span><span id="response-choices-message-function-call-name"></span><span id="response-choices-message-function-call-one-of-1-name"></span><span id="response-choices-message-tool-calls-function-name"></span><span id="response-choices-message-tool-calls-items-function-name"></span>        name:
          type: string
          description: The name of the function to call.
<span id="body-messages-all-of-0-function-call-arguments"></span><span id="body-messages-all-of-0-function-call-one-of-1-arguments"></span><span id="body-messages-all-of-0-tool-calls-function-arguments"></span><span id="body-messages-all-of-0-tool-calls-items-function-arguments"></span><span id="body-messages-function-call-arguments"></span><span id="body-messages-function-call-one-of-1-arguments"></span><span id="body-messages-items-all-of-0-function-call-arguments"></span><span id="body-messages-items-all-of-0-function-call-one-of-1-arguments"></span><span id="body-messages-items-all-of-0-tool-calls-function-arguments"></span><span id="body-messages-items-all-of-0-tool-calls-items-function-arguments"></span><span id="body-messages-items-function-call-arguments"></span><span id="body-messages-items-function-call-one-of-1-arguments"></span><span id="body-messages-items-one-of-3-all-of-0-function-call-arguments"></span><span id="body-messages-items-one-of-3-all-of-0-function-call-one-of-1-arguments"></span><span id="body-messages-items-one-of-3-all-of-0-tool-calls-function-arguments"></span><span id="body-messages-items-one-of-3-all-of-0-tool-calls-items-function-arguments"></span><span id="body-messages-items-one-of-3-function-call-arguments"></span><span id="body-messages-items-one-of-3-function-call-one-of-1-arguments"></span><span id="body-messages-items-one-of-3-tool-calls-function-arguments"></span><span id="body-messages-items-one-of-3-tool-calls-items-function-arguments"></span><span id="body-messages-items-tool-calls-function-arguments"></span><span id="body-messages-items-tool-calls-items-function-arguments"></span><span id="body-messages-one-of-3-all-of-0-function-call-arguments"></span><span id="body-messages-one-of-3-all-of-0-function-call-one-of-1-arguments"></span><span id="body-messages-one-of-3-all-of-0-tool-calls-function-arguments"></span><span id="body-messages-one-of-3-all-of-0-tool-calls-items-function-arguments"></span><span id="body-messages-one-of-3-function-call-arguments"></span><span id="body-messages-one-of-3-function-call-one-of-1-arguments"></span><span id="body-messages-one-of-3-tool-calls-function-arguments"></span><span id="body-messages-one-of-3-tool-calls-items-function-arguments"></span><span id="body-messages-tool-calls-function-arguments"></span><span id="body-messages-tool-calls-items-function-arguments"></span><span id="response-choices-items-message-function-call-arguments"></span><span id="response-choices-items-message-function-call-one-of-1-arguments"></span><span id="response-choices-items-message-tool-calls-function-arguments"></span><span id="response-choices-items-message-tool-calls-items-function-arguments"></span><span id="response-choices-message-function-call-arguments"></span><span id="response-choices-message-function-call-one-of-1-arguments"></span><span id="response-choices-message-tool-calls-function-arguments"></span><span id="response-choices-message-tool-calls-items-function-arguments"></span>        arguments:
          type: string
          description: &gt;-
            The arguments to call the function with, as generated by the model
            in JSON format. Note that the model does not always generate valid
            JSON, and may hallucinate parameters not defined by your function
            schema. Validate the arguments in your code before calling your
            function.
    ChatCompletionRequestToolMessageContent:
      oneOf:
<span id="body-messages-items-one-of-4-all-of-0-content-one-of-0"></span><span id="body-messages-items-one-of-4-content-one-of-0"></span><span id="body-messages-one-of-4-all-of-0-content-one-of-0"></span><span id="body-messages-one-of-4-content-one-of-0"></span>        - type: string
          description: The text contents of the tool message.
<span id="body-messages-items-one-of-4-all-of-0-content-one-of-1"></span><span id="body-messages-items-one-of-4-content-one-of-1"></span><span id="body-messages-one-of-4-all-of-0-content-one-of-1"></span><span id="body-messages-one-of-4-content-one-of-1"></span>        - type: array
<span id="body-messages-items-one-of-4-all-of-0-content-items"></span><span id="body-messages-items-one-of-4-all-of-0-content-one-of-1-items"></span><span id="body-messages-items-one-of-4-content-items"></span><span id="body-messages-items-one-of-4-content-one-of-1-items"></span><span id="body-messages-one-of-4-all-of-0-content-items"></span><span id="body-messages-one-of-4-all-of-0-content-one-of-1-items"></span><span id="body-messages-one-of-4-content-items"></span><span id="body-messages-one-of-4-content-one-of-1-items"></span>          items:
            $ref: '#/components/schemas/ChatCompletionRequestToolMessageContentPart'
          description: &gt;-
            An array of content parts with a defined type. For tool messages,
            only type `text` is supported.
      description: &gt;-
        The contents of a tool message: either a text string or an array of text
        content parts.
    ChatCompletionRequestMessageContentPartText:
      type: object
      description: A text content part of a message.
      required:
        - text
      properties:
<span id="body-messages-all-of-0-content-items-text"></span><span id="body-messages-all-of-0-content-one-of-1-items-text"></span><span id="body-messages-all-of-0-content-one-of-1-text"></span><span id="body-messages-all-of-0-content-text"></span><span id="body-messages-content-items-text"></span><span id="body-messages-content-one-of-1-items-text"></span><span id="body-messages-content-one-of-1-text"></span><span id="body-messages-content-text"></span><span id="body-messages-items-all-of-0-content-items-text"></span><span id="body-messages-items-all-of-0-content-one-of-1-items-text"></span><span id="body-messages-items-all-of-0-content-one-of-1-text"></span><span id="body-messages-items-all-of-0-content-text"></span><span id="body-messages-items-content-items-text"></span><span id="body-messages-items-content-one-of-1-items-text"></span><span id="body-messages-items-content-one-of-1-text"></span><span id="body-messages-items-content-text"></span><span id="body-messages-items-one-of-0-all-of-0-content-items-text"></span><span id="body-messages-items-one-of-0-all-of-0-content-one-of-1-items-text"></span><span id="body-messages-items-one-of-0-all-of-0-content-one-of-1-text"></span><span id="body-messages-items-one-of-0-all-of-0-content-text"></span><span id="body-messages-items-one-of-0-content-items-text"></span><span id="body-messages-items-one-of-0-content-one-of-1-items-text"></span><span id="body-messages-items-one-of-0-content-one-of-1-text"></span><span id="body-messages-items-one-of-0-content-text"></span><span id="body-messages-one-of-0-all-of-0-content-items-text"></span><span id="body-messages-one-of-0-all-of-0-content-one-of-1-items-text"></span><span id="body-messages-one-of-0-all-of-0-content-one-of-1-text"></span><span id="body-messages-one-of-0-all-of-0-content-text"></span><span id="body-messages-one-of-0-content-items-text"></span><span id="body-messages-one-of-0-content-one-of-1-items-text"></span><span id="body-messages-one-of-0-content-one-of-1-text"></span><span id="body-messages-one-of-0-content-text"></span><span id="body-prediction-content-items-text"></span><span id="body-prediction-content-one-of-1-items-text"></span><span id="body-prediction-content-one-of-1-text"></span><span id="body-prediction-content-text"></span><span id="body-prediction-one-of-0-content-items-text"></span><span id="body-prediction-one-of-0-content-one-of-1-items-text"></span><span id="body-prediction-one-of-0-content-one-of-1-text"></span><span id="body-prediction-one-of-0-content-text"></span><span id="body-prediction-one-of-1-content-items-text"></span><span id="body-prediction-one-of-1-content-one-of-1-items-text"></span><span id="body-prediction-one-of-1-content-one-of-1-text"></span><span id="body-prediction-one-of-1-content-text"></span><span id="body-prediction-one-of-1-one-of-0-content-items-text"></span><span id="body-prediction-one-of-1-one-of-0-content-one-of-1-items-text"></span><span id="body-prediction-one-of-1-one-of-0-content-one-of-1-text"></span><span id="body-prediction-one-of-1-one-of-0-content-text"></span>        text:
          type: string
    FunctionName:
      type: object
      description: A wrapper for a function name reference.
      required:
        - name
      properties:
<span id="body-tool-choice-named-function-name"></span><span id="body-tool-choice-one-of-1-named-function-name"></span><span id="body-tool-choice-one-of-1-one-of-1-named-function-name"></span>        name:
          type: string
          description: The name of the function to call.
    Role:
      type: string
      description: The role of the author of a message.
      enum:
        - system
        - user
        - assistant
        - tool
        - function
    ChatCompletionResponseMessageAudio:
      type: object
      description: Audio data in a chat completion response message.
      required:
        - id
        - expires_at
        - data
        - transcript
      properties:
<span id="response-choices-items-message-audio-id"></span><span id="response-choices-items-message-audio-one-of-1-id"></span><span id="response-choices-message-audio-id"></span><span id="response-choices-message-audio-one-of-1-id"></span>        id:
          type: string
          description: Unique identifier for this audio response.
<span id="response-choices-items-message-audio-expires-at"></span><span id="response-choices-items-message-audio-one-of-1-expires-at"></span><span id="response-choices-message-audio-expires-at"></span><span id="response-choices-message-audio-one-of-1-expires-at"></span>        expires_at:
          type: integer
          format: u-int32
          description: &gt;-
            The Unix timestamp (in seconds) for when this audio response will no
            longer be accessible on the server for use in multi-turn
            conversations.
          minimum: 0
<span id="response-choices-items-message-audio-data"></span><span id="response-choices-items-message-audio-one-of-1-data"></span><span id="response-choices-message-audio-data"></span><span id="response-choices-message-audio-one-of-1-data"></span>        data:
          type: string
          description: &gt;-
            Base64 encoded audio bytes generated by the model, in the format
            specified in the request.
<span id="response-choices-items-message-audio-one-of-1-transcript"></span><span id="response-choices-items-message-audio-transcript"></span><span id="response-choices-message-audio-one-of-1-transcript"></span><span id="response-choices-message-audio-transcript"></span>        transcript:
          type: string
          description: Transcript of the audio generated by the model.
    ChatCompletionTokenLogprob:
      type: object
      description: Log probability information for an output token.
      required:
        - token
        - logprob
        - top_logprobs
      properties:
<span id="response-choices-items-logprobs-content-items-token"></span><span id="response-choices-items-logprobs-content-token"></span><span id="response-choices-items-logprobs-one-of-1-content-items-token"></span><span id="response-choices-items-logprobs-one-of-1-content-token"></span><span id="response-choices-items-logprobs-one-of-1-refusal-items-token"></span><span id="response-choices-items-logprobs-one-of-1-refusal-token"></span><span id="response-choices-items-logprobs-refusal-items-token"></span><span id="response-choices-items-logprobs-refusal-token"></span><span id="response-choices-logprobs-content-items-token"></span><span id="response-choices-logprobs-content-token"></span><span id="response-choices-logprobs-one-of-1-content-items-token"></span><span id="response-choices-logprobs-one-of-1-content-token"></span><span id="response-choices-logprobs-one-of-1-refusal-items-token"></span><span id="response-choices-logprobs-one-of-1-refusal-token"></span><span id="response-choices-logprobs-refusal-items-token"></span><span id="response-choices-logprobs-refusal-token"></span>        token:
          type: string
          description: The token.
<span id="response-choices-items-logprobs-content-items-logprob"></span><span id="response-choices-items-logprobs-content-logprob"></span><span id="response-choices-items-logprobs-one-of-1-content-items-logprob"></span><span id="response-choices-items-logprobs-one-of-1-content-logprob"></span><span id="response-choices-items-logprobs-one-of-1-refusal-items-logprob"></span><span id="response-choices-items-logprobs-one-of-1-refusal-logprob"></span><span id="response-choices-items-logprobs-refusal-items-logprob"></span><span id="response-choices-items-logprobs-refusal-logprob"></span><span id="response-choices-logprobs-content-items-logprob"></span><span id="response-choices-logprobs-content-logprob"></span><span id="response-choices-logprobs-one-of-1-content-items-logprob"></span><span id="response-choices-logprobs-one-of-1-content-logprob"></span><span id="response-choices-logprobs-one-of-1-refusal-items-logprob"></span><span id="response-choices-logprobs-one-of-1-refusal-logprob"></span><span id="response-choices-logprobs-refusal-items-logprob"></span><span id="response-choices-logprobs-refusal-logprob"></span>        logprob:
          type: number
          format: float
          description: &gt;-
            The log probability of this token, if it is within the top 20 most
            likely tokens. Otherwise, the value `-9999.0` is used to signify
            that the token is very unlikely.
<span id="response-choices-items-logprobs-content-bytes"></span><span id="response-choices-items-logprobs-content-items-bytes"></span><span id="response-choices-items-logprobs-one-of-1-content-bytes"></span><span id="response-choices-items-logprobs-one-of-1-content-items-bytes"></span><span id="response-choices-items-logprobs-one-of-1-refusal-bytes"></span><span id="response-choices-items-logprobs-one-of-1-refusal-items-bytes"></span><span id="response-choices-items-logprobs-refusal-bytes"></span><span id="response-choices-items-logprobs-refusal-items-bytes"></span><span id="response-choices-logprobs-content-bytes"></span><span id="response-choices-logprobs-content-items-bytes"></span><span id="response-choices-logprobs-one-of-1-content-bytes"></span><span id="response-choices-logprobs-one-of-1-content-items-bytes"></span><span id="response-choices-logprobs-one-of-1-refusal-bytes"></span><span id="response-choices-logprobs-one-of-1-refusal-items-bytes"></span><span id="response-choices-logprobs-refusal-bytes"></span><span id="response-choices-logprobs-refusal-items-bytes"></span>        bytes:
          type:
            - array
            - 'null'
<span id="response-choices-items-logprobs-content-bytes-items"></span><span id="response-choices-items-logprobs-content-items-bytes-items"></span><span id="response-choices-items-logprobs-one-of-1-content-bytes-items"></span><span id="response-choices-items-logprobs-one-of-1-content-items-bytes-items"></span><span id="response-choices-items-logprobs-one-of-1-refusal-bytes-items"></span><span id="response-choices-items-logprobs-one-of-1-refusal-items-bytes-items"></span><span id="response-choices-items-logprobs-refusal-bytes-items"></span><span id="response-choices-items-logprobs-refusal-items-bytes-items"></span><span id="response-choices-logprobs-content-bytes-items"></span><span id="response-choices-logprobs-content-items-bytes-items"></span><span id="response-choices-logprobs-one-of-1-content-bytes-items"></span><span id="response-choices-logprobs-one-of-1-content-items-bytes-items"></span><span id="response-choices-logprobs-one-of-1-refusal-bytes-items"></span><span id="response-choices-logprobs-one-of-1-refusal-items-bytes-items"></span><span id="response-choices-logprobs-refusal-bytes-items"></span><span id="response-choices-logprobs-refusal-items-bytes-items"></span>          items:
            type: integer
            format: u-int8
            minimum: 0
          description: &gt;-
            A list of integers representing the UTF-8 bytes representation of
            the token. Useful in instances where characters are represented by
            multiple tokens and their byte representations must be combined to
            generate the correct text representation. Can be `null` if there is
            no bytes representation for the token.
<span id="response-choices-items-logprobs-content-items-top-logprobs"></span><span id="response-choices-items-logprobs-content-top-logprobs"></span><span id="response-choices-items-logprobs-one-of-1-content-items-top-logprobs"></span><span id="response-choices-items-logprobs-one-of-1-content-top-logprobs"></span><span id="response-choices-items-logprobs-one-of-1-refusal-items-top-logprobs"></span><span id="response-choices-items-logprobs-one-of-1-refusal-top-logprobs"></span><span id="response-choices-items-logprobs-refusal-items-top-logprobs"></span><span id="response-choices-items-logprobs-refusal-top-logprobs"></span><span id="response-choices-logprobs-content-items-top-logprobs"></span><span id="response-choices-logprobs-content-top-logprobs"></span><span id="response-choices-logprobs-one-of-1-content-items-top-logprobs"></span><span id="response-choices-logprobs-one-of-1-content-top-logprobs"></span><span id="response-choices-logprobs-one-of-1-refusal-items-top-logprobs"></span><span id="response-choices-logprobs-one-of-1-refusal-top-logprobs"></span><span id="response-choices-logprobs-refusal-items-top-logprobs"></span><span id="response-choices-logprobs-refusal-top-logprobs"></span>        top_logprobs:
          type: array
<span id="response-choices-items-logprobs-content-items-top-logprobs-items"></span><span id="response-choices-items-logprobs-content-top-logprobs-items"></span><span id="response-choices-items-logprobs-one-of-1-content-items-top-logprobs-items"></span><span id="response-choices-items-logprobs-one-of-1-content-top-logprobs-items"></span><span id="response-choices-items-logprobs-one-of-1-refusal-items-top-logprobs-items"></span><span id="response-choices-items-logprobs-one-of-1-refusal-top-logprobs-items"></span><span id="response-choices-items-logprobs-refusal-items-top-logprobs-items"></span><span id="response-choices-items-logprobs-refusal-top-logprobs-items"></span><span id="response-choices-logprobs-content-items-top-logprobs-items"></span><span id="response-choices-logprobs-content-top-logprobs-items"></span><span id="response-choices-logprobs-one-of-1-content-items-top-logprobs-items"></span><span id="response-choices-logprobs-one-of-1-content-top-logprobs-items"></span><span id="response-choices-logprobs-one-of-1-refusal-items-top-logprobs-items"></span><span id="response-choices-logprobs-one-of-1-refusal-top-logprobs-items"></span><span id="response-choices-logprobs-refusal-items-top-logprobs-items"></span><span id="response-choices-logprobs-refusal-top-logprobs-items"></span>          items:
            $ref: '#/components/schemas/TopLogprobs'
          description: &gt;-
            List of the most likely tokens and their log probability, at this
            token position. In rare cases, there may be fewer than the number of
            requested `top_logprobs` returned.
    ChatCompletionRequestSystemMessageContentPart:
      oneOf:
<span id="body-messages-all-of-0-content-items-one-of-0"></span><span id="body-messages-all-of-0-content-one-of-1-items-one-of-0"></span><span id="body-messages-all-of-0-content-one-of-1-one-of-0"></span><span id="body-messages-content-items-one-of-0"></span><span id="body-messages-content-one-of-1-items-one-of-0"></span><span id="body-messages-content-one-of-1-one-of-0"></span><span id="body-messages-items-all-of-0-content-items-one-of-0"></span><span id="body-messages-items-all-of-0-content-one-of-1-items-one-of-0"></span><span id="body-messages-items-all-of-0-content-one-of-1-one-of-0"></span><span id="body-messages-items-content-items-one-of-0"></span><span id="body-messages-items-content-one-of-1-items-one-of-0"></span><span id="body-messages-items-content-one-of-1-one-of-0"></span><span id="body-messages-items-one-of-1-all-of-0-content-items-one-of-0"></span><span id="body-messages-items-one-of-1-all-of-0-content-one-of-1-items-one-of-0"></span><span id="body-messages-items-one-of-1-all-of-0-content-one-of-1-one-of-0"></span><span id="body-messages-items-one-of-1-content-items-one-of-0"></span><span id="body-messages-items-one-of-1-content-one-of-1-items-one-of-0"></span><span id="body-messages-items-one-of-1-content-one-of-1-one-of-0"></span><span id="body-messages-one-of-1-all-of-0-content-items-one-of-0"></span><span id="body-messages-one-of-1-all-of-0-content-one-of-1-items-one-of-0"></span><span id="body-messages-one-of-1-all-of-0-content-one-of-1-one-of-0"></span><span id="body-messages-one-of-1-content-items-one-of-0"></span><span id="body-messages-one-of-1-content-one-of-1-items-one-of-0"></span><span id="body-messages-one-of-1-content-one-of-1-one-of-0"></span>        - allOf:
<span id="body-messages-all-of-0-content-all-of-0"></span><span id="body-messages-all-of-0-content-items-all-of-0"></span><span id="body-messages-all-of-0-content-items-one-of-0-all-of-0"></span><span id="body-messages-all-of-0-content-one-of-0-all-of-0"></span><span id="body-messages-all-of-0-content-one-of-1-all-of-0"></span><span id="body-messages-all-of-0-content-one-of-1-items-all-of-0"></span><span id="body-messages-all-of-0-content-one-of-1-items-one-of-0-all-of-0"></span><span id="body-messages-all-of-0-content-one-of-1-one-of-0-all-of-0"></span><span id="body-messages-content-all-of-0"></span><span id="body-messages-content-items-all-of-0"></span><span id="body-messages-content-items-one-of-0-all-of-0"></span><span id="body-messages-content-one-of-0-all-of-0"></span><span id="body-messages-content-one-of-1-all-of-0"></span><span id="body-messages-content-one-of-1-items-all-of-0"></span><span id="body-messages-content-one-of-1-items-one-of-0-all-of-0"></span><span id="body-messages-content-one-of-1-one-of-0-all-of-0"></span><span id="body-messages-items-all-of-0-content-all-of-0"></span><span id="body-messages-items-all-of-0-content-items-all-of-0"></span><span id="body-messages-items-all-of-0-content-items-one-of-0-all-of-0"></span><span id="body-messages-items-all-of-0-content-one-of-0-all-of-0"></span><span id="body-messages-items-all-of-0-content-one-of-1-all-of-0"></span><span id="body-messages-items-all-of-0-content-one-of-1-items-all-of-0"></span><span id="body-messages-items-all-of-0-content-one-of-1-items-one-of-0-all-of-0"></span><span id="body-messages-items-all-of-0-content-one-of-1-one-of-0-all-of-0"></span><span id="body-messages-items-content-all-of-0"></span><span id="body-messages-items-content-items-all-of-0"></span><span id="body-messages-items-content-items-one-of-0-all-of-0"></span><span id="body-messages-items-content-one-of-0-all-of-0"></span><span id="body-messages-items-content-one-of-1-all-of-0"></span><span id="body-messages-items-content-one-of-1-items-all-of-0"></span><span id="body-messages-items-content-one-of-1-items-one-of-0-all-of-0"></span><span id="body-messages-items-content-one-of-1-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-1-all-of-0-content-all-of-0"></span><span id="body-messages-items-one-of-1-all-of-0-content-items-all-of-0"></span><span id="body-messages-items-one-of-1-all-of-0-content-items-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-1-all-of-0-content-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-1-all-of-0-content-one-of-1-all-of-0"></span><span id="body-messages-items-one-of-1-all-of-0-content-one-of-1-items-all-of-0"></span><span id="body-messages-items-one-of-1-all-of-0-content-one-of-1-items-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-1-all-of-0-content-one-of-1-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-1-content-all-of-0"></span><span id="body-messages-items-one-of-1-content-items-all-of-0"></span><span id="body-messages-items-one-of-1-content-items-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-1-content-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-1-content-one-of-1-all-of-0"></span><span id="body-messages-items-one-of-1-content-one-of-1-items-all-of-0"></span><span id="body-messages-items-one-of-1-content-one-of-1-items-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-1-content-one-of-1-one-of-0-all-of-0"></span><span id="body-messages-one-of-1-all-of-0-content-all-of-0"></span><span id="body-messages-one-of-1-all-of-0-content-items-all-of-0"></span><span id="body-messages-one-of-1-all-of-0-content-items-one-of-0-all-of-0"></span><span id="body-messages-one-of-1-all-of-0-content-one-of-0-all-of-0"></span><span id="body-messages-one-of-1-all-of-0-content-one-of-1-all-of-0"></span><span id="body-messages-one-of-1-all-of-0-content-one-of-1-items-all-of-0"></span><span id="body-messages-one-of-1-all-of-0-content-one-of-1-items-one-of-0-all-of-0"></span><span id="body-messages-one-of-1-all-of-0-content-one-of-1-one-of-0-all-of-0"></span><span id="body-messages-one-of-1-content-all-of-0"></span><span id="body-messages-one-of-1-content-items-all-of-0"></span><span id="body-messages-one-of-1-content-items-one-of-0-all-of-0"></span><span id="body-messages-one-of-1-content-one-of-0-all-of-0"></span><span id="body-messages-one-of-1-content-one-of-1-all-of-0"></span><span id="body-messages-one-of-1-content-one-of-1-items-all-of-0"></span><span id="body-messages-one-of-1-content-one-of-1-items-one-of-0-all-of-0"></span><span id="body-messages-one-of-1-content-one-of-1-one-of-0-all-of-0"></span>            - $ref: '#/components/schemas/ChatCompletionRequestMessageContentPartText'
<span id="body-messages-all-of-0-content-all-of-1"></span><span id="body-messages-all-of-0-content-items-all-of-1"></span><span id="body-messages-all-of-0-content-items-one-of-0-all-of-1"></span><span id="body-messages-all-of-0-content-one-of-0-all-of-1"></span><span id="body-messages-all-of-0-content-one-of-1-all-of-1"></span><span id="body-messages-all-of-0-content-one-of-1-items-all-of-1"></span><span id="body-messages-all-of-0-content-one-of-1-items-one-of-0-all-of-1"></span><span id="body-messages-all-of-0-content-one-of-1-one-of-0-all-of-1"></span><span id="body-messages-content-all-of-1"></span><span id="body-messages-content-items-all-of-1"></span><span id="body-messages-content-items-one-of-0-all-of-1"></span><span id="body-messages-content-one-of-0-all-of-1"></span><span id="body-messages-content-one-of-1-all-of-1"></span><span id="body-messages-content-one-of-1-items-all-of-1"></span><span id="body-messages-content-one-of-1-items-one-of-0-all-of-1"></span><span id="body-messages-content-one-of-1-one-of-0-all-of-1"></span><span id="body-messages-items-all-of-0-content-all-of-1"></span><span id="body-messages-items-all-of-0-content-items-all-of-1"></span><span id="body-messages-items-all-of-0-content-items-one-of-0-all-of-1"></span><span id="body-messages-items-all-of-0-content-one-of-0-all-of-1"></span><span id="body-messages-items-all-of-0-content-one-of-1-all-of-1"></span><span id="body-messages-items-all-of-0-content-one-of-1-items-all-of-1"></span><span id="body-messages-items-all-of-0-content-one-of-1-items-one-of-0-all-of-1"></span><span id="body-messages-items-all-of-0-content-one-of-1-one-of-0-all-of-1"></span><span id="body-messages-items-content-all-of-1"></span><span id="body-messages-items-content-items-all-of-1"></span><span id="body-messages-items-content-items-one-of-0-all-of-1"></span><span id="body-messages-items-content-one-of-0-all-of-1"></span><span id="body-messages-items-content-one-of-1-all-of-1"></span><span id="body-messages-items-content-one-of-1-items-all-of-1"></span><span id="body-messages-items-content-one-of-1-items-one-of-0-all-of-1"></span><span id="body-messages-items-content-one-of-1-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-1-all-of-0-content-all-of-1"></span><span id="body-messages-items-one-of-1-all-of-0-content-items-all-of-1"></span><span id="body-messages-items-one-of-1-all-of-0-content-items-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-1-all-of-0-content-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-1-all-of-0-content-one-of-1-all-of-1"></span><span id="body-messages-items-one-of-1-all-of-0-content-one-of-1-items-all-of-1"></span><span id="body-messages-items-one-of-1-all-of-0-content-one-of-1-items-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-1-all-of-0-content-one-of-1-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-1-content-all-of-1"></span><span id="body-messages-items-one-of-1-content-items-all-of-1"></span><span id="body-messages-items-one-of-1-content-items-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-1-content-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-1-content-one-of-1-all-of-1"></span><span id="body-messages-items-one-of-1-content-one-of-1-items-all-of-1"></span><span id="body-messages-items-one-of-1-content-one-of-1-items-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-1-content-one-of-1-one-of-0-all-of-1"></span><span id="body-messages-one-of-1-all-of-0-content-all-of-1"></span><span id="body-messages-one-of-1-all-of-0-content-items-all-of-1"></span><span id="body-messages-one-of-1-all-of-0-content-items-one-of-0-all-of-1"></span><span id="body-messages-one-of-1-all-of-0-content-one-of-0-all-of-1"></span><span id="body-messages-one-of-1-all-of-0-content-one-of-1-all-of-1"></span><span id="body-messages-one-of-1-all-of-0-content-one-of-1-items-all-of-1"></span><span id="body-messages-one-of-1-all-of-0-content-one-of-1-items-one-of-0-all-of-1"></span><span id="body-messages-one-of-1-all-of-0-content-one-of-1-one-of-0-all-of-1"></span><span id="body-messages-one-of-1-content-all-of-1"></span><span id="body-messages-one-of-1-content-items-all-of-1"></span><span id="body-messages-one-of-1-content-items-one-of-0-all-of-1"></span><span id="body-messages-one-of-1-content-one-of-0-all-of-1"></span><span id="body-messages-one-of-1-content-one-of-1-all-of-1"></span><span id="body-messages-one-of-1-content-one-of-1-items-all-of-1"></span><span id="body-messages-one-of-1-content-one-of-1-items-one-of-0-all-of-1"></span><span id="body-messages-one-of-1-content-one-of-1-one-of-0-all-of-1"></span>            - type: object
              required:
                - type
              properties:
                type:
                  type: string
                  enum:
                    - text
      description: A content part in a system message (text only).
    ChatCompletionRequestUserMessageContentPart:
      oneOf:
<span id="body-messages-items-one-of-2-all-of-0-content-items-one-of-0"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-items-one-of-0"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-one-of-0"></span><span id="body-messages-items-one-of-2-content-items-one-of-0"></span><span id="body-messages-items-one-of-2-content-one-of-1-items-one-of-0"></span><span id="body-messages-items-one-of-2-content-one-of-1-one-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-items-one-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-items-one-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-one-of-0"></span><span id="body-messages-one-of-2-content-items-one-of-0"></span><span id="body-messages-one-of-2-content-one-of-1-items-one-of-0"></span><span id="body-messages-one-of-2-content-one-of-1-one-of-0"></span>        - allOf:
<span id="body-messages-items-one-of-2-all-of-0-content-all-of-0"></span><span id="body-messages-items-one-of-2-all-of-0-content-items-all-of-0"></span><span id="body-messages-items-one-of-2-all-of-0-content-items-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-all-of-0"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-items-all-of-0"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-items-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-2-content-all-of-0"></span><span id="body-messages-items-one-of-2-content-items-all-of-0"></span><span id="body-messages-items-one-of-2-content-items-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-2-content-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-2-content-one-of-1-all-of-0"></span><span id="body-messages-items-one-of-2-content-one-of-1-items-all-of-0"></span><span id="body-messages-items-one-of-2-content-one-of-1-items-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-2-content-one-of-1-one-of-0-all-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-all-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-items-all-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-items-one-of-0-all-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-0-all-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-all-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-items-all-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-items-one-of-0-all-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-one-of-0-all-of-0"></span><span id="body-messages-one-of-2-content-all-of-0"></span><span id="body-messages-one-of-2-content-items-all-of-0"></span><span id="body-messages-one-of-2-content-items-one-of-0-all-of-0"></span><span id="body-messages-one-of-2-content-one-of-0-all-of-0"></span><span id="body-messages-one-of-2-content-one-of-1-all-of-0"></span><span id="body-messages-one-of-2-content-one-of-1-items-all-of-0"></span><span id="body-messages-one-of-2-content-one-of-1-items-one-of-0-all-of-0"></span><span id="body-messages-one-of-2-content-one-of-1-one-of-0-all-of-0"></span>            - $ref: '#/components/schemas/ChatCompletionRequestMessageContentPartText'
<span id="body-messages-items-one-of-2-all-of-0-content-all-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-items-all-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-items-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-all-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-items-all-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-items-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-2-content-all-of-1"></span><span id="body-messages-items-one-of-2-content-items-all-of-1"></span><span id="body-messages-items-one-of-2-content-items-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-2-content-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-2-content-one-of-1-all-of-1"></span><span id="body-messages-items-one-of-2-content-one-of-1-items-all-of-1"></span><span id="body-messages-items-one-of-2-content-one-of-1-items-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-2-content-one-of-1-one-of-0-all-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-all-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-items-all-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-items-one-of-0-all-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-0-all-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-all-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-items-all-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-items-one-of-0-all-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-one-of-0-all-of-1"></span><span id="body-messages-one-of-2-content-all-of-1"></span><span id="body-messages-one-of-2-content-items-all-of-1"></span><span id="body-messages-one-of-2-content-items-one-of-0-all-of-1"></span><span id="body-messages-one-of-2-content-one-of-0-all-of-1"></span><span id="body-messages-one-of-2-content-one-of-1-all-of-1"></span><span id="body-messages-one-of-2-content-one-of-1-items-all-of-1"></span><span id="body-messages-one-of-2-content-one-of-1-items-one-of-0-all-of-1"></span><span id="body-messages-one-of-2-content-one-of-1-one-of-0-all-of-1"></span>            - type: object
              required:
                - type
              properties:
                type:
                  type: string
                  enum:
                    - text
<span id="body-messages-all-of-0-content-items-one-of-1"></span><span id="body-messages-all-of-0-content-one-of-1-items-one-of-1"></span><span id="body-messages-all-of-0-content-one-of-1-one-of-1"></span><span id="body-messages-content-items-one-of-1"></span><span id="body-messages-content-one-of-1-items-one-of-1"></span><span id="body-messages-content-one-of-1-one-of-1"></span><span id="body-messages-items-all-of-0-content-items-one-of-1"></span><span id="body-messages-items-all-of-0-content-one-of-1-items-one-of-1"></span><span id="body-messages-items-all-of-0-content-one-of-1-one-of-1"></span><span id="body-messages-items-content-items-one-of-1"></span><span id="body-messages-items-content-one-of-1-items-one-of-1"></span><span id="body-messages-items-content-one-of-1-one-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-items-one-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-items-one-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-one-of-1"></span><span id="body-messages-items-one-of-2-content-items-one-of-1"></span><span id="body-messages-items-one-of-2-content-one-of-1-items-one-of-1"></span><span id="body-messages-items-one-of-2-content-one-of-1-one-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-items-one-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-items-one-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-one-of-1"></span><span id="body-messages-one-of-2-content-items-one-of-1"></span><span id="body-messages-one-of-2-content-one-of-1-items-one-of-1"></span><span id="body-messages-one-of-2-content-one-of-1-one-of-1"></span>        - allOf:
<span id="body-messages-all-of-0-content-items-one-of-1-all-of-0"></span><span id="body-messages-all-of-0-content-one-of-1-items-one-of-1-all-of-0"></span><span id="body-messages-all-of-0-content-one-of-1-one-of-1-all-of-0"></span><span id="body-messages-content-items-one-of-1-all-of-0"></span><span id="body-messages-content-one-of-1-items-one-of-1-all-of-0"></span><span id="body-messages-content-one-of-1-one-of-1-all-of-0"></span><span id="body-messages-items-all-of-0-content-items-one-of-1-all-of-0"></span><span id="body-messages-items-all-of-0-content-one-of-1-items-one-of-1-all-of-0"></span><span id="body-messages-items-all-of-0-content-one-of-1-one-of-1-all-of-0"></span><span id="body-messages-items-content-items-one-of-1-all-of-0"></span><span id="body-messages-items-content-one-of-1-items-one-of-1-all-of-0"></span><span id="body-messages-items-content-one-of-1-one-of-1-all-of-0"></span><span id="body-messages-items-one-of-2-all-of-0-content-items-one-of-1-all-of-0"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-items-one-of-1-all-of-0"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-one-of-1-all-of-0"></span><span id="body-messages-items-one-of-2-content-items-one-of-1-all-of-0"></span><span id="body-messages-items-one-of-2-content-one-of-1-items-one-of-1-all-of-0"></span><span id="body-messages-items-one-of-2-content-one-of-1-one-of-1-all-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-items-one-of-1-all-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-items-one-of-1-all-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-one-of-1-all-of-0"></span><span id="body-messages-one-of-2-content-items-one-of-1-all-of-0"></span><span id="body-messages-one-of-2-content-one-of-1-items-one-of-1-all-of-0"></span><span id="body-messages-one-of-2-content-one-of-1-one-of-1-all-of-0"></span>            - $ref: &gt;-
                #/components/schemas/ChatCompletionRequestMessageContentPartImage
<span id="body-messages-all-of-0-content-items-one-of-1-all-of-1"></span><span id="body-messages-all-of-0-content-one-of-1-items-one-of-1-all-of-1"></span><span id="body-messages-all-of-0-content-one-of-1-one-of-1-all-of-1"></span><span id="body-messages-content-items-one-of-1-all-of-1"></span><span id="body-messages-content-one-of-1-items-one-of-1-all-of-1"></span><span id="body-messages-content-one-of-1-one-of-1-all-of-1"></span><span id="body-messages-items-all-of-0-content-items-one-of-1-all-of-1"></span><span id="body-messages-items-all-of-0-content-one-of-1-items-one-of-1-all-of-1"></span><span id="body-messages-items-all-of-0-content-one-of-1-one-of-1-all-of-1"></span><span id="body-messages-items-content-items-one-of-1-all-of-1"></span><span id="body-messages-items-content-one-of-1-items-one-of-1-all-of-1"></span><span id="body-messages-items-content-one-of-1-one-of-1-all-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-items-one-of-1-all-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-items-one-of-1-all-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-one-of-1-all-of-1"></span><span id="body-messages-items-one-of-2-content-items-one-of-1-all-of-1"></span><span id="body-messages-items-one-of-2-content-one-of-1-items-one-of-1-all-of-1"></span><span id="body-messages-items-one-of-2-content-one-of-1-one-of-1-all-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-items-one-of-1-all-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-items-one-of-1-all-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-one-of-1-all-of-1"></span><span id="body-messages-one-of-2-content-items-one-of-1-all-of-1"></span><span id="body-messages-one-of-2-content-one-of-1-items-one-of-1-all-of-1"></span><span id="body-messages-one-of-2-content-one-of-1-one-of-1-all-of-1"></span>            - type: object
              required:
                - type
              properties:
                type:
                  type: string
                  enum:
                    - image_url
<span id="body-messages-all-of-0-content-items-one-of-2"></span><span id="body-messages-all-of-0-content-one-of-1-items-one-of-2"></span><span id="body-messages-all-of-0-content-one-of-1-one-of-2"></span><span id="body-messages-all-of-0-content-one-of-2"></span><span id="body-messages-content-items-one-of-2"></span><span id="body-messages-content-one-of-1-items-one-of-2"></span><span id="body-messages-content-one-of-1-one-of-2"></span><span id="body-messages-content-one-of-2"></span><span id="body-messages-items-all-of-0-content-items-one-of-2"></span><span id="body-messages-items-all-of-0-content-one-of-1-items-one-of-2"></span><span id="body-messages-items-all-of-0-content-one-of-1-one-of-2"></span><span id="body-messages-items-all-of-0-content-one-of-2"></span><span id="body-messages-items-content-items-one-of-2"></span><span id="body-messages-items-content-one-of-1-items-one-of-2"></span><span id="body-messages-items-content-one-of-1-one-of-2"></span><span id="body-messages-items-content-one-of-2"></span><span id="body-messages-items-one-of-2-all-of-0-content-items-one-of-2"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-items-one-of-2"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-one-of-2"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-2"></span><span id="body-messages-items-one-of-2-content-items-one-of-2"></span><span id="body-messages-items-one-of-2-content-one-of-1-items-one-of-2"></span><span id="body-messages-items-one-of-2-content-one-of-1-one-of-2"></span><span id="body-messages-items-one-of-2-content-one-of-2"></span><span id="body-messages-one-of-2-all-of-0-content-items-one-of-2"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-items-one-of-2"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-one-of-2"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-2"></span><span id="body-messages-one-of-2-content-items-one-of-2"></span><span id="body-messages-one-of-2-content-one-of-1-items-one-of-2"></span><span id="body-messages-one-of-2-content-one-of-1-one-of-2"></span><span id="body-messages-one-of-2-content-one-of-2"></span>        - allOf:
<span id="body-messages-all-of-0-content-items-one-of-2-all-of-0"></span><span id="body-messages-all-of-0-content-one-of-1-items-one-of-2-all-of-0"></span><span id="body-messages-all-of-0-content-one-of-1-one-of-2-all-of-0"></span><span id="body-messages-all-of-0-content-one-of-2-all-of-0"></span><span id="body-messages-content-items-one-of-2-all-of-0"></span><span id="body-messages-content-one-of-1-items-one-of-2-all-of-0"></span><span id="body-messages-content-one-of-1-one-of-2-all-of-0"></span><span id="body-messages-content-one-of-2-all-of-0"></span><span id="body-messages-items-all-of-0-content-items-one-of-2-all-of-0"></span><span id="body-messages-items-all-of-0-content-one-of-1-items-one-of-2-all-of-0"></span><span id="body-messages-items-all-of-0-content-one-of-1-one-of-2-all-of-0"></span><span id="body-messages-items-all-of-0-content-one-of-2-all-of-0"></span><span id="body-messages-items-content-items-one-of-2-all-of-0"></span><span id="body-messages-items-content-one-of-1-items-one-of-2-all-of-0"></span><span id="body-messages-items-content-one-of-1-one-of-2-all-of-0"></span><span id="body-messages-items-content-one-of-2-all-of-0"></span><span id="body-messages-items-one-of-2-all-of-0-content-items-one-of-2-all-of-0"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-items-one-of-2-all-of-0"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-one-of-2-all-of-0"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-2-all-of-0"></span><span id="body-messages-items-one-of-2-content-items-one-of-2-all-of-0"></span><span id="body-messages-items-one-of-2-content-one-of-1-items-one-of-2-all-of-0"></span><span id="body-messages-items-one-of-2-content-one-of-1-one-of-2-all-of-0"></span><span id="body-messages-items-one-of-2-content-one-of-2-all-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-items-one-of-2-all-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-items-one-of-2-all-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-one-of-2-all-of-0"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-2-all-of-0"></span><span id="body-messages-one-of-2-content-items-one-of-2-all-of-0"></span><span id="body-messages-one-of-2-content-one-of-1-items-one-of-2-all-of-0"></span><span id="body-messages-one-of-2-content-one-of-1-one-of-2-all-of-0"></span><span id="body-messages-one-of-2-content-one-of-2-all-of-0"></span>            - $ref: &gt;-
                #/components/schemas/ChatCompletionRequestMessageContentPartAudio
<span id="body-messages-all-of-0-content-items-one-of-2-all-of-1"></span><span id="body-messages-all-of-0-content-one-of-1-items-one-of-2-all-of-1"></span><span id="body-messages-all-of-0-content-one-of-1-one-of-2-all-of-1"></span><span id="body-messages-all-of-0-content-one-of-2-all-of-1"></span><span id="body-messages-content-items-one-of-2-all-of-1"></span><span id="body-messages-content-one-of-1-items-one-of-2-all-of-1"></span><span id="body-messages-content-one-of-1-one-of-2-all-of-1"></span><span id="body-messages-content-one-of-2-all-of-1"></span><span id="body-messages-items-all-of-0-content-items-one-of-2-all-of-1"></span><span id="body-messages-items-all-of-0-content-one-of-1-items-one-of-2-all-of-1"></span><span id="body-messages-items-all-of-0-content-one-of-1-one-of-2-all-of-1"></span><span id="body-messages-items-all-of-0-content-one-of-2-all-of-1"></span><span id="body-messages-items-content-items-one-of-2-all-of-1"></span><span id="body-messages-items-content-one-of-1-items-one-of-2-all-of-1"></span><span id="body-messages-items-content-one-of-1-one-of-2-all-of-1"></span><span id="body-messages-items-content-one-of-2-all-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-items-one-of-2-all-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-items-one-of-2-all-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-1-one-of-2-all-of-1"></span><span id="body-messages-items-one-of-2-all-of-0-content-one-of-2-all-of-1"></span><span id="body-messages-items-one-of-2-content-items-one-of-2-all-of-1"></span><span id="body-messages-items-one-of-2-content-one-of-1-items-one-of-2-all-of-1"></span><span id="body-messages-items-one-of-2-content-one-of-1-one-of-2-all-of-1"></span><span id="body-messages-items-one-of-2-content-one-of-2-all-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-items-one-of-2-all-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-items-one-of-2-all-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-1-one-of-2-all-of-1"></span><span id="body-messages-one-of-2-all-of-0-content-one-of-2-all-of-1"></span><span id="body-messages-one-of-2-content-items-one-of-2-all-of-1"></span><span id="body-messages-one-of-2-content-one-of-1-items-one-of-2-all-of-1"></span><span id="body-messages-one-of-2-content-one-of-1-one-of-2-all-of-1"></span><span id="body-messages-one-of-2-content-one-of-2-all-of-1"></span>            - type: object
              required:
                - type
              properties:
                type:
                  type: string
                  enum:
                    - input_audio
      description: A content part in a user message (text, image, or audio).
    ChatCompletionRequestAssistantMessageContentPart:
      oneOf:
<span id="body-messages-all-of-0-content-one-of-1-one-of-1-items-one-of-0"></span><span id="body-messages-all-of-0-content-one-of-1-one-of-1-one-of-0"></span><span id="body-messages-content-one-of-1-one-of-1-items-one-of-0"></span><span id="body-messages-content-one-of-1-one-of-1-one-of-0"></span><span id="body-messages-items-all-of-0-content-one-of-1-one-of-1-items-one-of-0"></span><span id="body-messages-items-all-of-0-content-one-of-1-one-of-1-one-of-0"></span><span id="body-messages-items-content-one-of-1-one-of-1-items-one-of-0"></span><span id="body-messages-items-content-one-of-1-one-of-1-one-of-0"></span><span id="body-messages-items-one-of-3-all-of-0-content-items-one-of-0"></span><span id="body-messages-items-one-of-3-all-of-0-content-one-of-1-items-one-of-0"></span><span id="body-messages-items-one-of-3-all-of-0-content-one-of-1-one-of-1-items-one-of-0"></span><span id="body-messages-items-one-of-3-all-of-0-content-one-of-1-one-of-1-one-of-0"></span><span id="body-messages-items-one-of-3-content-items-one-of-0"></span><span id="body-messages-items-one-of-3-content-one-of-1-items-one-of-0"></span><span id="body-messages-items-one-of-3-content-one-of-1-one-of-1-items-one-of-0"></span><span id="body-messages-items-one-of-3-content-one-of-1-one-of-1-one-of-0"></span><span id="body-messages-one-of-3-all-of-0-content-items-one-of-0"></span><span id="body-messages-one-of-3-all-of-0-content-one-of-1-items-one-of-0"></span><span id="body-messages-one-of-3-all-of-0-content-one-of-1-one-of-1-items-one-of-0"></span><span id="body-messages-one-of-3-all-of-0-content-one-of-1-one-of-1-one-of-0"></span><span id="body-messages-one-of-3-content-items-one-of-0"></span><span id="body-messages-one-of-3-content-one-of-1-items-one-of-0"></span><span id="body-messages-one-of-3-content-one-of-1-one-of-1-items-one-of-0"></span><span id="body-messages-one-of-3-content-one-of-1-one-of-1-one-of-0"></span>        - allOf:
            - $ref: '#/components/schemas/ChatCompletionRequestMessageContentPartText'
            - type: object
              required:
                - type
              properties:
                type:
                  type: string
                  enum:
                    - text
<span id="body-messages-all-of-0-content-one-of-1-one-of-1-items-one-of-1"></span><span id="body-messages-all-of-0-content-one-of-1-one-of-1-one-of-1"></span><span id="body-messages-content-one-of-1-one-of-1-items-one-of-1"></span><span id="body-messages-content-one-of-1-one-of-1-one-of-1"></span><span id="body-messages-items-all-of-0-content-one-of-1-one-of-1-items-one-of-1"></span><span id="body-messages-items-all-of-0-content-one-of-1-one-of-1-one-of-1"></span><span id="body-messages-items-content-one-of-1-one-of-1-items-one-of-1"></span><span id="body-messages-items-content-one-of-1-one-of-1-one-of-1"></span><span id="body-messages-items-one-of-3-all-of-0-content-items-one-of-1"></span><span id="body-messages-items-one-of-3-all-of-0-content-one-of-1-items-one-of-1"></span><span id="body-messages-items-one-of-3-all-of-0-content-one-of-1-one-of-1-items-one-of-1"></span><span id="body-messages-items-one-of-3-all-of-0-content-one-of-1-one-of-1-one-of-1"></span><span id="body-messages-items-one-of-3-content-items-one-of-1"></span><span id="body-messages-items-one-of-3-content-one-of-1-items-one-of-1"></span><span id="body-messages-items-one-of-3-content-one-of-1-one-of-1-items-one-of-1"></span><span id="body-messages-items-one-of-3-content-one-of-1-one-of-1-one-of-1"></span><span id="body-messages-one-of-3-all-of-0-content-items-one-of-1"></span><span id="body-messages-one-of-3-all-of-0-content-one-of-1-items-one-of-1"></span><span id="body-messages-one-of-3-all-of-0-content-one-of-1-one-of-1-items-one-of-1"></span><span id="body-messages-one-of-3-all-of-0-content-one-of-1-one-of-1-one-of-1"></span><span id="body-messages-one-of-3-content-items-one-of-1"></span><span id="body-messages-one-of-3-content-one-of-1-items-one-of-1"></span><span id="body-messages-one-of-3-content-one-of-1-one-of-1-items-one-of-1"></span><span id="body-messages-one-of-3-content-one-of-1-one-of-1-one-of-1"></span>        - allOf:
            - $ref: &gt;-
                #/components/schemas/ChatCompletionRequestMessageContentPartRefusal
            - type: object
              required:
                - type
              properties:
                type:
                  type: string
                  enum:
                    - refusal
      description: A content part in an assistant message (text or refusal).
    ChatCompletionRequestToolMessageContentPart:
      oneOf:
<span id="body-messages-items-one-of-4-all-of-0-content-items-one-of-0"></span><span id="body-messages-items-one-of-4-all-of-0-content-one-of-1-items-one-of-0"></span><span id="body-messages-items-one-of-4-all-of-0-content-one-of-1-one-of-0"></span><span id="body-messages-items-one-of-4-content-items-one-of-0"></span><span id="body-messages-items-one-of-4-content-one-of-1-items-one-of-0"></span><span id="body-messages-items-one-of-4-content-one-of-1-one-of-0"></span><span id="body-messages-one-of-4-all-of-0-content-items-one-of-0"></span><span id="body-messages-one-of-4-all-of-0-content-one-of-1-items-one-of-0"></span><span id="body-messages-one-of-4-all-of-0-content-one-of-1-one-of-0"></span><span id="body-messages-one-of-4-content-items-one-of-0"></span><span id="body-messages-one-of-4-content-one-of-1-items-one-of-0"></span><span id="body-messages-one-of-4-content-one-of-1-one-of-0"></span>        - allOf:
<span id="body-messages-items-one-of-4-all-of-0-content-all-of-0"></span><span id="body-messages-items-one-of-4-all-of-0-content-items-all-of-0"></span><span id="body-messages-items-one-of-4-all-of-0-content-items-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-4-all-of-0-content-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-4-all-of-0-content-one-of-1-all-of-0"></span><span id="body-messages-items-one-of-4-all-of-0-content-one-of-1-items-all-of-0"></span><span id="body-messages-items-one-of-4-all-of-0-content-one-of-1-items-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-4-all-of-0-content-one-of-1-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-4-content-all-of-0"></span><span id="body-messages-items-one-of-4-content-items-all-of-0"></span><span id="body-messages-items-one-of-4-content-items-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-4-content-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-4-content-one-of-1-all-of-0"></span><span id="body-messages-items-one-of-4-content-one-of-1-items-all-of-0"></span><span id="body-messages-items-one-of-4-content-one-of-1-items-one-of-0-all-of-0"></span><span id="body-messages-items-one-of-4-content-one-of-1-one-of-0-all-of-0"></span><span id="body-messages-one-of-4-all-of-0-content-all-of-0"></span><span id="body-messages-one-of-4-all-of-0-content-items-all-of-0"></span><span id="body-messages-one-of-4-all-of-0-content-items-one-of-0-all-of-0"></span><span id="body-messages-one-of-4-all-of-0-content-one-of-0-all-of-0"></span><span id="body-messages-one-of-4-all-of-0-content-one-of-1-all-of-0"></span><span id="body-messages-one-of-4-all-of-0-content-one-of-1-items-all-of-0"></span><span id="body-messages-one-of-4-all-of-0-content-one-of-1-items-one-of-0-all-of-0"></span><span id="body-messages-one-of-4-all-of-0-content-one-of-1-one-of-0-all-of-0"></span><span id="body-messages-one-of-4-content-all-of-0"></span><span id="body-messages-one-of-4-content-items-all-of-0"></span><span id="body-messages-one-of-4-content-items-one-of-0-all-of-0"></span><span id="body-messages-one-of-4-content-one-of-0-all-of-0"></span><span id="body-messages-one-of-4-content-one-of-1-all-of-0"></span><span id="body-messages-one-of-4-content-one-of-1-items-all-of-0"></span><span id="body-messages-one-of-4-content-one-of-1-items-one-of-0-all-of-0"></span><span id="body-messages-one-of-4-content-one-of-1-one-of-0-all-of-0"></span>            - $ref: '#/components/schemas/ChatCompletionRequestMessageContentPartText'
<span id="body-messages-items-one-of-4-all-of-0-content-all-of-1"></span><span id="body-messages-items-one-of-4-all-of-0-content-items-all-of-1"></span><span id="body-messages-items-one-of-4-all-of-0-content-items-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-4-all-of-0-content-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-4-all-of-0-content-one-of-1-all-of-1"></span><span id="body-messages-items-one-of-4-all-of-0-content-one-of-1-items-all-of-1"></span><span id="body-messages-items-one-of-4-all-of-0-content-one-of-1-items-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-4-all-of-0-content-one-of-1-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-4-content-all-of-1"></span><span id="body-messages-items-one-of-4-content-items-all-of-1"></span><span id="body-messages-items-one-of-4-content-items-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-4-content-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-4-content-one-of-1-all-of-1"></span><span id="body-messages-items-one-of-4-content-one-of-1-items-all-of-1"></span><span id="body-messages-items-one-of-4-content-one-of-1-items-one-of-0-all-of-1"></span><span id="body-messages-items-one-of-4-content-one-of-1-one-of-0-all-of-1"></span><span id="body-messages-one-of-4-all-of-0-content-all-of-1"></span><span id="body-messages-one-of-4-all-of-0-content-items-all-of-1"></span><span id="body-messages-one-of-4-all-of-0-content-items-one-of-0-all-of-1"></span><span id="body-messages-one-of-4-all-of-0-content-one-of-0-all-of-1"></span><span id="body-messages-one-of-4-all-of-0-content-one-of-1-all-of-1"></span><span id="body-messages-one-of-4-all-of-0-content-one-of-1-items-all-of-1"></span><span id="body-messages-one-of-4-all-of-0-content-one-of-1-items-one-of-0-all-of-1"></span><span id="body-messages-one-of-4-all-of-0-content-one-of-1-one-of-0-all-of-1"></span><span id="body-messages-one-of-4-content-all-of-1"></span><span id="body-messages-one-of-4-content-items-all-of-1"></span><span id="body-messages-one-of-4-content-items-one-of-0-all-of-1"></span><span id="body-messages-one-of-4-content-one-of-0-all-of-1"></span><span id="body-messages-one-of-4-content-one-of-1-all-of-1"></span><span id="body-messages-one-of-4-content-one-of-1-items-all-of-1"></span><span id="body-messages-one-of-4-content-one-of-1-items-one-of-0-all-of-1"></span><span id="body-messages-one-of-4-content-one-of-1-one-of-0-all-of-1"></span>            - type: object
              required:
                - type
              properties:
                type:
                  type: string
                  enum:
                    - text
      description: A content part in a tool message (text only).
    TopLogprobs:
      type: object
      description: A most likely token and its log probability at a given position.
      required:
        - token
        - logprob
      properties:
<span id="response-choices-items-logprobs-content-items-top-logprobs-items-token"></span><span id="response-choices-items-logprobs-content-items-top-logprobs-token"></span><span id="response-choices-items-logprobs-content-top-logprobs-items-token"></span><span id="response-choices-items-logprobs-content-top-logprobs-token"></span><span id="response-choices-items-logprobs-one-of-1-content-items-top-logprobs-items-token"></span><span id="response-choices-items-logprobs-one-of-1-content-items-top-logprobs-token"></span><span id="response-choices-items-logprobs-one-of-1-content-top-logprobs-items-token"></span><span id="response-choices-items-logprobs-one-of-1-content-top-logprobs-token"></span><span id="response-choices-items-logprobs-one-of-1-refusal-items-top-logprobs-items-token"></span><span id="response-choices-items-logprobs-one-of-1-refusal-items-top-logprobs-token"></span><span id="response-choices-items-logprobs-one-of-1-refusal-top-logprobs-items-token"></span><span id="response-choices-items-logprobs-one-of-1-refusal-top-logprobs-token"></span><span id="response-choices-items-logprobs-refusal-items-top-logprobs-items-token"></span><span id="response-choices-items-logprobs-refusal-items-top-logprobs-token"></span><span id="response-choices-items-logprobs-refusal-top-logprobs-items-token"></span><span id="response-choices-items-logprobs-refusal-top-logprobs-token"></span><span id="response-choices-logprobs-content-items-top-logprobs-items-token"></span><span id="response-choices-logprobs-content-items-top-logprobs-token"></span><span id="response-choices-logprobs-content-top-logprobs-items-token"></span><span id="response-choices-logprobs-content-top-logprobs-token"></span><span id="response-choices-logprobs-one-of-1-content-items-top-logprobs-items-token"></span><span id="response-choices-logprobs-one-of-1-content-items-top-logprobs-token"></span><span id="response-choices-logprobs-one-of-1-content-top-logprobs-items-token"></span><span id="response-choices-logprobs-one-of-1-content-top-logprobs-token"></span><span id="response-choices-logprobs-one-of-1-refusal-items-top-logprobs-items-token"></span><span id="response-choices-logprobs-one-of-1-refusal-items-top-logprobs-token"></span><span id="response-choices-logprobs-one-of-1-refusal-top-logprobs-items-token"></span><span id="response-choices-logprobs-one-of-1-refusal-top-logprobs-token"></span><span id="response-choices-logprobs-refusal-items-top-logprobs-items-token"></span><span id="response-choices-logprobs-refusal-items-top-logprobs-token"></span><span id="response-choices-logprobs-refusal-top-logprobs-items-token"></span><span id="response-choices-logprobs-refusal-top-logprobs-token"></span>        token:
          type: string
          description: The token.
<span id="response-choices-items-logprobs-content-items-top-logprobs-items-logprob"></span><span id="response-choices-items-logprobs-content-items-top-logprobs-logprob"></span><span id="response-choices-items-logprobs-content-top-logprobs-items-logprob"></span><span id="response-choices-items-logprobs-content-top-logprobs-logprob"></span><span id="response-choices-items-logprobs-one-of-1-content-items-top-logprobs-items-logprob"></span><span id="response-choices-items-logprobs-one-of-1-content-items-top-logprobs-logprob"></span><span id="response-choices-items-logprobs-one-of-1-content-top-logprobs-items-logprob"></span><span id="response-choices-items-logprobs-one-of-1-content-top-logprobs-logprob"></span><span id="response-choices-items-logprobs-one-of-1-refusal-items-top-logprobs-items-logprob"></span><span id="response-choices-items-logprobs-one-of-1-refusal-items-top-logprobs-logprob"></span><span id="response-choices-items-logprobs-one-of-1-refusal-top-logprobs-items-logprob"></span><span id="response-choices-items-logprobs-one-of-1-refusal-top-logprobs-logprob"></span><span id="response-choices-items-logprobs-refusal-items-top-logprobs-items-logprob"></span><span id="response-choices-items-logprobs-refusal-items-top-logprobs-logprob"></span><span id="response-choices-items-logprobs-refusal-top-logprobs-items-logprob"></span><span id="response-choices-items-logprobs-refusal-top-logprobs-logprob"></span><span id="response-choices-logprobs-content-items-top-logprobs-items-logprob"></span><span id="response-choices-logprobs-content-items-top-logprobs-logprob"></span><span id="response-choices-logprobs-content-top-logprobs-items-logprob"></span><span id="response-choices-logprobs-content-top-logprobs-logprob"></span><span id="response-choices-logprobs-one-of-1-content-items-top-logprobs-items-logprob"></span><span id="response-choices-logprobs-one-of-1-content-items-top-logprobs-logprob"></span><span id="response-choices-logprobs-one-of-1-content-top-logprobs-items-logprob"></span><span id="response-choices-logprobs-one-of-1-content-top-logprobs-logprob"></span><span id="response-choices-logprobs-one-of-1-refusal-items-top-logprobs-items-logprob"></span><span id="response-choices-logprobs-one-of-1-refusal-items-top-logprobs-logprob"></span><span id="response-choices-logprobs-one-of-1-refusal-top-logprobs-items-logprob"></span><span id="response-choices-logprobs-one-of-1-refusal-top-logprobs-logprob"></span><span id="response-choices-logprobs-refusal-items-top-logprobs-items-logprob"></span><span id="response-choices-logprobs-refusal-items-top-logprobs-logprob"></span><span id="response-choices-logprobs-refusal-top-logprobs-items-logprob"></span><span id="response-choices-logprobs-refusal-top-logprobs-logprob"></span>        logprob:
          type: number
          format: float
          description: The log probability of this token.
<span id="response-choices-items-logprobs-content-items-top-logprobs-bytes"></span><span id="response-choices-items-logprobs-content-items-top-logprobs-items-bytes"></span><span id="response-choices-items-logprobs-content-top-logprobs-bytes"></span><span id="response-choices-items-logprobs-content-top-logprobs-items-bytes"></span><span id="response-choices-items-logprobs-one-of-1-content-items-top-logprobs-bytes"></span><span id="response-choices-items-logprobs-one-of-1-content-items-top-logprobs-items-bytes"></span><span id="response-choices-items-logprobs-one-of-1-content-top-logprobs-bytes"></span><span id="response-choices-items-logprobs-one-of-1-content-top-logprobs-items-bytes"></span><span id="response-choices-items-logprobs-one-of-1-refusal-items-top-logprobs-bytes"></span><span id="response-choices-items-logprobs-one-of-1-refusal-items-top-logprobs-items-bytes"></span><span id="response-choices-items-logprobs-one-of-1-refusal-top-logprobs-bytes"></span><span id="response-choices-items-logprobs-one-of-1-refusal-top-logprobs-items-bytes"></span><span id="response-choices-items-logprobs-refusal-items-top-logprobs-bytes"></span><span id="response-choices-items-logprobs-refusal-items-top-logprobs-items-bytes"></span><span id="response-choices-items-logprobs-refusal-top-logprobs-bytes"></span><span id="response-choices-items-logprobs-refusal-top-logprobs-items-bytes"></span><span id="response-choices-logprobs-content-items-top-logprobs-bytes"></span><span id="response-choices-logprobs-content-items-top-logprobs-items-bytes"></span><span id="response-choices-logprobs-content-top-logprobs-bytes"></span><span id="response-choices-logprobs-content-top-logprobs-items-bytes"></span><span id="response-choices-logprobs-one-of-1-content-items-top-logprobs-bytes"></span><span id="response-choices-logprobs-one-of-1-content-items-top-logprobs-items-bytes"></span><span id="response-choices-logprobs-one-of-1-content-top-logprobs-bytes"></span><span id="response-choices-logprobs-one-of-1-content-top-logprobs-items-bytes"></span><span id="response-choices-logprobs-one-of-1-refusal-items-top-logprobs-bytes"></span><span id="response-choices-logprobs-one-of-1-refusal-items-top-logprobs-items-bytes"></span><span id="response-choices-logprobs-one-of-1-refusal-top-logprobs-bytes"></span><span id="response-choices-logprobs-one-of-1-refusal-top-logprobs-items-bytes"></span><span id="response-choices-logprobs-refusal-items-top-logprobs-bytes"></span><span id="response-choices-logprobs-refusal-items-top-logprobs-items-bytes"></span><span id="response-choices-logprobs-refusal-top-logprobs-bytes"></span><span id="response-choices-logprobs-refusal-top-logprobs-items-bytes"></span>        bytes:
          type:
            - array
            - 'null'
          items:
            type: integer
            format: u-int8
            minimum: 0
          description: &gt;-
            A list of integers representing the UTF-8 bytes representation of
            the token. Useful in instances where characters are represented by
            multiple tokens and their byte representations must be combined to
            generate the correct text representation. Can be `null` if there is
            no bytes representation for the token.
    ChatCompletionRequestMessageContentPartImage:
      type: object
      description: An image content part of a message.
      required:
        - image_url
      properties:
        image_url:
          $ref: '#/components/schemas/ImageUrl'
    ChatCompletionRequestMessageContentPartAudio:
      type: object
      description: &gt;-
        Learn about [audio
        inputs](https://platform.openai.com/docs/guides/audio).
      required:
        - input_audio
      properties:
        input_audio:
          $ref: '#/components/schemas/InputAudio'
    ChatCompletionRequestMessageContentPartRefusal:
      type: object
      description: A refusal content part of a message.
      required:
        - refusal
      properties:
        refusal:
          type: string
          description: The refusal message generated by the model.
    ImageUrl:
      type: object
      description: An image URL with optional detail level.
      required:
        - url
      properties:
        url:
          type: string
          description: Either a URL of the image or the base64 encoded image data.
        detail:
          oneOf:
            - type: 'null'
            - $ref: '#/components/schemas/ImageDetail'
              description: &gt;-
                Specifies the detail level of the image. Learn more in the
                [Vision
                guide](https://platform.openai.com/docs/guides/vision/low-or-high-fidelity-image-understanding).
    InputAudio:
      type: object
      description: Input audio data for an audio content part.
      required:
        - data
        - format
      properties:
        data:
          type: string
          description: Base64 encoded audio data.
        format:
          $ref: '#/components/schemas/InputAudioFormat'
          description: &gt;-
            The format of the encoded audio data. Currently supports "wav" and
            "mp3".
    ImageDetail:
      type: string
      description: Specifies the detail level of the image.
      enum:
        - auto
        - low
        - high
    InputAudioFormat:
      type: string
      description: The format of the encoded audio data.
      enum:
        - wav
        - mp3
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