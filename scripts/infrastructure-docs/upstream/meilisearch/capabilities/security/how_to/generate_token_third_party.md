# Generate tenant tokens without a Meilisearch SDK

> This guide shows you the main steps when creating tenant tokens without using Meilisearch's official SDKs.

This guide shows you the main steps when creating tenant tokens using [`node-jsonwebtoken`](https://www.npmjs.com/package/jsonwebtoken), a third-party library.

## Generate a tenant token with `jsonwebtoken`

### Build the tenant token payload

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

For maximum security, you should also set an expiry date for your tenant tokens. The following example configures the token to expire 20 minutes after its creation:

```js
parseInt(Date.now() / 1000) + 20 * 60
```

### Create tenant token

First, include `jsonwebtoken` in your application. Next, assemble the token payload and pass it to `jsonwebtoken`'s `sign` method:

```js
const jwt = require('jsonwebtoken');

const apiKey = 'API_KEY';
const apiKeyUid = 'API_KEY_UID';
const currentUserID = 'USER_ID';
const expiryDate = parseInt(Date.now() / 1000) + 20 * 60; // 20 minutes

const tokenPayload = {
  searchRules: {
    'INDEX_NAME': {
      'filter': `user_id = ${currentUserID}`
     }
  },
  apiKeyUid: apiKeyUid,
  exp: expiryDate
};

const token = jwt.sign(tokenPayload, apiKey, {algorithm: 'HS256'});
```

`sign` requires the payload, a Meilisearch API key, and an encryption algorithm. Meilisearch supports the following encryption algorithms: `HS256`, `HS384`, and `HS512`.

Your tenant token is now ready to use.

> [!NOTE]
>
> Though this example used `jsonwebtoken`, a Node.js package, you may use any JWT-compatible library in whatever language you feel comfortable.

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

> [card] [Generate tokens without any library](https://www.meilisearch.com/docs/capabilities/security/how_to/generate_token_from_scratch)
>
> Build tenant tokens manually by assembling the JWT header, payload, and signature.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.