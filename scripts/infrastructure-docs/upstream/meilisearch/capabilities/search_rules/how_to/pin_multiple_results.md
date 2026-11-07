# Pin several documents for a query

> Control the first positions of your results by combining several pin actions in one rule.

When customers search for "coffee", you know what they should see first: the espresso machine you promote this month, then the mug that goes with it. One pin controls one position, so a rule with several pin actions builds the top of your results in the order you choose. Organic results fill everything below.

## Set up from the Meilisearch Cloud dashboard

Open your project in the [Meilisearch Cloud dashboard](https://cloud.meilisearch.com), select the **Search rules** tab, and click **New rule**. Give the rule an ID, such as `coffee-top-picks`, and a description.

### Pin the first document

In the **Actions** block, click **Add action**, select **Pin a document**, then click **Next**. Choose the **Index**, then search the **Document** by its ID or its content. Set its **Position**, which starts at `1` for the first result, and click **Add action**.

![Pin a document dialog with the index, the document, and the position](https://mintcdn.com/meilisearch-6b28dec2/pjgozmedxQwKwpe_/assets/images/cloud-search-rules/pin-second-document.png?fit=max\&auto=format\&n=pjgozmedxQwKwpe_\&q=85\&s=9f4c59442ba7564a55f283c76a4e5ba4)

### Pin the next ones

Repeat the same steps for every document you want to place, here the coffee mug at position `2`. The **Actions** block numbers your pins and shows which document each one promotes.

Pins do not have to follow each other. Pinning at positions `1` and `5` leaves organic results in between.

### Choose when the rule applies

In the **Conditions** block, set **Query** to `Contains all words`, then enter `coffee`. The rule now applies to every search containing that word, in any order, so it also covers "coffee machine" or "best coffee grinder".

![Actions block with two pins, and the query condition](https://mintcdn.com/meilisearch-6b28dec2/pjgozmedxQwKwpe_/assets/images/cloud-search-rules/pin-rule-overview.png?fit=max\&auto=format\&n=pjgozmedxQwKwpe_\&q=85\&s=d35a03c125773b6231f857751d5649b6)

Click **Create rule**. Meilisearch indexes the rule, which can take a few seconds to a few minutes. Once it is live, a search containing "coffee" starts with your two picks, in the order you chose.

## Set up on a self-hosted instance

> [!WARNING]
>
> Search rules must be turned on before you call the `/dynamic-search-rules` routes, otherwise Meilisearch returns an error. See [Create your first rule](https://www.meilisearch.com/docs/capabilities/search_rules/create_your_first_rule#turn-on-search-rules-1).

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/dynamic-search-rules/coffee-top-picks' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "description": "Show our two coffee picks first for coffee searches",
    "active": true,
    "conditions": {
      "query": {
        "words": "coffee"
      }
    },
    "actions": {
      "pin": [
        {
          "indexUid": "products",
          "id": "5",
          "position": 0
        },
        {
          "indexUid": "products",
          "id": "3",
          "position": 1
        }
      ]
    }
  }'
```

* Each entry in `pin` places one document at one position. The order of the entries does not matter, the `position` values do.
* Positions count from `0` in the API, so `0` is the first result. The dashboard displays the same slot as position `1`.
* If the current search filters exclude a pinned document, Meilisearch drops that pin and an organic result takes the slot. The other pins stay where you put them. See [specific behaviors](https://www.meilisearch.com/docs/capabilities/search_rules/advanced/specific_behaviors).

Meilisearch answers with a summarized task containing a `taskUid`, because creating a rule is an [asynchronous operation](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations):

```json
{
  "taskUid": 46,
  "indexUid": ".meili_dsr",
  "status": "enqueued",
  "type": "dsrUpdate",
  "enqueuedAt": "2026-05-28T09:12:41.157267963Z"
}
```

The rule applies to searches once this task reaches the `succeeded` status. To follow its progress, pass the `taskUid` to the [task API](https://www.meilisearch.com/docs/reference/api/tasks/get-task).

From then on, every search containing "coffee" starts with your two picks, in the order you chose, and the organic results follow.

## Next steps

> [card] [Boost documents](https://www.meilisearch.com/docs/capabilities/search_rules/how_to/boost_documents)
>
> Rank a selection of documents higher during a campaign

> [card] [Hide documents](https://www.meilisearch.com/docs/capabilities/search_rules/how_to/hide_documents)
>
> Remove documents from the results without reindexing

> [card] [Specific behaviors](https://www.meilisearch.com/docs/capabilities/search_rules/advanced/specific_behaviors)
>
> See how pins interact with filters and with the other actions

> [card] [API reference](https://www.meilisearch.com/docs/reference/api/search-rules/create-or-update-a-search-rule)
>
> Endpoints, rule fields, and update behavior


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.