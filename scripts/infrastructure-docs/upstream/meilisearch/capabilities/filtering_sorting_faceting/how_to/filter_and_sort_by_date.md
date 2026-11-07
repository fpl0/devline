# Filtering and sorting by date

> Learn how to index documents with chronological data, and how to sort and filter search results based on time.

In this guide, you will learn about Meilisearch's approach to date and time values, how to prepare your dataset for indexing, and how to chronologically sort and filter search results.

## Preparing your documents

To filter and sort search results chronologically, your documents must have at least one field containing a [UNIX timestamp](https://kb.narrative.io/what-is-unix-time). You may also use a string with a date in a format that can be sorted lexicographically, such as `"2025-01-13"`.

As an example, consider a database of video games. In this dataset, the release year is formatted as a timestamp:

```json
[
  {
    "id": 0,
    "title": "Return of the Obra Dinn",
    "genre": "adventure",
    "release_timestamp": 1538949600
  },
  {
    "id": 1,
    "title": "The Excavation of Hob's Barrow",
    "genre": "adventure",
    "release_timestamp": 1664316000
  },
  {
    "id": 2,
    "title": "Bayonetta 2",
    "genre": "action",
    "release_timestamp": 1411164000
  }
]
```

Once all documents in your dataset have a date field, [index your data](https://www.meilisearch.com/docs/reference/api/documents/add-or-replace-documents) as usual. The example below adds a [videogame dataset](https://www.meilisearch.com/docs/assets/datasets/videogames.json) to a `games` index:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/games/documents' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary @games.json
```

```javascript
const games = require('./games.json')
client.index('games').addDocuments(games).then((res) => console.log(res))
```

```python
import json

json_file = open('./games.json', encoding='utf-8')
games = json.load(json_file)
client.index('games').add_documents(games)
```

```php
$gamesJson = file_get_contents('games.json');
$games = json_decode($gamesJson);

$client->index('games')->addDocuments($games);
```

```java
import com.meilisearch.sdk;
import org.json.JSONArray;
import java.nio.file.Files;
import java.nio.file.Path;

Path fileName = Path.of("games.json");
String gamesJson = Files.readString(fileName);
Index index = client.index("games");
index.addDocuments(gamesJson);
```

```ruby
require 'json'

games = JSON.parse(File.read('games.json'))
client.index('games').add_documents(games)
```

```go
jsonFile, _ := os.Open("games.json")
defer jsonFile.Close()

byteValue, _ := io.ReadAll(jsonFile)
var games []map[string]interface{}
json.Unmarshal(byteValue, &games)

client.Index("games").AddDocuments(games, nil)
```

```csharp
string jsonString = await File.ReadAllTextAsync("games.json");
var games = JsonSerializer.Deserialize<IEnumerable<Movie>>(jsonString, options);
var index = client.Index("games");
await index.AddDocumentsAsync<Movie>(games);
```

```rust
let mut file = File::open("games.json")
  .unwrap();
let mut content = String::new();
file
  .read_to_string(&mut content)
  .unwrap();
let docs: Vec<Game> = serde_json::from_str(&content)
  .unwrap();

client
  .index("games")
  .add_documents(&docs, None)
  .await
  .unwrap();
```

```swift
let path = Bundle.main.url(forResource: "games", withExtension: "json")!
let documents: Data = try Data(contentsOf: path)

client.index("games").addDocuments(documents: documents) { (result) in
  switch result {
  case .success(let task):
      print(task)
  case .failure(let error):
      print(error)
  }
}
```

```dart
//import 'dart:io';
//import 'dart:convert';
final json = await File('games.json').readAsString();
await client.index('games').addDocumentsJson(json);
```

## Filtering by date

To filter search results based on their timestamp, add your document's timestamp field to the list of [`filterableAttributes`](https://www.meilisearch.com/docs/reference/api/settings/update-filterableattributes):

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/games/settings/filterable-attributes' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    "release_timestamp"
  ]'
```

```javascript
client.index('games').updateFilterableAttributes(['release_timestamp'])
```

```python
client.index('games').update_filterable_attributes(['release_timestamp'])
```

```php
$client->index('games')->updateFilterableAttributes(['release_timestamp']);
```

```java
client.index("movies").updateFilterableAttributesSettings(
  new String[] { "release_timestamp" });
```

```ruby
client.index('games').update_filterable_attributes(['release_timestamp'])
```

```go
filterableAttributes := []interface{}{"release_timestamp"}
client.Index("games").UpdateFilterableAttributes(&filterableAttributes)
```

```csharp
await client.Index("games").UpdateFilterableAttributesAsync(new string[] { "release_timestamp" });
```

```rust
let settings = Settings::new()
  .with_filterable_attributes(["release_timestamp"]);

let task: TaskInfo = client
  .index("games")
  .set_settings(&settings)
  .await
  .unwrap();
```

```swift
client.index("games").updateFilterableAttributes(["release_timestamp"]) { (result) in
  switch result {
  case .success(let task):
      print(task)
  case .failure(let error):
      print(error)
  }
}
```

```dart
await client
    .index('games')
    .updateFilterableAttributes(['release_timestamp']);
```

Once you have configured `filterableAttributes`, you can filter search results by date. The following query only returns games released between 2018 and 2022:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/games/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "",
    "filter": "release_timestamp >= 1514761200 AND release_timestamp < 1672527600"
  }'
```

```javascript
client.index('games').search('', {
  filter: 'release_timestamp >= 1514761200 AND release_timestamp < 1672527600'
})
```

```python
client.index('games').search('', {
  'filter': 'release_timestamp >= 1514761200 AND release_timestamp < 1672527600'
})
```

```php
$client->index('games')->search('', [
  'filter' => ['release_timestamp >= 1514761200 AND release_timestamp < 1672527600']
]);
```

```java
SearchRequest searchRequest = SearchRequest.builder().q("").filter(new String[] {"release_timestamp >= 1514761200 AND release_timestamp < 1672527600"}).build();
client.index("games").search(searchRequest);
```

```ruby
client.index('games').search('', {
  filter: 'release_timestamp >= 1514761200 AND release_timestamp < 1672527600'
})
```

```go
client.Index("games").Search("", &meilisearch.SearchRequest{
  Filter: "release_timestamp >= 1514761200 AND release_timestamp < 1672527600",
})
```

```csharp
var filters = new SearchQuery() { Filter = "release_timestamp >= 1514761200 AND release_timestamp < 1672527600" };
var games = await client.Index("games").SearchAsync<Game>("", filters);
```

```rust
let results: SearchResults<Game> = client
  .index("games")
  .search()
  .with_filter("release_timestamp >= 1514761200 AND release_timestamp < 1672527600")
  .execute()
  .await
  .unwrap();
```

```swift
let searchParameters = SearchParameters(
  query: "",
  filter: "release_timestamp >= 1514761200 AND release_timestamp < 1672527600"
)
client.index("games").search(searchParameters) { (result: Result<Searchable<Game>, Swift.Error>) in
  switch result {
  case .success(let searchResult):
      print(searchResult)
  case .failure(let error):
      print(error)
  }
}
```

```dart
await client.index('games').search(
      '',
      SearchQuery(
        filterExpression: Meili.and([
          Meili.gte(
            'release_timestamp'.toMeiliAttribute(),
            Meili.value(DateTime(2017, 12, 31, 23, 0)),
          ),
          Meili.lt(
            'release_timestamp'.toMeiliAttribute(),
            Meili.value(DateTime(2022, 12, 31, 23, 0)),
          ),
        ]),
      ),
    );
```

## Sorting by date

To sort search results chronologically, add your document's timestamp field to the list of [`sortableAttributes`](https://www.meilisearch.com/docs/reference/api/settings/update-sortableattributes):

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/games/settings/sortable-attributes' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    "release_timestamp"
  ]'
```

```javascript
client.index('games').updateSortableAttributes(['release_timestamp'])
```

```python
client.index('games').update_sortable_attributes(['release_timestamp'])
```

```php
$client->index('games')->updateSortableAttributes(['release_timestamp']);
```

```java
Settings settings = new Settings();
settings.setSortableAttributes(new String[] {"release_timestamp"});
client.index("games").updateSettings(settings);
```

```ruby
client.index('games').update_sortable_attributes(['release_timestamp'])
```

```go
sortableAttributes := []string{"release_timestamp","author"}
client.Index("games").UpdateSortableAttributes(&sortableAttributes)
```

```csharp
await client.Index("games").UpdateSortableAttributesAsync(new string[] { "release_timestamp" });
```

```rust
let settings = Settings::new()
  .with_sortable_attributes(["release_timestamp"]);

let task: TaskInfo = client
  .index("games")
  .set_settings(&settings)
  .await
  .unwrap();
```

```swift
client.index("games").updateSortableAttributes(["release_timestamp"]) { (result) in
  switch result {
  case .success(let task):
    print(task)
  case .failure(let error):
    print(error)
  }
}
```

```dart
await client
    .index('games')
    .updateSortableAttributes(['release_timestamp']);
```

Once you have configured `sortableAttributes`, you can sort your search results based on their timestamp. The following query returns all games sorted from most recent to oldest:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/games/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "",
    "sort": ["release_timestamp:desc"]
  }'
```

```javascript
client.index('games').search('', {
  sort: ['release_timestamp:desc'],
})
```

```python
client.index('games').search('', {
  'sort': ['release_timestamp:desc']
})
```

```php
$client->index('games')->search('', ['sort' => ['release_timestamp:desc']]);
```

```java
SearchRequest searchRequest = SearchRequest.builder().q("").sort(new String[] {"release_timestamp:desc"}).build();
client.index("games").search(searchRequest);
```

```ruby
client.index('games').search('', sort: ['release_timestamp:desc'])
```

```go
client.Index("games").Search("", &meilisearch.SearchRequest{
  Sort: []string{
    "release_timestamp:desc",
  },
})
```

```csharp
SearchQuery sort = new SearchQuery() { Sort = new string[] { "release_timestamp:desc" }};
await client.Index("games").SearchAsync<Game>("", sort);
```

```rust
let results: SearchResults<Game> = client
  .index("games")
  .search()
  .with_sort(["release_timestamp:desc"])
  .execute()
  .await
  .unwrap();
```

```swift
let searchParameters = SearchParameters(
  query: "",
  sort: ["release_timestamp:desc"],
)

client.index("games").search(searchParameters) { (result: Result<Searchable<Game>, Swift.Error>) in
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
    .index('games')
    .search('', SearchQuery(sort: ['release_timestamp:desc']));
```

## Next steps

> [card] [Combine filters and sort](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/combine_filters_and_sort)
>
> Use filters and sorting together to narrow and order search results.

> [card] [Sort results](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/sort_results)
>
> Configure sortable attributes and sort search results by any field.

> [card] [Filter expression reference](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/advanced/filter_expression_syntax)
>
> Learn about all available filter operators and expression syntax.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.