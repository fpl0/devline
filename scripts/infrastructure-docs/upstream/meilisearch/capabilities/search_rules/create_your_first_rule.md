# Create your first rule

> Turn on search rules and create a rule that pins a document to the top of your search results.

This guide shows you how to turn search rules on and build your first rule: pin one document to the top of your search results.

## Set up from the Meilisearch Cloud dashboard

Open your project in the [Meilisearch Cloud dashboard](https://cloud.meilisearch.com) and select the **Search rules** tab.

### Turn on search rules

Click **Enable Search Rules**. Your project can now store rules and apply them at search time.

![Meilisearch Cloud Search rules tab with an "Enable Search Rules" button in the add-on panel](https://mintcdn.com/meilisearch-6b28dec2/pjgozmedxQwKwpe_/assets/images/cloud-search-rules/enable-search-rules.png?fit=max\&auto=format\&n=pjgozmedxQwKwpe_\&q=85\&s=5fe6bb03d617687f0e5bd671efaeaa65)

### Create a new rule

Click **New rule** to open the rule editor.

![Search rules landing page with a "New rule" button in the top-right corner](https://mintcdn.com/meilisearch-6b28dec2/pjgozmedxQwKwpe_/assets/images/cloud-search-rules/new-rule-button.png?fit=max\&auto=format\&n=pjgozmedxQwKwpe_\&q=85\&s=a78fd0ee713450593032a2e99f5e47c2)

### Fill in the general information

| Field | What it does |
| - | - |
| **Rule ID** | The unique identifier of the rule. You cannot change it later without recreating the rule |
| **Description** | A free text note, so your team knows why the rule exists |
| **Rank** | Decides which rule wins when several rules match the same search. The lowest number wins |
| **Active** | Turns the rule on and off. Keep it on to apply the rule at search time |

![General block of the rule editor with the Rule ID, Description, Rank, and Active fields](https://mintcdn.com/meilisearch-6b28dec2/pjgozmedxQwKwpe_/assets/images/cloud-search-rules/general-block.png?fit=max\&auto=format\&n=pjgozmedxQwKwpe_\&q=85\&s=212cfde04b6d0f9563b6bb8ab1ace52d)

### Choose the action

The **Actions** block decides what the rule does to your results.

![Add action dialog with the pin, boost, deboost, and hide options](https://mintcdn.com/meilisearch-6b28dec2/pjgozmedxQwKwpe_/assets/images/cloud-search-rules/add-action-dialog.png?fit=max\&auto=format\&n=pjgozmedxQwKwpe_\&q=85\&s=814900d821154566f5317c35f1197962)

Select **Pin a document**, click **Next**, then fill in the three fields:

* **Index**: the index that holds the document you want to promote. The rule then applies only to searches on this index.
* **Document**: the document itself, selected by its primary key. Meilisearch promotes this exact document and no other one, even if similar documents exist.
* **Position**: the rank the document takes in the results, starting at `1` for the first result. It stays at this rank every time the rule matches, even when it does not match what the user typed.

![Pin a document dialog with an index dropdown, a document selector, and a position input](https://mintcdn.com/meilisearch-6b28dec2/pjgozmedxQwKwpe_/assets/images/cloud-search-rules/pin-document-dialog.png?fit=max\&auto=format\&n=pjgozmedxQwKwpe_\&q=85\&s=f95f3a6aae27e3df6b63744365b74630)

Click **Add action**.

The **Actions** block now lists your action, with the index it targets, the documents you selected, the filter you used, and how many documents it finally affects. A rule can hold several actions: click **Add action** again to add another one.

![Actions block listing one pin action with its index, document, filter, and affected documents](https://mintcdn.com/meilisearch-6b28dec2/pjgozmedxQwKwpe_/assets/images/cloud-search-rules/actions-block.png?fit=max\&auto=format\&n=pjgozmedxQwKwpe_\&q=85\&s=4c536e389af0f9231e35271fd9bea24e)

### Choose the conditions

The **Conditions** block decides when the rule runs. A search must match every condition for the rule to take effect. With the default values, the rule applies to all searches.

For this first rule, set **Query** to `Empty query` and leave the other conditions as they are.

![Conditions block with the Query list open on the "Empty query" option](https://mintcdn.com/meilisearch-6b28dec2/pjgozmedxQwKwpe_/assets/images/cloud-search-rules/conditions-block.png?fit=max\&auto=format\&n=pjgozmedxQwKwpe_\&q=85\&s=b96fbd9b79b5dea47a920b030dd1e625)

### Save the rule

The bar at the bottom of the editor recaps what your rule contains.

![Editor footer with the general, condition, and action recap next to the Create rule button](https://mintcdn.com/meilisearch-6b28dec2/pjgozmedxQwKwpe_/assets/images/cloud-search-rules/create-rule-footer.png?fit=max\&auto=format\&n=pjgozmedxQwKwpe_\&q=85\&s=b73ffcec4387471ac6fde0e5d5657216)

Click **Create rule**.

Your rule is now live: every time a user opens the search on your index without typing anything, the document you selected appears at the top of the results. All the other results remain organic.

### Review and manage your rules

The Search rules tab lists every rule with its conditions, actions, rank, and last update. Search the list by rule ID or description, use the row toggle to pause a rule without deleting it, or the icons on the right to edit or remove it.

![Search rules list with one rule showing its conditions, pin action, rank, last update, and active toggle](https://mintcdn.com/meilisearch-6b28dec2/pjgozmedxQwKwpe_/assets/images/cloud-search-rules/rules-list.png?fit=max\&auto=format\&n=pjgozmedxQwKwpe_\&q=85\&s=eac93375118aa1a98485a51fdb04ca7a)

Rule changes are applied asynchronously, so a rule you just created, updated, or deleted can take a moment to appear in the list.

## Set up on a self-hosted instance

### Turn on search rules

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/experimental-features/' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "dynamicSearchRules": true
  }'
```

Until search rules are turned on, the `/dynamic-search-rules` routes reject your requests and stored rules do not apply at search time.

### Create a rule

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/dynamic-search-rules/invoice-help' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "description": "Promote billing help for invoice searches",
    "precedence": 0,
    "active": true,
    "conditions": {
      "query": {
        "words": "invoice"
      }
    },
    "actions": {
      "pin": [
        {
          "indexUid": "support",
          "id": "billing-workspace-overview",
          "position": 0
        }
      ]
    }
  }'
```

This rule means: every time a user searches the `support` index with a query containing the word `invoice`, the document `billing-workspace-overview` appears at the first position of the results.

* The rule identifier comes from the URL, here `invoice-help`. The body does not contain it.
* `precedence` decides which rule wins when several rules match the same search. The lowest number wins.
* `conditions.query.words` makes the rule run only when every word it contains appears in the query, in any order.
* `actions.pin.indexUid` limits the rule to searches on the `support` index. Without it, the rule applies to any index that contains a document with this ID.
* `actions.pin.position` counts from `0`, so `0` is the first result. The dashboard displays the same slot as position `1`.

Meilisearch answers with a summarized task containing a `taskUid`, because creating a rule is an [asynchronous operation](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations). The rule only applies to searches once this task reaches the `succeeded` status. To follow its progress, pass the `taskUid` to the [task API](https://www.meilisearch.com/docs/reference/api/tasks/get-task).

For the full list of conditions and actions a rule accepts, see the [search rules API reference](https://www.meilisearch.com/docs/reference/api/search-rules/create-or-update-a-search-rule).

### Check the stored rule

```bash
curl \
  -X GET 'MEILISEARCH_URL/dynamic-search-rules/invoice-help' \
  -H 'Authorization: Bearer API_KEY'
```

Meilisearch returns the rule as it stored it:

```json
{
  "uid": "invoice-help",
  "description": "Promote billing help for invoice searches",
  "lastUpdatedAt": "2026-10-20T09:25:53.965008429Z",
  "precedence": 0,
  "active": true,
  "conditions": {
    "query": {
      "words": "invoice"
    }
  },
  "actions": {
    "pin": [
      {
        "indexUid": "support",
        "id": "billing-workspace-overview",
        "position": 0
      }
    ]
  }
}
```

If Meilisearch answers that the rule does not exist, the creation task is probably still in progress. Wait a few moments, then try again.

### Run a search

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/support/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{ "q": "invoice settings" }'
```

The document `billing-workspace-overview` comes back first, followed by your organic results. If the document does not exist, or if the search filters exclude it, Meilisearch drops the pin and returns the organic results only.

## Next steps

> [card] [Boost documents](https://www.meilisearch.com/docs/capabilities/search_rules/how_to/boost_documents)
>
> Rank a selection of documents higher during a campaign

> [card] [Hide documents](https://www.meilisearch.com/docs/capabilities/search_rules/how_to/hide_documents)
>
> Remove documents from the results without reindexing

> [card] [How search rules work](https://www.meilisearch.com/docs/capabilities/search_rules/advanced/how_search_rules_work)
>
> See how Meilisearch builds the final result set

> [card] [API reference](https://www.meilisearch.com/docs/reference/api/search-rules/create-or-update-a-search-rule)
>
> Endpoints, rule fields, and update behavior


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.