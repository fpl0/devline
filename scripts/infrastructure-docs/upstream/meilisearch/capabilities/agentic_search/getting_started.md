# Getting started with agentic search

> Connect Meilisearch to an AI agent using the AI SDK, then run your first agentic search request.

This guide walks you through building agentic search using [`@meilisearch/ai-sdk`](https://github.com/meilisearch/ai-sdk) and the [Vercel AI SDK](https://ai-sdk.dev). Using this approach, you can provide the LLM with search tools, allowing the model full control over when to search and how to use the results.

> [!TIP]
>
> To get something working fast, you can connect your agent to the [Meilisearch MCP server](https://www.meilisearch.com/docs/getting_started/integrations/mcp) instead.

## Before you begin

You need:

* A Meilisearch project with at least one index containing documents. If you don't have one yet, follow the [quick start](https://www.meilisearch.com/docs/getting_started/first_project).
* A search API key for that project. See [manage API keys](https://www.meilisearch.com/docs/capabilities/security/how_to/manage_api_keys) if you need to create one.
* An API key from an LLM provider. This guide uses OpenAI, but you can use any supported by the AI SDK.

## Install packages

Install the AI SDK, a model provider package, and the Meilisearch AI SDK:

```bash
npm install ai @ai-sdk/openai @meilisearch/ai-sdk
```

## Configure environment variables

Add your Meilisearch and LLM provider credentials to your environment:

```bash
MEILISEARCH_URL=https://your-project.meilisearch.io
MEILISEARCH_KEY=your-search-api-key
OPENAI_API_KEY=your-openai-api-key
```

## Run a first agentic request

Use `meilisearchSearch` to give an agent a search tool scoped to one index. The `description` field tells the model what the index contains and when to use it:

```ts
import { generateText, stepCountIs } from "ai";
import { openai } from "@ai-sdk/openai";
import { meilisearchSearch } from "@meilisearch/ai-sdk";

const { text } = await generateText({
  model: openai("gpt-5.4-mini"),
  prompt: "Find two movies about time travel, then summarize the difference.",
  tools: {
    search: meilisearchSearch({
      host: "MEILISEARCH_URL", // or process.env.MEILISEARCH_URL,
      apiKey: "MEILISEARCH_KEY", // or process.env.MEILISEARCH_KEY,
      indexUid: "movies",
      description: "Search the movies database",
    }),
  },
  stopWhen: stepCountIs(5),
});

console.log(text);
```

The model decides whether and how many times to call `search` before answering. You can steer behavior by [prompt engineering](https://www.meilisearch.com/docs/capabilities/agentic_search/how_to/optimize_chat_prompts), [adding guardrails](https://www.meilisearch.com/docs/capabilities/agentic_search/how_to/configure_guardrails), and using the AI SDK to configure your agent's capabilities.

## Building a chatbot

Chatbots are a common UX on top of agentic search. If you want a multi-turn interface, keep the message history yourself and use `streamText` to stream the response:

```ts
import { streamText, stepCountIs } from "ai";
import { openai } from "@ai-sdk/openai";
import { meilisearchSearch } from "@meilisearch/ai-sdk";

const search = meilisearchSearch({
  host: "MEILISEARCH_URL", // or process.env.MEILISEARCH_URL,
  apiKey: "MEILISEARCH_KEY", // or process.env.MEILISEARCH_KEY,
  indexUid: "movies",
  description: "Search the movies database",
});

const messages = [
  { role: "user", content: "Find two movies about time travel." },
];

const { textStream } = streamText({
  model: openai("gpt-5.4-mini"),
  messages,
  tools: { search },
  stopWhen: stepCountIs(5),
});

for await (const text of textStream) {
  process.stdout.write(text);
}
```

For a complete implementation, check out the example application on GitHub.

> [card] [AI SDK demo](https://github.com/meilisearch/ai-sdk-demo)
>
> Next.js chatbox example application

## Improving retrieval performance

The `@meilisearch/ai-sdk` provides tools that build upon Meilisearch capabilities. To improve the relevancy of your search results, apply the best practices for tuning your [full-text search](https://www.meilisearch.com/docs/capabilities/full_text_search/overview) and [hybrid search](https://www.meilisearch.com/docs/capabilities/hybrid_search/overview).

Since agentic search puts the model in control of the search process, you may find that [semantic search](https://www.meilisearch.com/docs/capabilities/hybrid_search/advanced/semantic_vs_hybrid) is a better starting point than full-text search. Still, your choice of model, how you configure [document embeddings](https://www.meilisearch.com/docs/capabilities/hybrid_search/advanced/document_template_best_practices), and your tool configuration will all impact the relevancy of results. We recommend using evals when iterating on your retrieval pipeline.

For a full list of available tools, see the [SDK API reference](https://github.com/meilisearch/ai-sdk). For advanced retrieval logic, you can build your own agent tools on top of the [`meilisearch-js`](https://www.meilisearch.com/docs/getting_started/sdks/javascript) SDK.

## Next steps

> [card] [Choose an embedder](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/choose_an_embedder)
>
> Configure hybrid search for better natural-language retrieval.

> [card] [Manage API keys](https://www.meilisearch.com/docs/capabilities/security/how_to/manage_api_keys)
>
> Scope search API keys to the indexes your agent should access.

> [card] [Display source documents](https://www.meilisearch.com/docs/capabilities/agentic_search/how_to/display_source_documents)
>
> Show users which documents an agent used to answer.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.