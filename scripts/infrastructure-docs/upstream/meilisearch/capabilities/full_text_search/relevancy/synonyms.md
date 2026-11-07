# Synonyms

> Use Meilisearch synonyms to indicate sets of query terms which should be considered equivalent during search.

If multiple words have an equivalent meaning in your dataset, you can [create a list of synonyms](https://www.meilisearch.com/docs/reference/api/settings/update-synonyms). This will make your search results more relevant.

Words set as synonyms won't always return the same results. With the default settings, the `movies` dataset should return 547 results for `great` and 66 for `fantastic`. Let's set them as synonyms:

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/movies/settings/synonyms' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "great": ["fantastic"], "fantastic": ["great"]
  }'
```

```javascript
client.index('movies').updateSynonyms({
  'great': ['fantastic'],
  'fantastic': ['great']
})
```

```python
client.index('movies').update_synonyms({
  'great': ['fantastic'],
  'fantastic': ['great']
})
```

```php
$client->index('movies')->updateSynonyms([
  'great' => ['fantastic'],
  'fantastic' => ['great'],
]);
```

```java
HashMap<String, String[]> synonyms = new HashMap<String, String[]>();
synonyms.put("great", new String[] {"fantastic"});
synonyms.put("fantastic", new String[] {"great"});

client.index("movies").updateSynonymsSettings(synonyms);
```

```ruby
client.index('movies').update_synonyms({
  great: ['fantastic'],
  fantastic: ['great']
})
```

```go
synonyms := map[string][]string{
  "great":      []string{"fantastic"},
  "fantastic":  []string{"great"},
}
client.Index("movies").UpdateSynonyms(&synonyms)
```

```csharp
var synonyms = new Dictionary<string, IEnumerable<string>>
{
    { "great", new string[] { "fantastic" } },
    { "fantastic", new string[] { "great" } }
};
await client.Index("movies").UpdateSynonymsAsync(synonyms);
```

```rust
let mut synonyms = std::collections::HashMap::new();
synonyms.insert(String::from("great"), vec![String::from("fantastic")]);
synonyms.insert(String::from("fantastic"), vec![String::from("great")]);

let task: TaskInfo = client
  .index("movies")
  .set_synonyms(&synonyms)
  .await
  .unwrap();
```

```swift
let synonyms: [String: [String]] = [
  "great": ["fantastic"],
  "fantastic": ["great"]
]

client.index("movies").updateSynonyms(synonyms) { (result) in
  switch result {
  case .success(let task):
      print(task)
  case .failure(let error):
      print(error)
  }
}
```

```dart
await client.index('movies').updateSynonyms({
  'great': ['fantastic'],
  'fantastic': ['great'],
});
```

With the new settings, searching for `great` returns 595 results and `fantastic` returns 423 results. This is due to various factors like [typos](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/typo_tolerance_settings#minwordsizefortypos) and [splitting the query](https://www.meilisearch.com/docs/resources/internals/concat#split-queries) to find relevant documents. The search for `great` will allow only one typo (for example, `create`) and take into account all variations of `great` (for instance, `greatest`) along with `fantastic`.

> [!WARNING]
>
> The number of search results may vary depending on changes to the `movies` dataset.

## Normalization

During the indexing process, all synonyms are processed by using the index settings, i.e., stop words, separator and non-separator tokens. Always make sure no synonyms are treated as separator tokens (e.g., `&`), or they'll be ignored during the search.

### Example

Consider a situation where `Résumé` and `CV` are set as synonyms.

```json
{
  "Résumé": [
    "CV"
  ],
  "CV": [
    "Résumé"
  ]
}
```

A search for `cv` would return any documents containing `cv` or `CV`, in addition to any that contain `Résumé`, `resumé`, `resume`, etc., unaffected by case or accent marks.

## One-way association

Use this when you want one word to be synonymous with another, but not the other way around.

```
phone => iphone
```

A search for `phone` will return documents containing `iphone` as if they contained the word `phone`.

However, if you search for `iphone`, documents containing `phone` will be ranked lower in the results due to [the typo rule](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/ranking_rules).

### Example

To create a one-way synonym list, this is the JSON syntax that should be [added to the settings](https://www.meilisearch.com/docs/reference/api/settings/update-synonyms).

```json
{
  "phone": [
    "iphone"
  ]
}
```

## Relevancy

**The exact search query will always take precedence over its synonyms.** The `exactness` ranking rule favors exact words over synonyms when ranking search results.

Taking the following set of search results:

```json
[
  {
    "id": 0,
    "title": "Ghouls 'n Ghosts"
  },
  {
    "id": 1,
    "title": "Phoenix Wright: Spirit of Justice"
  }
]
```

If you configure `ghost` as a synonym of `spirit`, queries searching for `spirit` will return document `1` before document `0`.

## Mutual association

By associating one or more synonyms with each other, they will be considered the same in both directions.

```
shoe <=> boot <=> slipper <=> sneakers
```

When a search is done with one of these words, all synonyms will be considered as the same word and will appear in the search results.

### Example

To create a mutual association between four words, this is the JSON syntax that should be [added to the settings](https://www.meilisearch.com/docs/reference/api/settings/update-synonyms).

```json
{
  "shoe": [
    "boot",
    "slipper",
    "sneakers"
  ],
  "boot": [
    "shoe",
    "slipper",
    "sneakers"
  ],
  "slipper": [
    "shoe",
    "boot",
    "sneakers"
  ],
  "sneakers": [
    "shoe",
    "boot",
    "slipper"
  ]
}
```

## Multi-word synonyms

Meilisearch treats multi-word synonyms as [phrases](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-q).

### Example

Suppose you set `San Francisco` and `SF` as synonyms with a [mutual association](#mutual-association)

```json
{
  "san francisco": [
    "sf"
  ],
  "sf": [
    "san francisco"
  ]
}
```

If you input `SF` as a search query, Meilisearch will also return results containing the phrase `San Francisco`.

However, depending on the ranking rules, they might be considered less [relevant](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/relevancy) than those containing `SF`. The reverse is also true: if your query is `San Francisco`, documents containing `San Francisco` may rank higher than those containing `SF`.

## Synonym term length limitation

Meilisearch only fetches synonyms for search terms that are between 1 and 3 words. Terms with 4 or more words will not return any synonym matches.

For example, if you set `"lord of the rings"` as a synonym for `"lotr"`, searching for `"lotr"` will return documents containing `"lord of the rings"`. However, if you search for `"lord of the rings"`, Meilisearch will not return documents containing `"lotr"` because the search term has more than 3 words.

## Maximum number of synonyms per term

A single term may have up to 50 synonyms. Meilisearch silently ignores any synonyms beyond this limit. For example, if you configure 51 synonyms for `book`, Meilisearch will only return results containing the term itself and the first 50 synonyms.

If any synonyms for a term contain more than one word, the sum of all words across all synonyms for that term cannot exceed 100 words. Meilisearch silently ignores any synonyms beyond this limit. For example, if you configure 40 synonyms for `computer` in your application, taken together these synonyms must contain fewer than 100 words.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.