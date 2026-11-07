# Getting started with self-hosted Meilisearch

> Learn how to install Meilisearch, index a dataset, and perform your first search.

This quick start walks you through installing Meilisearch, adding documents, and performing your first search.

To follow this tutorial you need:

* A [command line terminal](https://www.learnenough.com/command-line-tutorial#sec-running_a_terminal)
* [cURL](https://curl.se)

> [!TIP]
>
> Using Meilisearch Cloud? Check out the dedicated guide, [Getting started with Meilisearch Cloud](https://www.meilisearch.com/docs/getting_started/first_project).

## Setup and installation

First, you need to download and install Meilisearch. This command installs the latest Meilisearch version in your local machine:

```bash
# Install Meilisearch
curl -L https://install.meilisearch.com | sh
```

The rest of this guide assumes you are using Meilisearch locally, but you may also use Meilisearch over a cloud service such as [Meilisearch Cloud](https://www.meilisearch.com/cloud).

Learn more about other installation options in the [installation guide](https://www.meilisearch.com/docs/resources/self_hosting/getting_started/install_locally).

### Running Meilisearch

Next, launch Meilisearch by running the following command in your terminal:

```bash
# Launch Meilisearch
./meilisearch --master-key="aSampleMasterKey"
```

> [!NOTE]
>
> This tutorial uses `aSampleMasterKey` as a master key, but you may change it to any alphanumeric string with 16 or more bytes. In most cases, one character corresponds to one byte.

You should see something like this in response:

```
888b     d888          d8b 888 d8b                                            888
8888b   d8888          Y8P 888 Y8P                                            888
88888b.d88888              888                                                888
888Y88888P888  .d88b.  888 888 888 .d8888b   .d88b.   8888b.  888d888 .d8888b 88888b.
888 Y888P 888 d8P  Y8b 888 888 888 88K      d8P  Y8b     "88b 888P"  d88P"    888 "88b
888  Y8P  888 88888888 888 888 888 "Y8888b. 88888888 .d888888 888    888      888  888
888   "   888 Y8b.     888 888 888      X88 Y8b.     888  888 888    Y88b.    888  888
888       888  "Y8888  888 888 888  88888P'  "Y8888  "Y888888 888     "Y8888P 888  888

Database path:       "./data.ms"
Server listening on: "localhost:7700"
```

You now have a Meilisearch instance running in your terminal window. Keep this window open for the rest of this tutorial.

> [!NOTE]
>
> The above command uses the `--master-key` configuration option to secure Meilisearch. Setting a master key is optional but strongly recommended in development environments. Master keys are mandatory in production environments.

To learn more about securing Meilisearch, refer to the [security tutorial](https://www.meilisearch.com/docs/resources/self_hosting/security/basic_security).

## Add documents

In this quick start, you will search through a collection of movies.

To follow along, first click this link to download the file: [movies.json](https://www.meilisearch.com/docs/assets/datasets/movies.json). Then, move the downloaded file into your working directory.

> [!NOTE]
>
> Meilisearch accepts data in JSON, NDJSON, and CSV formats.

Open a new terminal window and run the following command:

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

Meilisearch stores data in the form of discrete records, called [documents](https://www.meilisearch.com/docs/resources/internals/documents). Each document is an object composed of multiple fields, which are pairs of one attribute and one value:

```json
{
  "attribute": "value"
}
```

Documents are grouped into collections, called [indexes](https://www.meilisearch.com/docs/resources/internals/indexes).

The previous command added documents from `movies.json` to a new index called `movies`. It also set `id` as the primary key.

> [!NOTE]
>
> Every index must have a [primary key](https://www.meilisearch.com/docs/resources/internals/primary_key#primary-field), an attribute shared across all documents in that index. If you try adding documents to an index and even a single one is missing the primary key, none of the documents will be stored.
>
> If you do not explicitly set the primary key, Meilisearch [infers](https://www.meilisearch.com/docs/resources/internals/primary_key#meilisearch-guesses-your-primary-key) it from your dataset.

After adding documents, you should receive a response like this:

```json
{
    "taskUid": 0,
    "indexUid": "movies",
    "status": "enqueued",
    "type": "documentAdditionOrUpdate",
    "enqueuedAt": "2021-08-11T09:25:53.000000Z"
}
```

Use the returned `taskUid` to [check the status](https://www.meilisearch.com/docs/reference/api/tasks/get-task) of your documents:

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

> [!NOTE]
>
> Most database operations in Meilisearch are [asynchronous](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations). Rather than being processed instantly, **API requests are added to a queue and processed one at a time**.

If the document addition is successful, the response should look like this:

```json
{
   "uid": 0,
   "indexUid": "movies",
   "status": "succeeded",
   "type": "documentAdditionOrUpdate",
   "canceledBy": null,
   "details": {
      "receivedDocuments": 19547,
      "indexedDocuments": 19547
   },
   "error": null,
   "duration": "PT0.030750S",
   "enqueuedAt": "2021-12-20T12:39:18.349288Z",
   "startedAt": "2021-12-20T12:39:18.352490Z",
   "finishedAt": "2021-12-20T12:39:18.380038Z"
}
```

If `status` is `enqueued` or `processing`, all you have to do is wait a short time and check again. Proceed to the next step once the task `status` has changed to `succeeded`.

## Search

Now that you have Meilisearch set up, you can start searching!

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/search' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer aSampleMasterKey' \
  --data-binary '{ "q": "botman" }'
```

```javascript
client.index('movies').search('botman').then((res) => console.log(res))
```

```python
client.index('movies').search('botman')
```

```php
$client->index('movies')->search('botman');
```

```java
client.index("movies").search("botman");
```

```ruby
client.index('movies').search('botman')
```

```go
client.Index("movies").Search("botman", &meilisearch.SearchRequest{})
```

```csharp
MeilisearchClient client = new MeilisearchClient("MEILISEARCH_URL", "masterKey");
var index = client.Index("movies");

var movies = await index.SearchAsync<Movie>("botman");
foreach (var movie in movies.Hits)
{
    Console.WriteLine(movie.Title);
}
```

```rust
// You can build a `SearchQuery` and execute it later:
let query: SearchQuery = SearchQuery::new(&movies)
  .with_query("botman")
  .build();

let results: SearchResults<Movie> = client
  .index("movies")
  .execute_query(&query)
  .await
  .unwrap();

// You can build a `SearchQuery` and execute it directly:
let results: SearchResults<Movie> = SearchQuery::new(&movies)
  .with_query("botman")
  .execute()
  .await
  .unwrap();

// You can search in an index directly:
let results: SearchResults<Movie> = client
  .index("movies")
  .search()
  .with_query("botman")
  .execute()
  .await
  .unwrap();
```

```swift
client.index("movies").search(SearchParameters(query: "botman")) { (result) in
    switch result {
    case .success(let searchResult):
        print(searchResult)
    case .failure(let error):
        print(error)
    }
}
```

```dart
await client.index('movies').search('botman');
```

> [!NOTE]
>
> This tutorial queries Meilisearch with the master key. In production environments, this is a security risk. Prefer using API keys to access Meilisearch's API in any public-facing application.

In the above code sample, the parameter `q` represents the search query. This query instructs Meilisearch to search for `botman` in the documents you added in [the previous step](#add-documents):

```json
{
  "hits": [
    {
      "id": 29751,
      "title": "Batman Unmasked: The Psychology of the Dark Knight",
      "poster": "https://image.tmdb.org/t/p/w1280/jjHu128XLARc2k4cJrblAvZe0HE.jpg",
      "overview": "Delve into the world of Batman and the vigilante justice tha",
      "release_date": "2008-07-15"
    },
    {
      "id": 471474,
      "title": "Batman: Gotham by Gaslight",
      "poster": "https://image.tmdb.org/t/p/w1280/7souLi5zqQCnpZVghaXv0Wowi0y.jpg",
      "overview": "ve Victorian Age Gotham City, Batman begins his war on crime",
      "release_date": "2018-01-12"
    },
    …
  ],
  "estimatedTotalHits": 66,
  "query": "botman",
  "limit": 20,
  "offset": 0,
  "processingTimeMs": 12
}
```

By default, Meilisearch only returns the first 20 results for a search query. You can change this using the [`limit` parameter](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-limit).

## What's next?

You now know how to install Meilisearch, create an index, add documents, check the status of an asynchronous task, and make a search request.

If you'd like to search through the documents you just added using a clean browser interface rather than the terminal, you can do so with [our built-in search preview](https://www.meilisearch.com/docs/resources/self_hosting/getting_started/search_preview). You can also [learn how to quickly build a front-end interface](https://www.meilisearch.com/docs/getting_started/instant_meilisearch/javascript) of your own.

For a more advanced approach, consult the [API reference](https://www.meilisearch.com/docs/reference/api/requests).


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.