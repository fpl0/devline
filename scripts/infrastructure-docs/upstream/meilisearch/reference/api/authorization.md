# Authorization

> How to authenticate with the Meilisearch API using API keys and the Authorization header.

> [!TIP]
>
> If you are new to Meilisearch, check out the [getting started guide](https://www.meilisearch.com/docs/resources/self_hosting/getting_started/quick_start).

By [providing Meilisearch with a master key at launch](https://www.meilisearch.com/docs/resources/self_hosting/security/basic_security), you protect your instance from unauthorized requests. The provided master key must be at least 16 bytes. From then on, you must include the `Authorization` header along with a valid API key to access protected routes (all routes except [`/health`](https://www.meilisearch.com/docs/reference/api/health)).

```bash
curl \
  -X GET 'MEILISEARCH_URL/keys' \
  -H 'Authorization: Bearer MASTER_KEY'
```

```javascript
const client = new MeiliSearch({ host: 'MEILISEARCH_URL', apiKey: 'masterKey' })
client.getKeys()
```

```python
client = Client('MEILISEARCH_URL', 'masterKey')
client.get_keys()
```

```php
$client = new Client('MEILISEARCH_URL', 'masterKey');
$client->getKeys();
```

```java
Client client = new Client(new Config("MEILISEARCH_URL", "masterKey"));
client.getKeys();
```

```ruby
client = MeiliSearch::Client.new('MEILISEARCH_URL', 'masterKey')
client.keys
```

```go
client := meilisearch.New("MEILISEARCH_URL", meilisearch.WithAPIKey("masterKey"))
client.GetKeys(nil);
```

```csharp
MeilisearchClient client = new MeilisearchClient("MEILISEARCH_URL", "masterKey");
var keys = await client.GetKeysAsync();
```

```rust
let client = Client::new("MEILISEARCH_URL", Some("MASTER_KEY")); let keys = client .get_keys() .await .unwrap();
```

```swift
client = try MeiliSearch(host: "MEILISEARCH_URL", apiKey: "masterKey")
client.getKeys { result in
    switch result {
    case .success(let keys):
        print(keys)
    case .failure(let error):
        print(error)
    }
}
```

```dart
var client = MeiliSearchClient('MEILISEARCH_URL', 'masterKey');
await client.getKeys();
```

The [`/keys`](https://www.meilisearch.com/docs/reference/api/keys) route can only be accessed using the master key. For security reasons, we recommend using regular API keys for all other routes.

[To learn more about keys and security, refer to our security tutorial.](https://www.meilisearch.com/docs/resources/self_hosting/security/basic_security)


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.