# Boost documents during a campaign

> Rank a selection of documents higher for a limited period, without pinning them to fixed positions.

Your summer sale runs from June 1 to June 30. During that month, you want every product on sale to rank higher than the rest, whatever users type. Pinning is too rigid here: you do not know in advance which products users will search for, and you do not want a fixed list at the top of every result page. Boosting keeps the results organic and only pushes the selected products up.

One rule covers the whole campaign: a time condition for the period, and a boost action for the products on sale.

## Set up from the Meilisearch Cloud dashboard

Open your project in the [Meilisearch Cloud dashboard](https://cloud.meilisearch.com), select the **Search rules** tab, and click **New rule**. Give the rule an ID, such as `summer-sale`, and a description.

### Boost the products on sale

In the **Actions** block, click **Add action**, select **Boost documents**, then click **Next**. Choose how to select the products: with a filter, or document by document. A filter fits here, because you want every product on sale, whichever they are at that moment.

![Dialog asking whether to select documents with a filter or by hand](https://mintcdn.com/meilisearch-6b28dec2/pjgozmedxQwKwpe_/assets/images/cloud-search-rules/select-documents-step.png?fit=max\&auto=format\&n=pjgozmedxQwKwpe_\&q=85\&s=4bd97aa4ab7d94b72404b0c183dbe3e4)

Pick **Use a filter**, then fill in the dialog:

* **Index**: the index holding the products, here `products`.
* **Filter**: the condition selecting them, here `on_sale = true`. The attribute you filter on must be in the [`filterableAttributes` setting](https://www.meilisearch.com/docs/reference/api/settings/get-filterableattributes) of the index, otherwise the action selects no document.
* **Strength**: how high the selected products climb. Start with the medium value, then run a few searches of your own and adjust: the right strength depends on your data and on the way your users search.

The dialog tells you how many documents the action affects, so you can check your filter before saving. Click **Add action**.

![Boost documents dialog with the index, the strength, and the filter](https://mintcdn.com/meilisearch-6b28dec2/pjgozmedxQwKwpe_/assets/images/cloud-search-rules/boost-action-dialog.png?fit=max\&auto=format\&n=pjgozmedxQwKwpe_\&q=85\&s=fc8afeb53846ff4d3352b145785da399)

### Limit the rule to the campaign

In the **Conditions** block, set **Time** to **Apply within time range**, then enter the start and the end of your campaign. Times are in UTC, so convert your local window before entering it.

![Actions block with the boost action, and the time range condition](https://mintcdn.com/meilisearch-6b28dec2/pjgozmedxQwKwpe_/assets/images/cloud-search-rules/boost-rule-overview.png?fit=max\&auto=format\&n=pjgozmedxQwKwpe_\&q=85\&s=51736da717ce35157012d594cf23a591)

Click **Create rule**. Meilisearch indexes the rule, which can take a few seconds to a few minutes. Once it is live, searches keep their usual results during the campaign, with the products on sale higher in the list.

## Set up on a self-hosted instance

> [!WARNING]
>
> Search rules must be turned on before you call the `/dynamic-search-rules` routes, otherwise Meilisearch returns an error. See [Create your first rule](https://www.meilisearch.com/docs/capabilities/search_rules/create_your_first_rule#turn-on-search-rules-1).

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/dynamic-search-rules/summer-sale' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "description": "Boost the products on sale during the summer sale",
    "active": true,
    "conditions": {
      "time": {
        "start": "2026-06-01T00:00:00Z",
        "end": "2026-06-30T23:59:59Z"
      }
    },
    "actions": {
      "scale": [
        {
          "indexUid": "products",
          "filter": "on_sale = true",
          "weight": 1.5
        }
      ]
    }
  }'
```

* `weight` multiplies the relevancy of the selected documents. Above `1`, the documents rank higher. A product that was tenth can move into the first results, but only if it matches the query.
* `filter` selects every document matching it. The attributes you use must be in the [`filterableAttributes` setting](https://www.meilisearch.com/docs/reference/api/settings/get-filterableattributes) of the index, otherwise the action selects nothing.
* `conditions.time` accepts RFC 3339 timestamps in UTC. Convert your local campaign window before sending it.

Meilisearch answers with a summarized task containing a `taskUid`, because creating a rule is an [asynchronous operation](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations):

```json
{
  "taskUid": 42,
  "indexUid": ".meili_dsr",
  "status": "enqueued",
  "type": "dsrUpdate",
  "enqueuedAt": "2026-05-28T09:25:53.295494760Z"
}
```

The rule applies to searches once this task reaches the `succeeded` status. To follow its progress, pass the `taskUid` to the [task API](https://www.meilisearch.com/docs/reference/api/tasks/get-task).

From then on, searches return the same documents as before, with the products on sale higher in the list. At the end of the campaign, the rule stops on its own and the ranking goes back to normal, with nothing to clean up.

## Next steps

> [card] [Deboost documents](https://www.meilisearch.com/docs/capabilities/search_rules/how_to/deboost_documents)
>
> Keep documents available but lower in the results

> [card] [Hide documents](https://www.meilisearch.com/docs/capabilities/search_rules/how_to/hide_documents)
>
> Remove documents from the results without reindexing

> [card] [Pause a rule](https://www.meilisearch.com/docs/capabilities/search_rules/how_to/pause_a_rule)
>
> Stop a campaign before its end date

> [card] [API reference](https://www.meilisearch.com/docs/reference/api/search-rules/create-or-update-a-search-rule)
>
> Endpoints, rule fields, and update behavior


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.