# Multitenancy and tenant tokens

> This guide shows you the main steps when creating tenant tokens using Meilisearch's official SDKs.

There are two steps to use tenant tokens with an official SDK: generating the tenant token, and making a search request using that token.

## Generate a tenant token with an official SDK

First, import the SDK. Then create a set of [search rules](https://www.meilisearch.com/docs/capabilities/security/advanced/tenant_token_payload#search-rules):

```json
{
  "patient_medical_records": {
    "filter": "user_id = 1"
  }
}
```

Search rules must be an object where each key corresponds to an index in your instance. You may configure any number of filters for each index.

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

For maximum security, you should also define an expiry date for tenant tokens.

Finally, send this data to your chosen SDK's tenant token generator:

```javascript
import { generateTenantToken } from 'meilisearch/token'

const searchRules = {
  patient_medical_records: {
    filter: 'user_id = 1'
  }
}
const apiKey = 'B5KdX2MY2jV6EXfUs6scSfmC...'
const apiKeyUid = '85c3c2f9-bdd6-41f1-abd8-11fcf80e0f76'
const expiresAt = new Date('2025-12-20') // optional

const token = await generateTenantToken({ apiKey, apiKeyUid, searchRules, expiresAt })
```

```python
uid = '85c3c2f9-bdd6-41f1-abd8-11fcf80e0f76';
api_key = 'B5KdX2MY2jV6EXfUs6scSfmC...'
expires_at = datetime(2025, 12, 20)
search_rules = {
  'patient_medical_records': {
    'filter': 'user_id = 1'
  }
}
token = client.generate_tenant_token(api_key_uid=uid, search_rules=search_rules, api_key=api_key, expires_at=expires_at)
```

```php
$apiKeyUid = '85c3c2f9-bdd6-41f1-abd8-11fcf80e0f76';
$searchRules = (object) [
  'patient_medical_records' => (object) [
    'filter' => 'user_id = 1',
  ]
];
$options = [
    'apiKey' => 'B5KdX2MY2jV6EXfUs6scSfmC...',
    'expiresAt' => new DateTime('2025-12-20'),
];

$token = $client->generateTenantToken($apiKeyUid, $searchRules, $options);
```

```java
Map<String, Object> filters = new HashMap<String, Object>();
filters.put("filter", "user_id = 1");
Map<String, Object> searchRules = new HashMap<String, Object>();
searchRules.put("patient_medical_records", filters);

Date expiresAt = new SimpleDateFormat("yyyy-MM-dd").parse("2025-12-20");
TimeZone.setDefault(TimeZone.getTimeZone("UTC"));
TenantTokenOptions options = new TenantTokenOptions();
options.setApiKey("B5KdX2MY2jV6EXfUs6scSfmC...");
options.setExpiresAt(expiresAt);

String token = client.generateTenantToken("85c3c2f9-bdd6-41f1-abd8-11fcf80e0f76", searchRules, options);
```

```ruby
uid = '85c3c2f9-bdd6-41f1-abd8-11fcf80e0f76'
api_key = 'B5KdX2MY2jV6EXfUs6scSfmC...'
expires_at = Time.new(2025, 12, 20).utc
search_rules = {
  'patient_medical_records' => {
    'filter' => 'user_id = 1'
  }
}

token = client.generate_tenant_token(uid, search_rules, api_key: api_key, expires_at: expires_at)
```

```go
searchRules := map[string]interface{}{
  "patient_medical_records": map[string]string{
    "filter": "user_id = 1",
  },
}
options := &meilisearch.TenantTokenOptions{
  APIKey: "B5KdX2MY2jV6EXfUs6scSfmC...",
  ExpiresAt: time.Date(2025, time.December, 20, 0, 0, 0, 0, time.UTC),
}

token, err := client.GenerateTenantToken(searchRules, options);
```

```csharp
var apiKey = "B5KdX2MY2jV6EXfUs6scSfmC...";
var expiresAt = new DateTime(2025, 12, 20);
var searchRules = new TenantTokenRules(new Dictionary<string, object> {
  { "patient_medical_records", new Dictionary<string, object> { { "filter", "user_id = 1" } } }
});

token = client.GenerateTenantToken(
  searchRules,
  apiKey: apiKey // optional,
  expiresAt: expiresAt // optional
);
```

```rust
let api_key = "B5KdX2MY2jV6EXfUs6scSfmC...";
let api_key_uid = "6062abda-a5aa-4414-ac91-ecd7944c0f8d";
let expires_at = time::macros::datetime!(2025 - 12 - 20 00:00:00 UTC);
let search_rules = json!({ "patient_medical_records": { "filter": "user_id = 1" } });

let token = client
  .generate_tenant_token(api_key_uid, search_rules, api_key, expires_at)
  .unwrap();
```

```swift
let apiKey = "B5KdX2MY2jV6EXfUs6scSfmC..."
let expiresAt = Date.distantFuture
let searchRules = SearchRulesGroup(SearchRules("patient_medical_records", filter: "user_id = 1"))

client.generateTenantToken(
  searchRules,
  apiKey: apiKey, // optional
  expiresAt: expiresAt // optional
) { (result: Result<String, Error>) in
  switch result {
  case .success(let token):
    print(token)
  case .failure(let error):
    print(error)
  }
}
```

```dart
final uid = '85c3c2f9-bdd6-41f1-abd8-11fcf80e0f76';
final apiKey = 'B5KdX2MY2jV6EXfUs6scSfmC...';
final expiresAt = DateTime.utc(2025, 12, 20);
final searchRules = {
  'patient_medical_records': {
    'filter': 'user_id = 1'
  }
};

final token = client.generateTenantToken(
  uid,
  searchRules,
  apiKey: apiKey, // optional
  expiresAt: expiresAt // optional
);
```

The SDK will return a valid tenant token.

## Make a search request using a tenant token

After creating a token, you must send it your application's front end. Exactly how to do that depends on your specific setup.

Once the tenant token is available, use it to authenticate search requests as if it were an API key:

```javascript
const frontEndClient = new MeiliSearch({ host: 'MEILISEARCH_URL', apiKey: token })
frontEndClient.index('patient_medical_records').search('blood test')
```

```python
front_end_client = Client('MEILISEARCH_URL', token)
front_end_client.index('patient_medical_records').search('blood test')
```

```php
$frontEndClient = new Client('MEILISEARCH_URL', $token);
$frontEndClient->index('patient_medical_records')->search('blood test');
```

```java
Client frontEndClient = new Client(new Config("MEILISEARCH_URL", token));
frontEndClient.index("patient_medical_records").search("blood test");
```

```ruby
front_end_client = MeiliSearch::Client.new('MEILISEARCH_URL', token)

front_end_client.index('patient_medical_records').search('blood test')
```

```go
client := meilisearch.New("MEILISEARCH_URL", meilisearch.WithAPIKey("masterKey"))
client.Index("patient_medical_records").Search("blood test", &meilisearch.SearchRequest{});
```

```csharp
frontEndClient = new MeilisearchClient("MEILISEARCH_URL", token);
var searchResult = await frontEndClient.Index("patient_medical_records").SearchAsync<Patient>("blood test");
```

```rust
let front_end_client = Client::new("MEILISEARCH_URL", Some(token));
let results: SearchResults<Patient> = front_end_client
  .index("patient_medical_records")
  .search()
  .with_query("blood test")
  .execute()
  .await
  .unwrap();
```

```swift
let frontEndClient = MeiliSearch(host: "MEILISEARCH_URL", apiKey: token)

client.index("patient_medical_records")
  .search(parameters) { (result: Result<SearchResult<Record>, Swift.Error>) in
    switch result {
    case .success(let searchResult):
      print(searchResult)
    case .failure(let error):
      print(error)
    }
}
```

```dart
final frontEndClient = MeiliSearchClient('MEILISEARCH_URL', token);
await frontEndClient.index('patient_medical_records').search('blood test');
```

Applications may use tenant tokens and API keys interchangeably when searching. For example, the same application might use a default search API key for queries on public indexes and a tenant token for logged-in users searching on private data.

## Next steps

> [card] [Generate a token with a third-party library](https://www.meilisearch.com/docs/capabilities/security/how_to/generate_token_third_party)
>
> Create tenant tokens using JWT libraries instead of Meilisearch SDKs.

> [card] [Generate a token without any library](https://www.meilisearch.com/docs/capabilities/security/how_to/generate_token_from_scratch)
>
> Build tenant tokens manually by assembling the JWT header, payload, and signature.

> [card] [Tenant token payload reference](https://www.meilisearch.com/docs/capabilities/security/advanced/tenant_token_payload)
>
> Learn about all available fields in the tenant token payload.

> [card] [Manage API keys](https://www.meilisearch.com/docs/capabilities/security/how_to/manage_api_keys)
>
> Create, update, and delete API keys for your Meilisearch instance.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.