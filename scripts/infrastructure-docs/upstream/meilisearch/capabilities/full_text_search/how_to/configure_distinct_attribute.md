# Configure distinct attribute

> Use the distinct attribute to deduplicate search results by returning only one document per unique value of a given field.

The distinct attribute is a special, user-designated field. It is most commonly used to prevent Meilisearch from returning a set of several similar documents, instead forcing it to return only one.

You may set a distinct attribute in two ways: using the `distinctAttribute` index setting during configuration, or the `distinct` search parameter at search time.

## Setting a distinct attribute during configuration

`distinctAttribute` is an index setting that configures a default distinct attribute Meilisearch applies to all searches and facet retrievals in that index.

> [!WARNING]
>
> There can be only one `distinctAttribute` per index. Trying to set multiple fields as a `distinctAttribute` will return an error.

> [!WARNING]
>
> Updating `distinctAttribute` will re-index all documents in the index, which can take some time. When setting up a new index, we recommend updating your index settings first and then adding documents. This reduces RAM consumption compared to adding documents and then changing settings, since Meilisearch would have to re-index the whole dataset.

The value of a field configured as a distinct attribute will always be unique among returned documents. This means **there will never be more than one occurrence of the same value** in the distinct attribute field among the returned documents.

When multiple documents have the same value for the distinct attribute, Meilisearch returns only the highest-ranked result after applying [ranking rules](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/ranking_rules). If two or more documents are equivalent in terms of ranking, Meilisearch returns the first result according to its `internal_id`.

## Example

Suppose you have an e-commerce dataset. For an index that contains information about jackets, you may have several identical items with minor variations such as color or size.

As shown below, this dataset contains three documents representing different versions of a Lee jeans leather jacket. One of the jackets is brown, one is black, and the last one is blue.

```json
[
  {
    "id": 1,
    "description": "Leather jacket",
    "brand": "Lee jeans",
    "color": "brown",
    "product_id": "123456"
  },
  {
    "id": 2,
    "description": "Leather jacket",
    "brand": "Lee jeans",
    "color": "black",
    "product_id": "123456"
  },
  {
    "id": 3,
    "description": "Leather jacket",
    "brand": "Lee jeans",
    "color": "blue",
    "product_id": "123456"
  }
]
```

By default, a search for `lee leather jacket` would return all three documents. This might not be desired, since displaying nearly identical variations of the same item can make results appear cluttered.

In this case, you may want to return only one document with the `product_id` corresponding to this Lee jeans leather jacket. To do so, you could set `product_id` as the `distinctAttribute`.

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/jackets/settings/distinct-attribute' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '"product_id"'
```

```javascript
client.index('jackets').updateDistinctAttribute('product_id')
```

```python
client.index('jackets').update_distinct_attribute('product_id')
```

```php
$client->index('jackets')->updateDistinctAttribute('product_id');
```

```java
client.index("jackets").updateDistinctAttributeSettings("product_id");
```

```ruby
client.index('jackets').update_distinct_attribute('product_id')
```

```go
client.Index("jackets").UpdateDistinctAttribute("product_id")
```

```csharp
await client.Index("jackets").UpdateDistinctAttributeAsync("product_id");
```

```rust
let task: TaskInfo = client
  .index("jackets")
  .set_distinct_attribute("product_id")
  .await
  .unwrap();
```

```swift
client.index("jackets").updateDistinctAttribute("product_id") { (result) in
    switch result {
    case .success(let task):
        print(task)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('jackets').updateDistinctAttribute('product_id');
```

By setting `distinctAttribute` to `product_id`, search requests **will never return more than one document with the same `product_id`**.

After setting the distinct attribute as shown above, querying for `lee leather jacket` would only return the first document found. The response would look like this:

```json
{
  "hits": [
    {
      "id": 1,
      "description": "Leather jacket",
      "brand": "Lee jeans",
      "color": "brown",
      "product_id": "123456"
    }
  ],
  "offset": 0,
  "limit": 20,
  "estimatedTotalHits": 1,
  "processingTimeMs": 0,
  "query": "lee leather jacket"
}
```

For more in-depth information on distinct attribute, consult the [API reference](https://www.meilisearch.com/docs/reference/api/settings/get-distinctattribute).

## Setting a distinct attribute at search time

`distinct` is a search parameter you may add to any search query. It allows you to selectively use distinct attributes depending on the context. `distinct` takes precedence over `distinctAttribute`.

To use an attribute with `distinct`, first add it to the `filterableAttributes` list:

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/products/settings/filterable-attributes' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    "product_id",
    "sku",
    "url"
  ]'
```

```javascript
client.index('products').updateFilterableAttributes(['product_id', 'sku', 'url'])
```

```python
client.index('products').update_filterable_attributes(['product_id', 'sku', 'url'])
```

```php
$client->index('products')->updateFilterableAttributes(['product_id', 'sku', 'url']);
```

```java
Settings settings = new Settings();
settings.setFilterableAttributes(new String[] { "product_id", "SKU", "url" });
client.index("products").updateSettings(settings);
```

```ruby
client.index('products').update_filterable_attributes([
  'product_id',
  'sku',
  'url'
])
```

```go
filterableAttributes := []interface{}{
  "product_id",
  "sku",
  "url",
}
client.Index("products").UpdateFilterableAttributes(&filterableAttributes)
```

```csharp
List<string> attributes = new() { "product_id", "sku", "url" };
TaskInfo result = await client.Index("products").UpdateFilterableAttributesAsync(attributes);
```

```rust
let task: TaskInfo = client
  .index("products")
  .settings()
  .set_filterable_attributes(["product_id", "sku", "url"])
  .execute()
  .await
  .unwrap();
```

Then use `distinct` in a search query, specifying one of the configured attributes:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/products/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "white shirt",
    "distinct": "sku"
  }'
```

```javascript
client.index('products').search('white shirt', { distinct: 'sku' })
```

```python
client.index('products').search('white shirt', { distinct: 'sku' })
```

```php
$client->index('products')->search('white shirt', [
  'distinct' => 'sku'
]);
```

```java
SearchRequest searchRequest = SearchRequest.builder().q("white shirt").distinct("sku").build();
client.index("products").search(searchRequest);
```

```ruby
client.index('products').search('white shirt', {
  distinct: 'sku'
})
```

```go
client.Index("products").Search("white shirt", &meilisearch.SearchRequest{
  Distinct: "sku",
})
```

```csharp
var params = new SearchQuery()
{
  Distinct = "sku"
};
await client.Index("products").SearchAsync<Product>("white shirt", params);
```

```rust
let res = client
  .index("products")
  .search()
  .with_query("white shirt")
  .with_distinct("sku")
  .execute()
  .await
  .unwrap();
```


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.