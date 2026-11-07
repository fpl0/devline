# Getting started with indexing

> Add your first documents to a Meilisearch index, check task status, and verify your data is searchable.

This guide walks you through adding documents to Meilisearch for the first time. You will prepare a dataset, send it to an index, monitor the indexing task, and verify the documents are searchable.

## Prepare your documents

Meilisearch accepts documents in three formats: **JSON**, **NDJSON**, and **CSV**. Each document must contain a field that serves as a unique **[primary key](https://www.meilisearch.com/docs/resources/internals/primary_key)**.

Here is a small sample dataset of movies in JSON format:

```json
[
  {
    "id": 1,
    "title": "Carol",
    "genres": ["Romance", "Drama"],
    "year": 2015
  },
  {
    "id": 2,
    "title": "Wonder Woman",
    "genres": ["Action", "Adventure"],
    "year": 2017
  },
  {
    "id": 3,
    "title": "Life of Pi",
    "genres": ["Adventure", "Drama"],
    "year": 2012
  },
  {
    "id": 4,
    "title": "Mad Max: Fury Road",
    "genres": ["Action", "Adventure"],
    "year": 2015
  }
]
```

In this dataset, `id` is the primary key. Meilisearch automatically infers the primary key if a field is named `id`. If your primary key has a different name, you must specify it when adding documents.

## Send documents to an index

Use the `POST /indexes/{index_uid}/documents` endpoint to add documents. If the index does not exist yet, Meilisearch creates it automatically.

For a large dataset stored in a file:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/documents?primaryKey=id' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer aSampleMasterKey' \
  --data-binary @movies.json
```

```javascript
// With npm:
// npm install meilisearch

// Or with pnpm:
// pnpm add meilisearch

// In your .js file:
// With the `require` syntax:
const { MeiliSearch } = require('meilisearch')
const movies = require('./movies.json')
// With the `import` syntax:
import { MeiliSearch } from 'meilisearch'
import movies from './movies.json'

const client = new MeiliSearch({
  host: 'MEILISEARCH_URL',
  apiKey: 'aSampleMasterKey'
})
client.index('movies').addDocuments(movies)
  .then((res) => console.log(res))
```

```python
# In the command line:
# pip3 install meilisearch

# In your .py file:
import meilisearch
import json

client = meilisearch.Client('MEILISEARCH_URL', 'aSampleMasterKey')

json_file = open('movies.json', encoding='utf-8')
movies = json.load(json_file)
client.index('movies').add_documents(movies)
```

```php
/**
 * Using `meilisearch-php` with the Guzzle HTTP client, in the command line:
 *   composer require meilisearch/meilisearch-php \
 *     guzzlehttp/guzzle \
 *     http-interop/http-factory-guzzle:^1.0
 */

/**
 * In your PHP file:
 */
<?php

require_once __DIR__ . '/vendor/autoload.php';

use Meilisearch\Client;

$client = new Client('MEILISEARCH_URL', 'aSampleMasterKey');

$movies_json = file_get_contents('movies.json');
$movies = json_decode($movies_json);

$client->index('movies')->addDocuments($movies);
```

```java
// For Maven:
// Add the following code to the `<dependencies>` section of your project:
//
// <dependency>
//   <groupId>com.meilisearch.sdk</groupId>
//   <artifactId>meilisearch-java</artifactId>
//   <version>0.22.1</version>
//   <type>pom</type>
// </dependency>

// For Gradle
// Add the following line to the `dependencies` section of your `build.gradle`:
//
// implementation 'com.meilisearch.sdk:meilisearch-java:0.22.1'

// In your .java file:
import com.meilisearch.sdk;
import java.nio.file.Files;
import java.nio.file.Path;

Path fileName = Path.of("movies.json");
String moviesJson = Files.readString(fileName);
Client client = new Client(new Config("MEILISEARCH_URL", "aSampleMasterKey"));
Index index = client.index("movies");
index.addDocuments(moviesJson);
```

```ruby
# In the command line:
# bundle add meilisearch

# In your .rb file:
require 'json'
require 'meilisearch'

client = MeiliSearch::Client.new('MEILISEARCH_URL', 'aSampleMasterKey')

movies_json = File.read('movies.json')
movies = JSON.parse(movies_json)

client.index('movies').add_documents(movies)
```

```go
// In the command line:
// go get -u github.com/meilisearch/meilisearch-go

// In your .go file:
package main

import (
  "os"
  "encoding/json"
  "io"

  "github.com/meilisearch/meilisearch-go"
)

func main() {
  client := meilisearch.New("MEILISEARCH_URL", meilisearch.WithAPIKey("masterKey"))

  jsonFile, _ := os.Open("movies.json")
  defer jsonFile.Close()

  byteValue, _ := io.ReadAll(jsonFile)
  var movies []map[string]interface{}
  json.Unmarshal(byteValue, &movies)

  _, err := client.Index("movies").AddDocuments(movies, nil)
  if err != nil {
      panic(err)
  }
}
```

```csharp
// In the command line:
// dotnet add package Meilisearch

// In your .cs file:
using System.IO;
using System.Text.Json;
using Meilisearch;
using System.Threading.Tasks;
using System.Collections.Generic;

namespace Meilisearch_demo
{
    public class Movie
    {
        public string Id { get; set; }
        public string Title { get; set; }
        public string Poster { get; set; }
        public string Overview { get; set; }
        public IEnumerable<string> Genres { get; set; }
    }

    internal class Program
    {
        static async Task Main(string[] args)
        {
            MeilisearchClient client = new MeilisearchClient("MEILISEARCH_URL", "aSampleMasterKey");
            var options = new JsonSerializerOptions
            {
                PropertyNameCaseInsensitive = true
            };

            string jsonString = await File.ReadAllTextAsync("movies.json");
            var movies = JsonSerializer.Deserialize<IEnumerable<Movie>>(jsonString, options);

            var index = client.Index("movies");
            await index.AddDocumentsAsync<Movie>(movies);
        }
    }
}
```

```text
// In your .toml file:
  [dependencies]
  meilisearch-sdk = "0.33.0"
  # futures: because we want to block on futures
  futures = "0.3"
  # serde: required if you are going to use documents
  serde = { version="1.0",   features = ["derive"] }
  # serde_json: required in some parts of this guide
  serde_json = "1.0"



// In your .rs file:
// Documents in the Rust library are strongly typed
#[derive(Serialize, Deserialize)]
struct Movie {
  id: i64,
  title: String,
  poster: String,
  overview: String,
  release_date: i64,
  genres: Vec<String>
}

// You will often need this `Movie` struct in other parts of this documentation. (you will have to change it a bit sometimes)
// You can also use schemaless values, by putting a `serde_json::Value` inside your own struct like this:
#[derive(Serialize, Deserialize)]
struct Movie {
  id: i64,
  #[serde(flatten)]
  value: serde_json::Value,
}

// Then, add documents into the index:
use meilisearch_sdk::{
  indexes::*,
  client::*,
  search::*,
  settings::*
};
use serde::{Serialize, Deserialize};
use std::{io::prelude::*, fs::File};
use futures::executor::block_on;

fn main() { block_on(async move {
  let client = Client::new("MEILISEARCH_URL", Some("aSampleMasterKey"));

  // Reading and parsing the file
  let mut file = File::open("movies.json")
    .unwrap();
  let mut content = String::new();
  file
    .read_to_string(&mut content)
    .unwrap();
  let movies_docs: Vec<Movie> = serde_json::from_str(&content)
    .unwrap();

  // Adding documents
  client
    .index("movies")
    .add_documents(&movies_docs, None)
    .await
    .unwrap();
})}
```

```swift
// Add this to your `Package.swift`
dependencies: [
  .package(url: "https://github.com/meilisearch/meilisearch-swift.git", from: "0.17.0")
]

// In your .swift file:
let path = Bundle.main.url(forResource: "movies", withExtension: "json")!
let documents: Data = try Data(contentsOf: path)
let client = try MeiliSearch(host: "MEILISEARCH_URL", apiKey: "aSampleMasterKey")

client.index("movies").addDocuments(documents: documents) { (result) in
    switch result {
    case .success(let task):
        print(task)
    case .failure(let error):
        print(error)
    }
}
```

```dart
// In the command line:
// dart pub add meilisearch
// In your .dart file:
import 'package:meilisearch/meilisearch.dart';
import 'dart:io';
import 'dart:convert';
var client = MeiliSearchClient('MEILISEARCH_URL', 'aSampleMasterKey');
final json = await File('movies.json').readAsString();
await client.index('movies').addDocumentsJson(json);
```

For a small number of documents sent inline:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/documents' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    {
      "id": 287947,
      "title": "Shazam",
      "poster": "https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg",
      "overview": "A boy is given the ability to become an adult superhero in times of need with a single magic word.",
      "release_date": "2019-03-23"
    }
  ]'
```

```javascript
client.index('movies').addDocuments([{
    id: 287947,
    title: 'Shazam',
    poster: 'https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg',
    overview: 'A boy is given the ability to become an adult superhero in times of need with a single magic word.',
    release_date: '2019-03-23'
}], { skipCreation: true })
```

```python
client.index('movies').add_documents([{
  'id': 287947,
  'title': 'Shazam',
  'poster': 'https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg',
  'overview': 'A boy is given the ability to become an adult superhero in times of need with a single magic word.',
  'release_date': '2019-03-23'
}], skip_creation=True)
```

```php
$client->index('movies')->addDocuments([
  [
    'id' => 287947,
    'title' => 'Shazam',
    'poster' => 'https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg',
    'overview' => 'A boy is given the ability to become an adult superhero in times of need with a single magic word.',
    'release_date' => '2019-03-23'
  ]
]);
```

```java
client.index("movies").addDocuments("[{"
  + "\"id\": 287947,"
  + "\"title\": \"Shazam\","
  + "\"poster\": \"https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg\","
  + "\"overview\": \"A boy is given the ability to become an adult superhero in times of need with a single magic word.\","
  + "\"release_date\": \"2019-03-23\""
  + "}]"
);
```

```ruby
client.index('movies').add_documents([
  {
    id: 287947,
    title: 'Shazam',
    poster: 'https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg',
    overview: 'A boy is given the ability to become an adult superhero in times of need with a single magic word.',
    release_date: '2019-03-23'
  }
])
```

```go
documents := []map[string]interface{}{
  {
    "id":           287947,
    "title":        "Shazam",
    "poster":       "https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg",
    "overview":     "A boy is given the ability to become an adult superhero in times of need with a single magic word.",
    "release_date": "2019-03-23",
  },
}
options := &meilisearch.DocumentOptions{SkipCreation: false}
client.Index("movies").AddDocuments(documents, options)
```

```csharp
var movie = new[]
{
    new Movie
    {
          Id = "287947",
          Title = "Shazam",
          Poster = "https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg",
          Overview = "A boy is given the ability to become an adult superhero in times of need with a single magic word.",
          ReleaseDate = "2019-03-23"
    }
};
await index.AddDocumentsAsync(movie);
```

```rust
let task: TaskInfo = client
  .index("movies")
  .add_or_replace(&[
    Movie {
      id: 287947,
      title: "Shazam".to_string(),
      poster: "https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg".to_string(),
      overview: "A boy is given the ability to become an adult superhero in times of need with a single magic word.".to_string(),
      release_date: "2019-03-23".to_string(),
    }
  ], None)
  .await
  .unwrap();
```

```swift
let documentJsonString = """
[
  {
    "reference_number": 287947,
    "title": "Shazam",
    "poster": "https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg",
    "overview": "A boy is given the ability to become an adult superhero in times of need with a single magic word.",
    "release_date": "2019-03-23"
  }
]
"""
let documents: Data = documentJsonString.data(using: .utf8)!

client.index("movies").addDocuments(documents: documents) { (result) in
    switch result {
    case .success(let task):
        print(task)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('movies').addDocuments([
  {
    'id': 287947,
    'title': 'Shazam',
    'poster':
        'https://image.tmdb.org/t/p/w1280/xnopI5Xtky18MPhK40cZAGAOVeV.jpg',
    'overview':
        'A boy is given the ability to become an adult superhero in times of need with a single magic word.',
    'release_date': '2019-03-23'
  }
]);
```

Meilisearch returns a summarized task object confirming your request has been accepted:

```json
{
  "taskUid": 0,
  "indexUid": "movies",
  "status": "enqueued",
  "type": "documentAdditionOrUpdate",
  "enqueuedAt": "2024-08-11T09:25:53.000000Z"
}
```

## Check the task status

All indexing operations in Meilisearch are [asynchronous](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations). Use the `taskUid` from the response to check whether your documents have been indexed:

```bash
curl \
  -X GET 'MEILISEARCH_URL/tasks/0' \
  -H 'Authorization: Bearer aSampleMasterKey'
```

```javascript
client.tasks.getTask(0)
```

```python
client.get_task(0)
```

```php
$client->getTask(0);
```

```java
client.getTask(0);
```

```ruby
client.task(0)
```

```go
client.GetTask(0)
```

```csharp
TaskInfo task = await client.GetTaskAsync(0);
```

```rust
client
  .get_task(0)
  .await
  .unwrap();
```

```swift
client.getTask(taskUid: 0) { (result) in
    switch result {
    case .success(let task):
        print(task)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.getTask(0);
```

A successful task returns a status of `succeeded`:

```json
{
  "uid": 0,
  "indexUid": "movies",
  "status": "succeeded",
  "type": "documentAdditionOrUpdate",
  "details": {
    "receivedDocuments": 4,
    "indexedDocuments": 4
  }
}
```

If the status is `failed`, the response includes an `error` object explaining what went wrong.

## Verify documents are searchable

Once the task succeeds, your documents are ready to search. Test with a simple query:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/search' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{ "q": "wonder" }'
```

You should see "Wonder Woman" in the results.

## Accepted document formats

| Format | Content-Type header | Notes |
| - | - | - |
| JSON | `application/json` | Array of objects. Most common format. |
| NDJSON | `application/x-ndjson` | One JSON object per line. Useful for streaming large datasets. |
| CSV | `text/csv` | First row must be column headers. All values are strings by default. |

## Next steps

> [card] [Monitor tasks](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/monitor_tasks)
>
> Track and manage asynchronous indexing operations

> [card] [Add and update documents](https://www.meilisearch.com/docs/capabilities/indexing/how_to/add_and_update_documents)
>
> Learn the difference between replacing and partially updating documents

> [card] [Indexing overview](https://www.meilisearch.com/docs/capabilities/indexing/overview)
>
> Understand how indexing works in Meilisearch

> [card] [Best practices](https://www.meilisearch.com/docs/capabilities/indexing/advanced/indexing_best_practices)
>
> Optimize your indexing performance


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.