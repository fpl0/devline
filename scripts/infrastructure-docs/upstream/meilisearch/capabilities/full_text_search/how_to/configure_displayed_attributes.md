# Configure displayed attributes

> Choose which document fields appear in search results by setting the displayedAttributes index setting.

By default, all fields in a document are **displayed** in search results. Use `displayedAttributes` to control which fields are returned when a document matches a query.

Fields not listed in `displayedAttributes` are still stored in the database and remain [searchable](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_searchable_attributes) if configured. You can add them back to the displayed list at any time.

## Set displayed attributes

Suppose you manage a movies database and only want search results to show the title, overview, release date, and genres:

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/movies/settings/displayed-attributes' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '[
      "title",
      "overview",
      "genres",
      "release_date"
    ]'
```

```javascript
client.index('movies').updateDisplayedAttributes([
    'title',
    'overview',
    'genres',
    'release_date',
  ]
)
```

```python
client.index('movies').update_displayed_attributes([
    'title',
    'overview',
    'genres',
    'release_date'
])
```

```php
$client->index('movies')->updateDisplayedAttributes([
  'title',
  'overview',
  'genres',
  'release_date'
]);
```

```java
String[] attributes = {"title", "overview", "genres", "release_date"}
client.index("movies").updateDisplayedAttributesSettings(attributes);
```

```ruby
client.index('movies').update_settings({
  displayed_attributes: [
    'title',
    'overview',
    'genres',
    'release_date'
  ]
})
```

```go
displayedAttributes := []string{
  "title",
  "overview",
  "genres",
  "release_date",
}
client.Index("movies").UpdateDisplayedAttributes(&displayedAttributes)
```

```csharp
await client.Index("movies").UpdateDisplayedAttributesAsync(new[]
{
    "title",
    "overview",
    "genres",
    "release_date"
});
```

```rust
let displayed_attributes = [
  "title",
  "overview",
  "genres",
  "release_date"
];

let task: TaskInfo = client
  .index("movies")
  .set_displayed_attributes(&displayed_attributes)
  .await
  .unwrap();
```

```swift
let displayedAttributes: [String] = [
    "title",
    "overview",
    "genres",
    "release_date"
]
client.index("movies").updateDisplayedAttributes(displayedAttributes) { (result) in
    switch result {
    case .success(let task):
        print(task)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('movies').updateDisplayedAttributes([
  'title',
  'overview',
  'genres',
  'release_date',
]);
```

With this configuration, fields like `id`, `poster_url`, or `internal_rating` are excluded from search results even if they exist in the document.

## When to limit displayed attributes

* **Performance**: reducing the number of displayed fields makes response payloads smaller, especially when documents contain large text fields or many attributes
* **Security**: hide internal fields (admin notes, cost prices, internal IDs) from the search response without removing them from the index
* **Clarity**: return only the fields your UI needs, reducing frontend parsing work

## Reset displayed attributes

To restore the default behavior (all fields displayed), reset the setting:

```bash
curl \
  -X DELETE 'MEILISEARCH_URL/indexes/movies/settings/displayed-attributes' \
  -H 'Authorization: Bearer MEILISEARCH_KEY'
```

> [!NOTE]
>
> All fields are always stored in the database regardless of display settings. Making a field non-displayed does not delete it. You can also use `attributesToRetrieve` at search time to limit which displayed fields are returned for a specific query, without changing the index setting.

## Next steps

> [card] [Configure searchable attributes](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_searchable_attributes)
>
> Control which fields are searched and their ranking priority

> [card] [Highlight search results](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/highlight_search_results)
>
> Show matched terms in the displayed fields


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.