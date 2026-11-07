# Configure a custom dictionary

> Teach Meilisearch to treat groups of strings as a single term by supplying a supplementary dictionary of user-defined words.

The `dictionary` setting allows you to instruct Meilisearch to consider groups of strings as a single term by adding a supplementary dictionary of user-defined terms. Entries in the dictionary override the default tokenizer, so Meilisearch will recognize them as indivisible tokens during both indexing and search.

## When to use a custom dictionary

A custom dictionary is particularly useful in two situations:

* **Datasets with many domain-specific words and in languages where words are not separated by whitespace**, such as Japanese. Adding domain terms or uninterrupted character sequences to the dictionary ensures Meilisearch treats them as single units instead of fragmenting them during tokenization.
* **Space-separated languages that contain names or abbreviations with interleaved dots and spaces**, such as `"J. R. R. Tolkien"` and `"W. E. B. Du Bois"`. Without a custom dictionary, the default tokenizer splits these names into separate letters and periods, which makes them hard to match as a cohesive term.

## Check current dictionary

Retrieve the current `dictionary` setting for an index:

```bash
curl \
  -X GET 'MEILISEARCH_URL/indexes/books/settings/dictionary' \
  -H 'Authorization: Bearer API_KEY'
```

```javascript
client.index('books').getDictionary()
```

```python
client.index('books').get_dictionary()
```

```php
$client->index('books')->getDictionary();
```

```java
client.index("books").getDictionarySettings();
```

```ruby
client.index('books').dictionary
```

```go
client.Index("books").GetDictionary()
```

```csharp
var indexDictionary = await client.Index("books").GetDictionaryAsync();
```

```rust
let task: TaskInfo = client
  .index('books')
  .get_dictionary()
  .await
  .unwrap();
```

```swift
client.index("books").getDictionary { result in
  // handle result
}
```

By default, the response is an empty array `[]`, meaning no custom dictionary entries are configured.

## Update the dictionary

Add custom terms to the dictionary:

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/books/settings/dictionary' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    "J. R. R.",
    "W. E. B."
  ]'
```

```javascript
client.index('books').updateDictionary(['J. R. R.', 'W. E. B.'])
```

```python
client.index('books').update_dictionary(["J. R. R.", "W. E. B."])
```

```php
$client->index('books')->updateDictionary(['J. R. R.', 'W. E. B.']);
```

```java
client.index("books").updateDictionarySettings(new String[] {"J. R. R.", "W. E. B."});
```

```ruby
client.index('books').update_dictionary(['J. R. R.', 'W. E. B.'])
```

```go
client.Index("books").UpdateDictionary([]string{
  "J. R. R.",
  "W. E. B.",
})
```

```csharp
var newDictionary = new string[] { "J. R. R.", "W. E. B." };
await client.Index("books").UpdateDictionaryAsync(newDictionary);
```

```rust
let task: TaskInfo = client
  .index('books')
  .set_dictionary(['J. R. R.', 'W. E. B.'])
  .await
  .unwrap();
```

```swift
client.index("books").updateDictionary(["J. R. R.", "W. E. B."]) { result in
  // handle result
}
```

After this request completes, Meilisearch treats `"J. R. R."` and `"W. E. B."` as single tokens. Queries for `"J. R. R. Tolkien"` will match documents where the name appears exactly as spelled, instead of being broken into separate characters.

> [!WARNING]
>
> Updating `dictionary` triggers a re-indexing of all documents in the index. This is an [asynchronous](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations) operation. Use the [task API](https://www.meilisearch.com/docs/reference/api/tasks/get-all-tasks) to monitor progress.

## Reset the dictionary

Clear the custom dictionary and return to the default tokenizer behavior:

```bash
curl \
  -X DELETE 'MEILISEARCH_URL/indexes/books/settings/dictionary' \
  -H 'Authorization: Bearer API_KEY'
```

```javascript
client.index('books').resetDictionary()
```

```python
client.index('books').reset_dictionary()
```

```php
$client->index('books')->resetDictionary();
```

```java
client.index("books").resetDictionarySettings();
```

```ruby
client.index('books').reset_dictionary
```

```go
client.Index("books").ResetDictionary()
```

```csharp
await client.Index("books").ResetDictionaryAsync();
```

```rust
let task: TaskInfo = client
  .index('books')
  .reset_dictionary()
  .await
  .unwrap();
```

```swift
client.index("books").resetDictionary { result in
  // handle result
}
```

## Dictionary vs. synonyms vs. stop words

* Use [`synonyms`](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/synonyms) to map different words to the same concept (for example, `NYC` to `New York City`).
* Use [`stopWords`](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_stop_words) to ignore common terms that add no signal to search.
* Use `dictionary` to preserve multi-character or whitespace-containing sequences that the default tokenizer would otherwise split.

> [!NOTE]
>
> For the full API reference, see [get dictionary](https://www.meilisearch.com/docs/reference/api/settings/get-dictionary).


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.