# Sort search results

> By default, Meilisearch sorts results according to their relevancy. You can alter this behavior so users can decide at search time results they want to see first.

By default, Meilisearch focuses on ordering results according to their relevancy. You can alter this sorting behavior so users can decide at search time what type of results they want to see first.

This can be useful in many situations, such as when a user wants to see the cheapest products available in a webshop.

> [!TIP]
>
> Sorting at search time can be particularly effective when combined with placeholder searches (`"q": null`).

## Configure Meilisearch for sorting at search time

To allow your users to sort results at search time you must:

1. Decide which attributes you want to use for sorting
2. Add those attributes to the `sortableAttributes` index setting
3. Update Meilisearch's [ranking rules](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/relevancy) (optional)

> [!NOTE]
>
> Meilisearch sorts strings in lexicographic order based on their byte values. For example, `á`, which has a value of 225, will be sorted after `z`, which has a value of 122.
>
> Uppercase letters are sorted as if they were lowercase. They will still appear uppercase in search results.

### Add attributes to `sortableAttributes`

Meilisearch allows you to sort results based on document fields. Only fields containing numbers, strings, arrays of numeric values, and arrays of string values can be used for sorting.

After you have decided which fields you will allow your users to sort on, you must add their attributes to the [`sortableAttributes` index setting](https://www.meilisearch.com/docs/reference/api/settings/get-sortableattributes).

> [!WARNING]
>
> If a field has values of different types across documents, Meilisearch will give precedence to numbers over strings. This means documents with numeric field values will be ranked higher than those with string values.
>
> This can lead to unexpected behavior when sorting. For optimal user experience, only sort based on fields containing the same type of value.

> [!WARNING]
>
> Adding an attribute to `sortableAttributes` that does not exist in any document is accepted silently: if the field does not exist, no error will be thrown. Double-check the attribute name when configuring sorting, as a typo will not surface until you notice results are not sorted as expected.

#### Example

Suppose you have collection of books containing the following fields:

```json
[
  {
    "id": 1,
    "title": "Solaris",
    "author": "Stanislaw Lem",
    "genres": [
      "science fiction"
    ],
    "rating": {
      "critics": 95,
      "users": 87
    },
    "price": 5.00
  },
  …
]
```

If you are using this dataset in a webshop, you might want to allow your users to sort on `author` and `price`:

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/books/settings/sortable-attributes' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    "author",
    "price"
  ]'
```

```javascript
client.index('books').updateSortableAttributes([
    'author',
    'price'
  ])
```

```python
client.index('books').update_sortable_attributes([
  'author',
  'price'
])
```

```php
$client->index('books')->updateSortableAttributes([
  'author',
  'price'
]);
```

```java
client.index("books").updateSortableAttributesSettings(new String[] {"price", "author"});
```

```ruby
client.index('books').update_sortable_attributes(['author', 'price'])
```

```go
sortableAttributes := []string{
  "author",
  "price",
}
client.Index("books").UpdateSortableAttributes(&sortableAttributes)
```

```csharp
await client.Index("books").UpdateSortableAttributesAsync(new [] { "price", "author" });
```

```rust
let sortable_attributes = [
  "author",
  "price"
];

let task: TaskInfo = client
  .index("books")
  .set_sortable_attributes(&sortable_attributes)
  .await
  .unwrap();
```

```swift
client.index("books").updateSortableAttributes(["price", "author"]) { (result: Result<TaskInfo, Swift.Error>) in
  switch result {
  case .success(let task):
    print(task)
  case .failure(let error):
    print(error)
  }
}
```

```dart
await client.index('books').updateSortableAttributes(['author', 'price']);
```

### Customize ranking rule order (optional)

When users sort results at search time, [Meilisearch's ranking rules](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/relevancy) are set up so the top matches emphasize relevant results over sorting order. You might need to alter this behavior depending on your application's needs.

This is the default configuration of Meilisearch's ranking rules:

```json
[
  "words",
  "typo",
  "proximity",
  "attributeRank",
  "sort",
  "wordPosition",
  "exactness"
]
```

`"sort"` is in fifth place. This means it acts as a tie-breaker rule: Meilisearch will first place results closely matching search terms at the top of the returned documents list and only then will apply the `"sort"` parameters as requested by the user. In other words, by default Meilisearch provides a very relevant sorting.

Placing `"sort"` ranking rule higher in the list will emphasize exhaustive sorting over relevant sorting: your results will more closely follow the sorting order your user chose, but will not be as relevant.

> [!TIP]
>
> Sorting applies equally to all documents. Meilisearch does not offer native support for promoting, pinning, and boosting specific documents so they are displayed more prominently than other search results. Consult these Meilisearch blog articles for workarounds on [implementing promoted search results with React InstantSearch](https://blog.meilisearch.com/promoted-search-results-with-react-instantsearch) and [document boosting](https://blog.meilisearch.com/document-boosting).

#### Example

If your users care more about finding cheaper books than they care about finding specific matches to their queries, you can place `sort` much higher in the ranking rules:

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/books/settings/ranking-rules' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    "words",
    "sort",
    "typo",
    "proximity",
    "attributeRank",
    "wordPosition",
    "exactness"
  ]'
```

```javascript
client.index('books').updateRankingRules([
  'words',
  'sort',
  'typo',
  'proximity',
  'attribute',
  'exactness'
])
```

```python
client.index('books').update_ranking_rules([
    'words',
    'sort',
    'typo',
    'proximity',
    'attribute',
    'exactness'
])
```

```php
$client->index('books')->updateRankingRules([
  'words',
  'sort',
  'typo',
  'proximity',
  'attribute',
  'exactness'
]);
```

```java
Settings settings = new Settings();
settings.setRankingRules(new String[]
{
  "words",
  "sort",
  "typo",
  "proximity",
  "attribute",
  "exactness"
});
client.index("books").updateSettings(settings);
```

```ruby
client.index('books').update_ranking_rules([
  'words',
  'sort',
  'typo',
  'proximity',
  'attribute',
  'exactness'
])
```

```go
rankingRules := []string{
  "words",
  "sort",
  "typo",
  "proximity",
  "attribute",
  "exactness",
}
client.Index("books").UpdateRankingRules(&rankingRules)
```

```csharp
await client.Index("books").UpdateRankingRulesAsync(new[]
{
    "words",
    "sort",
    "typo",
    "proximity",
    "attribute",
    "exactness"
});
```

```rust
let ranking_rules = [
  "words",
  "sort",
  "typo",
  "proximity",
  "attribute",
  "exactness"
];

let task: TaskInfo = client
  .index("books")
  .set_ranking_rules(&ranking_rules)
  .await
  .unwrap();
```

```swift
let rankingRules: [String] = [
  "words",
  "sort",
  "typo",
  "proximity",
  "attribute",
  "exactness"
]
client.index("books").updateRankingRules(rankingRules) { (result) in
  switch result {
  case .success(let task):
    print(task)
  case .failure(let error):
    print(error)
  }
}
```

```dart
await client.index('books').updateRankingRules(
    ['words', 'sort', 'typo', 'proximity', 'attribute', 'exactness']);
```

## Sort results at search time

After configuring `sortableAttributes`, you can use the [`sort` search parameter](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-sort) to control the sorting order of your search results.

`sort` expects a list of attributes that have been added to the `sortableAttributes` list.

Attributes must be given as `attribute:sorting_order`. In other words, each attribute must be followed by a colon (`:`) and a sorting order: either ascending (`asc`) or descending (`desc`).

When using the `POST` route, `sort` expects an array of strings:

```json
"sort": [
  "price:asc",
  "author:desc"
]
```

When using the `GET` route, `sort` expects a comma-separated string:

```
sort="price:desc,author:asc"
```

The order of `sort` values matter: the higher an attribute is in the search parameter value, the more Meilisearch will prioritize it over attributes placed lower. In our example, if multiple documents have the same value for `price`, Meilisearch will decide the order between these similarly-priced documents based on their `author`.

### Example

Suppose you are searching for books in a webshop and want to see the cheapest science fiction titles. This query searches for `"science fiction"` books sorted from cheapest to most expensive:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/books/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "science fiction",
    "sort": ["price:asc"]
  }'
```

```javascript
client.index('books').search('science fiction', {
  sort: ['price:asc'],
})
```

```python
client.index('books').search('science fiction', {
  'sort': ['price:asc']
})
```

```php
$client->index('books')->search('science fiction', ['sort' => ['price:asc']]);
```

```java
SearchRequest searchRequest = SearchRequest.builder().q("science fiction").sort(new String[] {"price:asc"}).build();
client.index("books").search(searchRequest);
```

```ruby
client.index('books').search('science fiction', { sort: ['price:asc'] })
```

```go
resp, err := client.Index("books").Search("science fiction", &meilisearch.SearchRequest{
  Sort: []string{
    "price:asc",
  },
})
```

```csharp
var sq = new SearchQuery
{
  Sort = new[] { "price:asc" },
};
await client.Index("books").SearchAsync<Book>("science fiction", sq);
```

```rust
let results: SearchResults<Books> = client
  .index("books")
  .search()
  .with_query("science fiction")
  .with_sort(&["price:asc"])
  .execute()
  .await
  .unwrap();
```

```swift
let searchParameters = SearchParameters(
  query: "science fiction",
  sort: ["price:asc"]
)
client.index("books").search(searchParameters) { (result: Result<Searchable<Book>, Swift.Error>) in
  switch result {
  case .success(let searchResult):
    print(searchResult)
  case .failure(let error):
    print(error)
  }
}
```

```dart
await client
    .index('books')
    .search('science fiction', SearchQuery(sort: ['price:asc']));
```

With our example dataset, the results look like this:

```json
[
  {
    "id": 1,
    "title": "Solaris",
    "author": "Stanislaw Lem",
    "genres": [
      "science fiction"
    ],
    "rating": {
      "critics": 95,
      "users": 87
    },
    "price": 5.00
  },
  {
    "id": 2,
    "title": "The Parable of the Sower",
    "author": "Octavia E. Butler",
    "genres": [
      "science fiction"
    ],
    "rating": {
      "critics": 90,
      "users": 92
    },
    "price": 10.00
  }
]
```

It is common to search books based on an author's name. `sort` can help grouping results from the same author. This query would only return books matching the query term `"butler"` and group results according to their authors:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/books/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "butler",
    "sort": ["author:desc"]
  }'
```

```javascript
client.index('books').search('butler', {
  sort: ['author:desc'],
})
```

```python
client.index('books').search('butler', {
  'sort': ['author:desc']
})
```

```php
$client->index('books')->search('butler', ['sort' => ['author:desc']]);
```

```java
SearchRequest searchRequest = SearchRequest.builder().q("butler").sort(new String[] {"author:desc"}).build();
client.index("books").search(searchRequest);
```

```ruby
client.index('books').search('butler', { sort: ['author:desc'] })
```

```go
resp, err := client.Index("books").Search("butler", &meilisearch.SearchRequest{
  Sort: []string{
    "author:desc",
  },
})
```

```csharp
var sq = new SearchQuery
{
  Sort = new[] { "author:desc" },
};
await client.Index("books").SearchAsync<Book>("butler", sq);
```

```rust
let results: SearchResults<Books> = client
  .index("books")
  .search()
  .with_query("butler")
  .with_sort(&["author:desc"])
  .execute()
  .await
  .unwrap();
```

```swift
let searchParameters = SearchParameters(
  query: "butler",
  sort: ["author:desc"]
)
client.index("books").search(searchParameters) { (result: Result<Searchable<Book>, Swift.Error>) in
  switch result {
  case .success(let searchResult):
    print(searchResult)
  case .failure(let error):
    print(error)
  }
}
```

```dart
await client
    .index('books')
    .search('butler', SearchQuery(sort: ['author:desc']));
```

```json
[
  {
    "id": 2,
    "title": "The Parable of the Sower",
    "author": "Octavia E. Butler",
    "genres": [
      "science fiction"
    ],
    "rating": {
      "critics": 90,
      "users": 92
    },
    "price": 10.00
  },
  {
    "id": 5,
    "title": "Wild Seed",
    "author": "Octavia E. Butler",
    "genres": [
      "fantasy"
    ],
    "rating": {
      "critics": 84,
      "users": 80
    },
    "price": 5.00
  },
  {
    "id": 4,
    "title": "Gender Trouble",
    "author": "Judith Butler",
    "genres": [
      "feminism",
      "philosophy"
    ],
    "rating": {
      "critics": 86,
      "users": 73
    },
    "price": 10.00
  }
]
```

### Sort by nested fields

Use dot notation to sort results based on a document's nested fields. The following query sorts returned documents by their user review scores:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/books/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "science fiction",
    "sort": ["rating.users:asc"]
  }'
```

```javascript
client.index('books').search('science fiction', {
  'sort': ['rating.users:asc'],
})
```

```python
client.index('books').search('science fiction', {
  'sort': ['rating.users:asc']
})
```

```php
$client->index('books')->search('science fiction', ['sort' => ['rating.users:asc']]);
```

```java
SearchRequest searchRequest = SearchRequest.builder().q("science fiction").sort(new String[] {"rating.users:asc"}).build();
client.index("books").search(searchRequest);
```

```ruby
client.index('books').search('science fiction', { sort: ['rating.users:asc'] })
```

```go
resp, err := client.Index("books").Search("science fiction", &meilisearch.SearchRequest{
  Sort: []string{
    "rating.users:asc",
  },
})
```

```csharp
SearchQuery sort = new SearchQuery() { Sort = new string[] { "rating.users:asc" }};
await client.Index("books").SearchAsync<Book>("science fiction", sort);
```

```rust
let results: SearchResults<Books> = client
  .index("books")
  .search()
  .with_query("science fiction")
  .with_sort(&["rating.users:asc"])
  .execute()
  .await
  .unwrap();
```

```swift
let searchParameters = SearchParameters(
    query: "science fiction",
    sort: ["rating.users:asc"]
)
client.index("books").search(searchParameters) { (result: Result<Searchable<Meteorite>, Swift.Error>) in
    switch result {
    case .success(let searchResult):
        print(searchResult)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client
    .index('movie_ratings')
    .search('thriller', SearchQuery(sort: ['rating.users:asc']));
```

## Sorting and custom ranking rules

There is a lot of overlap between sorting and configuring [custom ranking rules](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/custom_ranking_rules), as both can greatly influence which results a user will see first.

Sorting is most useful when you want your users to be able to alter the order of returned results at query time. For example, webshop users might want to order results by price depending on what they are searching and to change whether they see the most expensive or the cheapest products first.

Custom ranking rules, instead, establish a default sorting rule that is enforced in every search. This approach can be useful when you want to promote certain results above all others, regardless of a user's preferences. For example, you might want a webshop to always feature discounted products first, no matter what a user is searching for.

## Example application

Take a look at our demos for examples of how to implement sorting:

* **Ecommerce demo**: [preview](https://ecommerce.meilisearch.com/) • [GitHub repository](https://github.com/meilisearch/ecommerce-demo/)
* **CRM SaaS demo**: [preview](https://saas.meilisearch.com/) • [GitHub repository](https://github.com/meilisearch/saas-demo/)


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.