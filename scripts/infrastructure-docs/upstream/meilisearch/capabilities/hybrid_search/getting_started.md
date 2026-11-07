# Getting started with AI-powered search

> Configure an embedding model and perform your first semantic search with Meilisearch. This tutorial uses OpenAI, but Meilisearch supports many providers.

AI-powered search uses **embedding models** to retrieve search results based on the meaning and context of a query, not just matching keywords. Unlike LLMs, embedding models are lightweight, fast, and inexpensive to run.

This tutorial uses OpenAI as the embedding provider because it is the simplest to set up. Meilisearch supports [many other providers](https://www.meilisearch.com/docs/capabilities/hybrid_search/overview#supported-embedder-providers) including Cohere, Mistral, Gemini, Cloudflare, Voyage, AWS Bedrock, and more.

> [!NOTE]
>
> This tutorial requires an [OpenAI API key](https://platform.openai.com/api-keys).

## Create a new index

First, create a new Meilisearch project. If this is your first time using Meilisearch, follow the [quick start](https://www.meilisearch.com/docs/getting_started/first_project) then come back to this tutorial.

Next, create a `kitchenware` index and add [this kitchenware products dataset](https://www.meilisearch.com/docs/assets/datasets/kitchenware.json) to it. It will take Meilisearch a few moments to process your request, but you can continue to the next step while your data is indexing.

## Configure an embedder

In this step, you will configure an OpenAI embedder. Meilisearch uses **embedders** to convert documents and queries into **embeddings**, numerical vectors that capture their semantic meaning. Once configured, Meilisearch generates and caches all embeddings automatically.

Open a blank file in your text editor. You will build your embedder configuration one step at a time.

### Choose an embedder name

In your blank file, create your `embedder` object:

```json
{
  "products-openai": {}
}
```

`products-openai` is the name of your embedder for this tutorial. You can name embedders any way you want, but try to keep it simple, short, and easy to remember.

### Choose an embedder source

Meilisearch relies on third-party embedding models to generate embeddings. These services are referred to as the embedder source.

Add a new `source` field to your embedder object:

```json
{
  "products-openai": {
    "source": "openAi"
  }
}
```

### Choose an embedder model

Embedding models vary in size, cost, and quality. Add a new `model` field to your embedder object:

```json
{
  "products-openai": {
    "source": "openAi",
    "model": "text-embedding-3-small"
  }
}
```

`text-embedding-3-small` is a cost-effective model for general usage. OpenAI also offers `text-embedding-3-large` for higher accuracy at a higher cost.

### Create your API key

Log into OpenAI, or create an account if this is your first time using it. Generate a new API key using [OpenAI's web interface](https://platform.openai.com/api-keys).

Add the `apiKey` field to your embedder:

```json
{
  "products-openai": {
    "source": "openAi",
    "model": "text-embedding-3-small",
    "apiKey": "OPEN_AI_API_KEY"
  }
}
```

Replace `OPEN_AI_API_KEY` with your own API key.

> [!TIP]
>
> You may use any key tier for this tutorial. Use at least [Tier 2 keys](https://platform.openai.com/docs/guides/rate-limits/usage-tiers?context=tier-two) in production environments.

### Design a document template

Documents can be complex objects with many fields. A **document template** tells Meilisearch which fields to include when generating the embedding, using [Liquid](https://shopify.github.io/liquid/basics/introduction/) syntax.

A good template should be short and only include the most relevant information. Add the following `documentTemplate` to your embedder:

```json
{
  "products-openai": {
    "source": "openAi",
    "model": "text-embedding-3-small",
    "apiKey": "OPEN_AI_API_KEY",
    "documentTemplate": "An object used in a kitchen named '{{doc.name}}'"
  }
}
```

This template gives general context (`An object used in a kitchen`) and adds the information specific to each document (`doc.name`, with values like `wooden spoon` or `rolling pin`).

> [!NOTE]
>
> The kitchenware sample dataset only contains three fields: `id`, `name`, and `price`. Since `price` is a numeric value with no semantic meaning, this template uses only `name`. In a production dataset with richer fields—descriptions, categories, materials, or tags—include those fields in your template to improve relevancy.

For more advanced templates, see [document template best practices](https://www.meilisearch.com/docs/capabilities/hybrid_search/advanced/document_template_best_practices).

### Send the configuration to Meilisearch

Your embedder object is ready. Send it to Meilisearch by updating your index settings:

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/kitchenware/settings/embedders' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "products-openai": {
      "source": "openAi",
      "apiKey": "OPEN_AI_API_KEY",
      "model": "text-embedding-3-small",
      "documentTemplate": "An object used in a kitchen named '\''{{doc.name}}'\''"
    }
  }'
```

Replace `MEILISEARCH_URL` with the address of your Meilisearch project, and `OPEN_AI_API_KEY` with your [OpenAI API key](https://platform.openai.com/api-keys).

Meilisearch will automatically batch your documents and send them to OpenAI for embedding generation. Embeddings are cached, so only new or modified documents are processed on subsequent indexing operations.

## Perform a hybrid search

Hybrid searches are very similar to basic text searches. Query the `/search` endpoint with a request containing both the `q` and the `hybrid` parameters:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/kitchenware/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "kitchen utensils made of wood",
    "hybrid": {
      "embedder": "products-openai"
    }
  }'
```

Meilisearch runs both keyword and semantic search, then merges the results using its [smart scoring system](https://www.meilisearch.com/docs/capabilities/hybrid_search/overview#smart-result-ranking). The most relevant results appear first, whether they matched by exact keywords or by meaning.

> [!TIP]
>
> Semantic search can return low-relevancy results when a query has no good match in your index. Use `rankingScoreThreshold` to filter out results below a minimum relevancy score:
>
> ```json theme={null}
> {
>   "q": "plates for hot food",
>   "hybrid": { "semanticRatio": 0.9, "embedder": "products-openai" },
>   "rankingScoreThreshold": 0.5
> }
> ```
>
> Values range from `0.0` (return all results) to `1.0` (return only near-perfect matches). Start around `0.5` and adjust based on your data.

## Next steps

> [card] [Choose an embedder](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/choose_an_embedder)
>
> Compare providers and pick the right one for your use case

> [card] [Custom hybrid ranking](https://www.meilisearch.com/docs/capabilities/hybrid_search/advanced/custom_hybrid_ranking)
>
> Tune semanticRatio to balance keyword and semantic results

### Guides for other embedding providers

This tutorial used OpenAI, but Meilisearch works with many providers. Each guide below walks you through the full configuration:

> [card] [Cohere](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/configure_cohere_embedder)
>
> Cloud-hosted multilingual embeddings

> [card] [Mistral](https://www.meilisearch.com/docs/capabilities/hybrid_search/providers/mistral)
>
> Mistral's embedding API

> [card] [Google Gemini](https://www.meilisearch.com/docs/capabilities/hybrid_search/providers/gemini)
>
> Google's embedding models

> [card] [Cloudflare](https://www.meilisearch.com/docs/capabilities/hybrid_search/providers/cloudflare)
>
> Cloudflare Workers AI embeddings

> [card] [Voyage AI](https://www.meilisearch.com/docs/capabilities/hybrid_search/providers/voyage)
>
> Specialized embedding models

> [card] [Jina](https://www.meilisearch.com/docs/capabilities/hybrid_search/providers/jina)
>
> Multilingual embedding models

> [card] [TwelveLabs](https://www.meilisearch.com/docs/capabilities/hybrid_search/providers/twelvelabs)
>
> Multimodal search across text, images, and video

> [card] [AWS Bedrock](https://www.meilisearch.com/docs/capabilities/hybrid_search/providers/bedrock)
>
> Amazon's embedding service

> [card] [HuggingFace](https://www.meilisearch.com/docs/capabilities/hybrid_search/providers/huggingface)
>
> HuggingFace Inference Endpoints

> [card] [REST embedder](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/configure_rest_embedder)
>
> Connect any embedding API


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.