# Filter search results

> In this guide you will see how to configure and use Meilisearch filters in a hypothetical movie database.

In this guide you will see how to configure and use Meilisearch filters in a hypothetical movie database.

## Configure index settings

Suppose you have a collection of movies called `movie_ratings` containing the following fields:

```json
[
  {
    "id": 458723,
    "title": "Us",
    "director": "Jordan Peele",
    "release_date": 1552521600,
    "genres": [
      "Thriller",
      "Horror",
      "Mystery"
    ],
    "rating": {
      "critics": 86,
      "users": 73
    },
  },
  …
]
```

If you want to filter results based on an attribute, you must first add it to the `filterableAttributes` list:

```bash
curl \
  -X PUT 'MEILISEARCH_URL/indexes/movie_ratings/settings/filterable-attributes' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    "genres",
    "director",
    "release_date",
    "ratings"
  ]'
```

```javascript
client.index('movies')
  .updateFilterableAttributes([
    'director',
    'genres'
  ])
```

```python
client.index('movies').update_filterable_attributes([
    'director',
    'genres',
])
```

```php
$client->index('movies')->updateFilterableAttributes(['director', 'genres']);
```

```java
client.index("movies").updateFilterableAttributesSettings(new String[]
{
  "genres",
  "director"
});
```

```ruby
client.index('movies').update_filterable_attributes([
  'director',
  'genres'
])
```

```go
resp, err := client.Index("movies").UpdateFilterableAttributes(&[]interface{}{
  "director",
  "genres",
})
```

```csharp
await client.Index("movies").UpdateFilterableAttributesAsync(new [] { "director", "genres" });
```

```rust
let task: TaskInfo = client
  .index("movies")
  .set_filterable_attributes(["director", "genres"])
  .await
  .unwrap();
```

```swift
client.index("movies").updateFilterableAttributes(["genre", "director"]) { (result) in
    switch result {
    case .success(let task):
        print(task)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('movies').updateFilterableAttributes([
  'director',
  'genres',
]);
```

**This step is mandatory and cannot be done at search time**. Updating `filterableAttributes` requires Meilisearch to re-index all your data, which will take an amount of time proportionate to your dataset size and complexity.

> [!NOTE]
>
> By default, `filterableAttributes` is empty. Filters do not work without first explicitly adding attributes to the `filterableAttributes` list.

> [!TIP]
>
> Calculating `comparison` filters (such as `<`, `>`, or `TO`) is a resource-intensive operation. Disabling them may lead to better search and indexing performance. `equality` filters use fewer resources and have limited impact on performance. Use [granular filterable attributes](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/configure_granular_filters) to enable only the filter operations you actually need for each attribute.

## Use `filter` when searching

After updating the [`filterableAttributes` index setting](https://www.meilisearch.com/docs/reference/api/settings/get-filterableattributes), you can use `filter` to fine-tune your search results.

`filter` is a search parameter you may use at search time. `filter` accepts [filter expressions](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/advanced/filter_expression_syntax) built using any attributes present in the `filterableAttributes` list.

The following code sample returns `Avengers` movies released after 18 March 1995:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movie_ratings/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "Avengers",
    "filter": "release_date > 795484800"
  }'
```

```javascript
client.index('movie_ratings').search('Avengers', {
  filter: 'release_date > 795484800'
})
```

```python
client.index('movie_ratings').search('Avengers', {
  'filter': 'release_date > 795484800'
})
```

```php
$client->index('movie_ratings')->search('Avengers', [
  'filter' => 'release_date > 795484800'
]);
```

```java
SearchRequest searchRequest = SearchRequest.builder().q("Avengers").filter(new String[] {"release_date > \"795484800\""}).build();
client.index("movie_ratings").search(searchRequest);
```

```ruby
client.index('movie_ratings').search('Avengers', { filter: 'release_date > 795484800' })
```

```go
resp, err := client.Index("movie_ratings").Search("Avengers", &meilisearch.SearchRequest{
  Filter: "release_date > \"795484800\"",
})
```

```csharp
SearchQuery filters = new SearchQuery() { Filter = "release_date > \"795484800\"" };
var movies = await client.Index("movie_ratings").SearchAsync<Movie>("Avengers", filters);
```

```rust
let results: SearchResults<Movie> = client
  .index("movie_ratings")
  .search()
  .with_query("Avengers")
  .with_filter("release_date > 795484800")
  .execute()
  .await
  .unwrap();
```

```swift
let searchParameters = SearchParameters(
    query: "Avengers",
    filter: "release_date > 795484800"
)
client.index("movie_ratings").search(searchParameters) { (result: Result<Searchable<Movie>, Swift.Error>) in
    switch result {
    case .success(let searchResult):
        print(searchResult)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('movie_ratings').search(
      'Avengers',
      SearchQuery(
        filterExpression: Meili.gt(
          Meili.attr('release_date'),
          DateTime.utc(1995, 3, 18).toMeiliValue(),
        ),
      ),
    );
```

Use dot notation to filter results based on a document's [nested fields](https://www.meilisearch.com/docs/resources/internals/datatypes). The following query only returns thrillers with good user reviews:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movie_ratings/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "thriller",
    "filter": "rating.users >= 90"
  }'
```

```javascript
client.index('movie_ratings').search('thriller', {
  filter: 'rating.users >= 90'
})
```

```python
client.index('movie_ratings').search('thriller', {
  'filter': 'rating.users >= 90'
})
```

```php
$client->index('movie_ratings')->search('thriller', [
  'filter' => 'rating.users >= 90'
]);
```

```java
SearchRequest searchRequest = SearchRequest.builder().q("thriller").filter(new String[] {"rating.users >= 90"}).build();
client.index("movie_ratings").search(searchRequest);
```

```ruby
client.index('movies_ratings').search('thriller', {
  filter: 'rating.users >= 90'
})
```

```go
resp, err := client.Index("movie_ratings").Search("thriller", &meilisearch.SearchRequest{
  Filter: "rating.users >= 90",
})
```

```csharp
var filters = new SearchQuery() { Filter = "rating.users >= 90" };
var movies = await client.Index("movie_ratings").SearchAsync<MovieRating>("thriller", filters);
```

```rust
let results: SearchResults<MovieRatings> = client
  .index("movie_rating")
  .search()
  .with_query("thriller")
  .with_filter("rating.users >= 90")
  .execute()
  .await
  .unwrap();
```

```swift
let searchParameters = SearchParameters(
    query: "thriller",
    filter: "rating.users >= 90"
)
client.index("movie_ratings").search(searchParameters) { (result: Result<Searchable<Meteorite>, Swift.Error>) in
    switch result {
    case .success(let searchResult):
        print(searchResult)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('movie_ratings').search(
      'thriller',
      SearchQuery(
        filterExpression: Meili.gte(
          //or Meili.attr('rating.users')
          //or 'rating.users'.toMeiliAttribute()
          Meili.attrFromParts(['rating', 'users']),
          Meili.value(90),
        ),
      ),
    );
```

You can also combine multiple conditions. For example, you can limit your search so it only includes `Batman` movies directed by either `Tim Burton` or `Christopher Nolan`:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movie_ratings/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "Batman",
    "filter": "release_date > 795484800 AND (director = \"Tim Burton\" OR director = \"Christopher Nolan\")"
  }'
```

```javascript
client.index('movie_ratings').search('Batman', {
  filter: 'release_date > 795484800 AND (director = "Tim Burton" OR director = "Christopher Nolan")'
})
```

```python
client.index('movie_ratings').search('Batman', {
  'filter': 'release_date > 795484800 AND (director = "Tim Burton" OR director = "Christopher Nolan")'
})
```

```php
$client->index('movie_ratings')->search('Batman', [
  'filter' => 'release_date > 795484800 AND (director = "Tim Burton" OR director = "Christopher Nolan")'
]);
```

```java
SearchRequest searchRequest = SearchRequest.builder().q("Batman").filter(new String[] {"release_date > 795484800 AND (director = \"Tim Burton\" OR director = \"Christopher Nolan\")"}).build();
client.index("movie_ratings").search(searchRequest);
```

```ruby
client.index('movie_ratings').search('Batman', {
  filter: 'release_date > 795484800 AND (director = "Tim Burton" OR director = "Christopher Nolan")'
})
```

```go
resp, err := client.Index("movie_ratings").Search("Batman", &meilisearch.SearchRequest{
  Filter: "release_date > 795484800 AND (director = \"Tim Burton\" OR director = \"Christopher Nolan\")",
})
```

```csharp
SearchQuery filters = new SearchQuery() { Filter = "release_date > 795484800 AND (director =
\"Tim Burton\" OR director = \"Christopher Nolan\")" };
var movies = await client.Index("movie_ratings").SearchAsync<Movie>("Batman", filters);
```

```rust
let results: SearchResults<Movie> = client
  .index("movie_ratings")
  .search()
  .with_query("Batman")
  .with_filter(r#"release_date > 795484800 AND (director = "Tim Burton" OR director = "Christopher Nolan")"#)
  .execute()
  .await
  .unwrap();
```

```swift
let searchParameters = SearchParameters(
    query: "Batman",
    filter: "release_date > 795484800 AND (director = \"Tim Burton\" OR director = \"Christopher Nolan\"")
client.index("movie_ratings").search(searchParameters) { (result: Result<Searchable<Movie>, Swift.Error>) in
    switch result {
    case .success(let searchResult):
        print(searchResult)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('movie_ratings').search(
      'Batman',
      SearchQuery(
        filterExpression: Meili.and([
          Meili.attr('release_date')
              .gt(DateTime.utc(1995, 3, 18).toMeiliValue()),
          Meili.or([
            'director'.toMeiliAttribute().eq('Tim Burton'.toMeiliValue()),
            'director'
                .toMeiliAttribute()
                .eq('Christopher Nolan'.toMeiliValue()),
          ]),
        ]),
      ),
    );
```

Here, the parentheses are mandatory: without them, the filter would return movies directed by `Tim Burton` and released after 1995 or any film directed by `Christopher Nolan`, without constraints on its release date. This happens because `AND` takes precedence over `OR`.

If you only want recent `Planet of the Apes` movies that weren't directed by `Tim Burton`, you can use this filter:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movie_ratings/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "Planet of the Apes",
    "filter": "release_date > 1577884550 AND (NOT director = \"Tim Burton\")"
  }' \
```

```javascript
client.index('movie_ratings').search('Planet of the Apes', {
  filter: "release_date > 1577884550 AND (NOT director = \"Tim Burton\")"
})
```

```python
client.index('movie_ratings').search('Planet of the Apes', {
  'filter': 'release_date > 1577884550 AND (NOT director = "Tim Burton"))'
})
```

```php
$client->index('movie_ratings')->search('Planet of the Apes', [
  'filter' => 'release_date > 1577884550 AND (NOT director = "Tim Burton")'
]);
```

```java
SearchRequest searchRequest = SearchRequest.builder().q("Planet of the Apes").filter(new String[] {"release_date > 1577884550 AND (NOT director = \"Tim Burton\")"}).build();
client.index("movie_ratings").search(searchRequest);
```

```ruby
client.index('movie_ratings').search('Planet of the Apes', {
  filter: "release_date > 1577884550 AND (NOT director = \"Tim Burton\")"
})
```

```go
resp, err := client.Index("movie_ratings").Search("Planet of the Apes", &meilisearch.SearchRequest{
  Filter: "release_date > 1577884550 AND (NOT director = \"Tim Burton\")",
})
```

```csharp
SearchQuery filters = new SearchQuery() { Filter = "release_date > 1577884550 AND (NOT director = \"Tim Burton\")" };
var movies = await client.Index("movie_ratings").SearchAsync<Movie>("Planet of the Apes", filters);
```

```rust
let results: SearchResults<Movie> = client
  .index("movie_ratings")
  .search()
  .with_query("Planet of the Apes")
  .with_filter(r#"release_date > 1577884550 AND (NOT director = "Tim Burton")"#)
  .execute()
  .await
  .unwrap();
```

```swift
let searchParameters = SearchParameters(
    query: "Planet of the Apes",
    filter: "release_date > 1577884550 AND (NOT director = \"Tim Burton\"))
client.index("movie_ratings").search(searchParameters) { (result: Result<Searchable<Movie>, Swift.Error>) in
    switch result {
    case .success(let searchResult):
        print(searchResult)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('movie_ratings').search(
      'Planet of the Apes',
      SearchQuery(
        filterExpression: Meili.and([
          Meili.attr('release_date')
              .gt(DateTime.utc(2020, 1, 1, 13, 15, 50).toMeiliValue()),
          Meili.not(
            Meili.attr('director').eq("Tim Burton".toMeiliValue()),
          ),
        ]),
      ),
    );
```

```
release_date > 1577884550 AND (NOT director = "Tim Burton" AND director EXISTS)
```

> [!WARNING]
>
> [Synonyms](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/synonyms) don't apply to filters. Meaning, if you have `SF` and `San Francisco` set as synonyms, filtering by `SF` and `San Francisco` will show you different results.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.