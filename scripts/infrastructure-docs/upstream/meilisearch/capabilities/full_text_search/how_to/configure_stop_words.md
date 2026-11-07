# Configure stop words

> Set up stop words to ignore common terms like "the", "a", and "is" during search, improving both performance and relevance.

Stop words are common terms that appear in nearly every document and add little value to search relevancy. Words like "the", "is", "at", and "of" are typical examples. Configuring stop words tells Meilisearch to ignore these terms during [indexing](https://www.meilisearch.com/docs/capabilities/indexing/overview) and searching, which improves both query speed and result quality.

## Why configure stop words

Without stop words, a search for `the lord of the rings` treats every word equally. The words "the" and "of" match nearly every document, diluting the relevancy of the more meaningful terms "lord" and "rings". By marking "the" and "of" as stop words, Meilisearch focuses on the terms that actually matter.

Stop words also improve search performance. Since these common words appear in many documents, ignoring them reduces the number of comparisons Meilisearch needs to make during each query.

## Check current stop words

Retrieve the current stop words for an index:

```bash
curl \
  -X GET 'MEILISEARCH_URL/indexes/movies/settings/stop-words' \
  -H 'Authorization: Bearer API_KEY'
```

```javascript
client.index('movies').getStopWords()
```

```python
client.index('movies').get_stop_words()
```

```php
$client->index('movies')->getStopWords();
```

```java
client.index("movies").getStopWordsSettings();
```

```ruby
client.index('movies').stop_words
```

```go
client.Index("movies").GetStopWords()
```

```csharp
await client.Index("movies").GetStopWordsAsync();
```

```rust
let stop_words: Vec<String> = client
  .index("movies")
  .get_stop_words()
  .await
  .unwrap();
```

```swift
client.index("movies").getStopWords { (result) in
    switch result {
    case .success(let stopWords):
        print(stopWords)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('movies').getStopWords();
```

By default, the response is an empty array `[]`, meaning no stop words are configured.

## Update stop words

Set a list of stop words for an index. Here is an example with common English stop words:

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/movies/settings/stop-words' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    "the",
    "of",
    "to"
  ]'
```

```javascript
client.index('movies').updateStopWords(['of', 'the', 'to'])
```

```python
client.index('movies').update_stop_words(['of', 'the', 'to'])
```

```php
$client->index('movies')->updateStopWords(['the', 'of', 'to']);
```

```java
client.index("movies").updateStopWordsSettings(new String[] {"of", "the", "to"});
```

```ruby
client.index('movies').update_stop_words(['of', 'the', 'to'])
```

```go
stopWords := []string{"of", "the", "to"}
client.Index("movies").UpdateStopWords(&stopWords)
```

```csharp
await client.Index("movies").UpdateStopWordsAsync(new[] {"of", "the", "to"});
```

```rust
let stop_words = ["of", "the", "to"];
let task: TaskInfo = client
  .index("movies")
  .set_stop_words(&stop_words)
  .await
  .unwrap();
```

```swift
let stopWords: [String] = ["of", "the", "to"]
client.index("movies").updateStopWords(stopWords) { (result) in
    switch result {
    case .success(let task):
        print(task)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('movies').updateStopWords(['of', 'the', 'to']);
```

> [!WARNING]
>
> Updating stop words triggers a re-indexing of all documents in the index. This is an [asynchronous](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations) operation. Use the [task API](https://www.meilisearch.com/docs/reference/api/tasks/get-all-tasks) to monitor progress.

### Common English stop words

Here is a more comprehensive list you can use as a starting point for English-language datasets:

```json
[
  "a", "an", "and", "are", "as", "at", "be", "but", "by",
  "for", "if", "in", "into", "is", "it", "no", "not", "of",
  "on", "or", "such", "that", "the", "their", "then", "there",
  "these", "they", "this", "to", "was", "will", "with"
]
```

Adapt this list to your dataset and language. For example, French datasets might include words like "le", "la", "les", "de", "du", "des".

If your application serves multiple languages, the recommended approach is to create a separate index per language and configure language-specific stop words for each index. This avoids situations where a stop word in one language is a meaningful term in another (for example, "die" is a stop word in German but a meaningful English word).

### Important considerations

* **Stop words are index-specific.** Each index has its own stop word list. If you have multiple indexes with different languages, configure appropriate stop words for each one.
* **Stop words are case-sensitive.** Adding `"the"` is not equivalent to adding `"The"`.
* **Stop words affect indexing.** Meilisearch removes stop words from the index, so changing this setting requires re-indexing.

## Reset stop words

Remove all stop words and return to the default behavior:

```bash
curl \
  -X DELETE 'MEILISEARCH_URL/indexes/movies/settings/stop-words' \
  -H 'Authorization: Bearer API_KEY'
```

```javascript
client.index('movies').resetStopWords()
```

```python
client.index('movies').reset_stop_words()
```

```php
$client->index('movies')->resetStopWords();
```

```java
client.index("movies").resetStopWordsSettings();
```

```ruby
client.index('movies').reset_stop_words
```

```go
client.Index("movies").ResetStopWords()
```

```csharp
await client.Index("movies").ResetStopWordsAsync();
```

```rust
let task: TaskInfo = client
  .index("movies")
  .reset_stop_words()
  .await
  .unwrap();
```

```swift
client.index("movies").resetStopWords { (result) in
    switch result {
    case .success(let task):
        print(task)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('movies').resetStopWords();
```

After resetting, Meilisearch treats all words as meaningful during indexing and searching.

## Effect on phrase search

Stop words are also ignored inside [phrase searches](https://www.meilisearch.com/docs/capabilities/full_text_search/getting_started/phrase_search). If "the" is a stop word, searching for `"the great gatsby"` effectively matches the same documents as searching for `"great gatsby"`, because "the" is removed from the query.

> [!NOTE]
>
> For the full API reference, see [get stop words](https://www.meilisearch.com/docs/reference/api/settings/get-stopwords).


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.