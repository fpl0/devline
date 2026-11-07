# Pause a rule without deleting it

> Stop a search rule from applying while keeping its configuration, so you can turn it back on later.

A campaign ends earlier than planned, or a rule behaves in a way you did not expect in production. You want it to stop applying right away, but you do not want to rebuild it from memory next week. Pausing a rule keeps every condition and action in place and only stops it from running at search time.

## Pause a rule from the Meilisearch Cloud dashboard

Open your project in the [Meilisearch Cloud dashboard](https://cloud.meilisearch.com) and select the **Search rules** tab. Find your rule in the list, then click the **Active** toggle in its row.

![Search rules list with the Active toggle on each row](https://mintcdn.com/meilisearch-6b28dec2/pjgozmedxQwKwpe_/assets/images/cloud-search-rules/rules-list-with-actions.png?fit=max\&auto=format\&n=pjgozmedxQwKwpe_\&q=85\&s=052fd419dbd20c88b7aec4cebebe4264)

Click the toggle again to run the rule once more. Nothing else changes, so the rule applies exactly as it did before.

## Pause a rule on a self-hosted instance

> [!WARNING]
>
> Search rules must be turned on before you call the `/dynamic-search-rules` routes, otherwise Meilisearch returns an error. See [Create your first rule](https://www.meilisearch.com/docs/capabilities/search_rules/create_your_first_rule#turn-on-search-rules-1).

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/dynamic-search-rules/summer-sale' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "active": false
  }'
```

The `PATCH` route only modifies the fields you send, so your conditions and actions stay untouched. Send `"active": true` to run the rule again.

Meilisearch answers with a summarized task containing a `taskUid`, because updating a rule is an [asynchronous operation](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations):

```json
{
  "taskUid": 45,
  "indexUid": ".meili_dsr",
  "status": "enqueued",
  "type": "dsrUpdate",
  "enqueuedAt": "2026-06-12T14:03:47.665870469Z"
}
```

The rule stops applying to searches once this task reaches the `succeeded` status. To follow its progress, pass the `taskUid` to the [task API](https://www.meilisearch.com/docs/reference/api/tasks/get-task).

## Pause or delete

A paused rule stays in your list, keeps its definition, and is skipped at search time. Deleting a rule with `DELETE /dynamic-search-rules/{uid}` is final: you have to create it again from scratch.

Pause the rules you may need again, such as the campaigns that come back every year. Delete the ones that are wrong or that you will never use again.

## Next steps

> [card] [List or filter existing rules](https://www.meilisearch.com/docs/capabilities/search_rules/how_to/list_and_filter_rules)
>
> Find the rules to pause or to turn back on

> [card] [Boost documents](https://www.meilisearch.com/docs/capabilities/search_rules/how_to/boost_documents)
>
> Rank a selection of documents higher during a campaign

> [card] [Hide documents](https://www.meilisearch.com/docs/capabilities/search_rules/how_to/hide_documents)
>
> Remove documents from the results without reindexing

> [card] [API reference](https://www.meilisearch.com/docs/reference/api/search-rules/create-or-update-a-search-rule)
>
> Full request and response shapes


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.