# Generate a tenant token without a library

> This guide shows you the main steps when creating tenant tokens without using any libraries.

Generating tenant tokens without a library is possible, but not recommended. This guide summarizes the necessary steps.

The full process requires you to create a token header, prepare the data payload with at least one set of search rules, and then sign the token with an API key.

## Prepare token header

The token header must specify a `JWT` type and an encryption algorithm. Supported tenant token encryption algorithms are `HS256`, `HS384`, and `HS512`.

```json
{
  "alg": "HS256",
  "typ": "JWT"
}
```

## Build token payload

First, create a set of search rules:

```json
{
  "INDEX_NAME": {
    "filter": "ATTRIBUTE = VALUE"
  }
}
```

Next, find your default search API key. Query the [get API keys endpoint](https://www.meilisearch.com/docs/reference/api/keys/get-api-key) and inspect the `uid` field to obtain your API key's UID:

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

For maximum security, you should also set an expiry date for your tenant tokens. The following Node.js example configures the token to expire 20 minutes after its creation:

```js
parseInt(Date.now() / 1000) + 20 * 60
```

Lastly, assemble all parts of the payload in a single object:

```json
{
  "exp": UNIX_TIMESTAMP,
  "apiKeyUid": "API_KEY_UID",
  "searchRules": {
    "INDEX_NAME": {
      "filter": "ATTRIBUTE = VALUE"
    }
  }
}
```

Consult the [token payload reference](https://www.meilisearch.com/docs/capabilities/security/advanced/tenant_token_payload) for more information on the requirements for each payload field.

## Encode header and payload

You must then encode both the header and the payload into `base64`, concatenate them, and generate the token by signing it using your chosen encryption algorithm.

## Make a search request using a tenant token

After signing the token, you can use it to make search queries in the same way you would use an API key.

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/patient_medical_records/search' \
  -H 'Authorization: Bearer TENANT_TOKEN'
```

## Next steps

> [card] [Tenant token payload reference](https://www.meilisearch.com/docs/capabilities/security/advanced/tenant_token_payload)
>
> Detailed reference for all fields in the tenant token payload.

> [card] [Generate tokens with an SDK](https://www.meilisearch.com/docs/capabilities/security/getting_started)
>
> Use a Meilisearch SDK to generate tenant tokens with less manual work.

> [card] [Generate tokens with a third-party library](https://www.meilisearch.com/docs/capabilities/security/how_to/generate_token_third_party)
>
> Create tenant tokens using JWT libraries like jsonwebtoken.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.