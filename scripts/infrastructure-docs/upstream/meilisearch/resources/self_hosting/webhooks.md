# Using task webhooks

> Learn how to use webhooks to react to changes in your Meilisearch database.

This guide teaches you how to configure a single webhook via instance options to notify a URL when Meilisearch completes a [task](https://www.meilisearch.com/docs/capabilities/indexing/tasks_and_batches/async_operations).

> [!TIP]
>
> If you are using Meilisearch Cloud or need to configure multiple webhooks, use the [`/webhooks` API route](https://www.meilisearch.com/docs/reference/api/webhooks) instead.

## Requirements

* a command-line console
* a self-hosted Meilisearch instance
* a server configured to receive `POST` requests with an ndjson payload

## Configure the webhook URL

> [!WARNING]
>
> 🚩 To be able to configure a webhook to notify internal services (such as `localhost`), you will need to [allow requests on private networks](https://www.meilisearch.com/docs/resources/self_hosting/configuration/reference#allow-requests-to-private-networks-experimental). 🚩

Restart your Meilisearch instance and provide the webhook URL to `--task-webhook-URL`:

```sh
meilisearch --task-webhook-url http://localhost:8000
```

You may also define the webhook URL with environment variables or in the configuration file with `MEILI_TASK_WEBHOOK_URL`.

## Limits and constraints

> [!NOTE]
>
> You can create up to 20 webhooks per instance via the [`/webhooks` API route](https://www.meilisearch.com/docs/reference/api/webhooks). Having many webhooks active at the same time may negatively impact performance, so only register the webhooks you actively need.

> [!WARNING]
>
> The value of `Authorization` headers is redacted in responses from `GET /webhooks` and `GET /webhooks/{uuid}`. Do not use the redacted header values returned by Meilisearch when updating a webhook, or the webhook will start sending invalid credentials to your endpoint. Store the original secret on your side and resend it explicitly whenever you patch the webhook.

> [!NOTE]
>
> Meilisearch Cloud may create internal webhooks to support features such as Analytics and monitoring. These Cloud-reserved webhooks are always returned with `isEditable: false` and cannot be updated or deleted through the API.

## Optional: configure an authorization header and allow requests on private networks

Depending on your setup, you may need to provide an authorization header and allow requests on private networks.
Provide these using `task-webhook-authorization-header` and `experimental-allowed-ip-networks`:

```sh
meilisearch \
  --task-webhook-url http://localhost:8000 \
  --task-webhook-authorization-header Bearer aSampleMasterKey \
  --experimental-allowed-ip-networks 127.0.0.0/8
```

## Test the webhook

A common asynchronous operation is adding or updating documents to an index. The following example adds a test document to our `movies` index:

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

When Meilisearch finishes indexing this document, it will send a `POST` request the URL you configured with `--task-webhook-url`. The request body will be one or more task objects in [ndjson](https://github.com/ndjson/ndjson-spec) format:

```ndjson
{"uid":4,"batchUid":4,"indexUid":"movies","status":"succeeded","type":"documentAdditionOrUpdate","canceledBy":null,"details":{"receivedDocuments":1,"indexedDocuments":1},"duration":"PT0.001192S","enqueuedAt":"2022-08-04T12:28:15.159167Z","startedAt":"2022-08-04T12:28:15.161996Z","finishedAt":"2022-08-04T12:28:15.163188Z"}
```

If Meilisearch has batched multiple tasks, it will only trigger the webhook once all tasks in a batch are finished. In this case, the response payload will include all tasks, each separated by a new line:

```ndjson
{"uid":4,"batchUid":4,"indexUid":"movies","status":"succeeded","type":"documentAdditionOrUpdate","canceledBy":null,"details":{"receivedDocuments":1,"indexedDocuments":1},"duration":"PT0.001192S","enqueuedAt":"2022-08-04T12:28:15.159167Z","startedAt":"2022-08-04T12:28:15.161996Z","finishedAt":"2022-08-04T12:28:15.163188Z"}
{"uid":5,"batchUid":4,"indexUid":"movies","status":"succeeded","type":"documentAdditionOrUpdate","canceledBy":null,"details":{"receivedDocuments":1,"indexedDocuments":1},"duration":"PT0.001192S","enqueuedAt":"2022-08-04T12:28:15.159167Z","startedAt":"2022-08-04T12:28:15.161996Z","finishedAt":"2022-08-04T12:28:15.163188Z"}
{"uid":6,"batchUid":4,"indexUid":"movies","status":"succeeded","type":"documentAdditionOrUpdate","canceledBy":null,"details":{"receivedDocuments":1,"indexedDocuments":1},"duration":"PT0.001192S","enqueuedAt":"2022-08-04T12:28:15.159167Z","startedAt":"2022-08-04T12:28:15.161996Z","finishedAt":"2022-08-04T12:28:15.163188Z"}
```


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.