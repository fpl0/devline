# Hide documents from search results

> Remove a selection of documents from the results without deleting them from your index.

You stop selling a product, but you keep it in your index: order history, direct links, and your analytics still need it. It should simply disappear from search. The same need comes up with archived help articles, or with a document you must take down immediately after a complaint.

Hiding solves this in one rule, and takes effect without reindexing your documents.

## Set up from the Meilisearch Cloud dashboard

Open your project in the [Meilisearch Cloud dashboard](https://cloud.meilisearch.com), select the **Search rules** tab, and click **New rule**. Give the rule an ID, such as `hide-discontinued`, and a description.

In the **Actions** block, click **Add action**, select **Hide documents**, then click **Next**. Pick **Use a filter** to hide a whole set of documents at once, or **Pick documents by hand** for a short list, then fill in the dialog:

* **Index**: the index holding the documents, here `products`.
* **Filter**: the condition selecting them, here `discontinued = true`. The attribute you filter on must be in the [`filterableAttributes` setting](https://www.meilisearch.com/docs/reference/api/settings/get-filterableattributes) of the index, otherwise the action selects no document.

The dialog tells you how many documents disappear from the results. Click **Add action**.

![Hide documents dialog with the index and the filter](https://mintcdn.com/meilisearch-6b28dec2/pjgozmedxQwKwpe_/assets/images/cloud-search-rules/hide-action-dialog.png?fit=max\&auto=format\&n=pjgozmedxQwKwpe_\&q=85\&s=34bfd4e8782bd907fbd768449c8d292c)

Leave the **Conditions** block with its default values, so the rule applies to every search, then click **Create rule**. Meilisearch indexes the rule, which can take a few seconds to a few minutes. Once it is live, the discontinued products no longer appear in any search, and nothing was removed from your index. To bring them back, pause the rule or delete it.

## Set up on a self-hosted instance

> [!WARNING]
>
> Search rules must be turned on before you call the `/dynamic-search-rules` routes, otherwise Meilisearch returns an error. See [Create your first rule](https://www.meilisearch.com/docs/capabilities/search_rules/create_your_first_rule#turn-on-search-rules-1).

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/dynamic-search-rules/hide-discontinued' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "description": "Keep the discontinued products out of the results",
    "active": true,
    "actions": {
      "scale": [
        {
          "indexUid": "products",
          "filter": "discontinued = true",
          "weight": 0
        }
      ]
    }
  }'
```

* Hiding is a `scale` action with a `weight` of `0`.
* Hidden documents leave the results and the facet counts, but they stay in your index. You can still retrieve them with the documents endpoints.
* If another rule pins a hidden document, the document is pinned. See [specific behaviors](https://www.meilisearch.com/docs/capabilities/search_rules/advanced/specific_behaviors).
* The attributes used in `filter` must be in the [`filterableAttributes` setting](https://www.meilisearch.com/docs/reference/api/settings/get-filterableattributes) of the index, otherwise the action selects nothing.

Meilisearch answers with a summarized task containing a `taskUid`, because creating a rule is an [asynchronous operation](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations):

```json
{
  "taskUid": 44,
  "indexUid": ".meili_dsr",
  "status": "enqueued",
  "type": "dsrUpdate",
  "enqueuedAt": "2026-05-28T09:42:08.427773885Z"
}
```

The rule applies to searches once this task reaches the `succeeded` status. To follow its progress, pass the `taskUid` to the [task API](https://www.meilisearch.com/docs/reference/api/tasks/get-task).

From then on, the discontinued products disappear from the results and from the facet counts, for every search and every user, while they stay in your index.

## Next steps

> [card] [Deboost documents](https://www.meilisearch.com/docs/capabilities/search_rules/how_to/deboost_documents)
>
> Lower documents instead of removing them

> [card] [Boost documents](https://www.meilisearch.com/docs/capabilities/search_rules/how_to/boost_documents)
>
> Rank a selection of documents higher

> [card] [Pause a rule](https://www.meilisearch.com/docs/capabilities/search_rules/how_to/pause_a_rule)
>
> Bring the hidden documents back

> [card] [API reference](https://www.meilisearch.com/docs/reference/api/search-rules/create-or-update-a-search-rule)
>
> Endpoints, rule fields, and update behavior


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.