# Sync data to Meilisearch with Kestra

> Keep Meilisearch in sync with your data sources using declarative Kestra workflows.

[Kestra](https://kestra.io) is an open-source orchestration platform that runs data pipelines from declarative YAML. The [Kestra Meilisearch plugin](https://kestra.io/plugins/plugin-meilisearch) lets you index documents into Meilisearch as a first-class workflow task, so keeping your search index current becomes an observable, retryable, scheduled job instead of a script you have to babysit.

## Why orchestrate the sync

A cron job calling a custom script works right up until it doesn't: a network blip leaves your index half-updated, nobody notices the job died days ago, and there is no record of what synced when. Kestra makes the sync a first-class workflow. Every run is logged, retried on failure, observable in a UI, and triggered by a schedule or an event. You describe what should happen, and Kestra handles execution, state, and failure.

## How the pipelines work

Most guides in this series follow the same arc: a one-shot **backfill** for the initial load, then an **incremental sync** that keeps Meilisearch current as data is inserted, updated, and deleted. The Elasticsearch and OpenSearch guides are migrations, pairing the backfill with a safe parallel-run cutover. Three ideas recur throughout:

* **ION is the handoff format.** Every Kestra extractor writes [ION](https://amazon-ion.github.io/ion-docs/) to internal storage, and the Meilisearch `DocumentAdd` task reads exactly that. The pipelines are just "extract, convert, index" with no glue code.
* **Upserts are idempotent.** `DocumentAdd` is add-or-replace, so overlapping sync windows and accidental re-runs are always safe.
* **Deletes need explicit handling.** `DocumentAdd` never removes documents. Each guide covers the delete strategy appropriate to its source (soft deletes plus `documents/delete-batch`, tombstone events, or an index alias swap).

> [!NOTE]
>
> Store credentials as Kestra secrets (`{{ secret('NAME') }}`), never as plaintext. Inline values in these guides are for readability only.

## Guides

Each guide targets a [Meilisearch Cloud](https://www.meilisearch.com/cloud?utm_campaign=oss\&utm_source=docs\&utm_medium=kestra-integration) project (fully managed, nothing to host). You supply your project URL and Default Admin API key.

> [card] [PostgreSQL](https://www.meilisearch.com/docs/getting_started/integrations/kestra/postgresql)
>
> Relational database. Scheduled lookback window on `updated_at`, soft deletes via delete-batch.

> [card] [MongoDB](https://www.meilisearch.com/docs/getting_started/integrations/kestra/mongodb)
>
> Document database. Scheduled lookback on an `updatedAt` ISO string, soft deletes via delete-batch.

> [card] [Amazon S3](https://www.meilisearch.com/docs/getting_started/integrations/kestra/amazon_s3)
>
> Object storage. Event-driven trigger on a prefix with exactly-once processing.

> [card] [Kafka](https://www.meilisearch.com/docs/getting_started/integrations/kestra/kafka)
>
> Event stream. Real-time trigger, one execution per message.

> [card] [REST API](https://www.meilisearch.com/docs/getting_started/integrations/kestra/rest_api)
>
> HTTP API. A `?updated_since` filter with a KV watermark, or a scheduled full re-index.

> [card] [Elasticsearch](https://www.meilisearch.com/docs/getting_started/integrations/kestra/elasticsearch)
>
> Search engine migration. Scroll the whole index, then cut over with a parallel run.

> [card] [OpenSearch](https://www.meilisearch.com/docs/getting_started/integrations/kestra/opensearch)
>
> Search engine migration. Scroll the whole index, then cut over with a parallel run.

> [card] [RabbitMQ](https://www.meilisearch.com/docs/getting_started/integrations/kestra/rabbitmq)
>
> AMQP queue. Real-time trigger, one execution per message, delete events routed to delete-batch.

> [card] [Shopify](https://www.meilisearch.com/docs/getting_started/integrations/kestra/shopify)
>
> E-commerce platform. Native `updatedAtMin` incremental sync, webhooks for real-time and deletes.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.