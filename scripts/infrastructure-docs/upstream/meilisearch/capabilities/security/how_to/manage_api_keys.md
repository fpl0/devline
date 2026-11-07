# Manage API keys

> Create, rotate, and scope API keys to control access to your Meilisearch instance.

API keys control who can access your Meilisearch instance and what actions they can perform. Each key has specific permissions and can be scoped to specific indexes. For multi-tenant scenarios, consider using [tenant tokens](https://www.meilisearch.com/docs/capabilities/security/overview) to restrict search results per user.

## API key types

Meilisearch provides several types of API keys:

| Key type | Purpose | Usage |
| :- | :- | :- |
| Default admin key | Full API access | Day-to-day admin operations |
| Default search key | Search-only access | Client-side search requests |
| Custom API keys | Scoped permissions | Fine-grained access control |

> [!WARNING]
>
> Never expose admin API keys in client-side code or public repositories. Use them only server-side to manage API keys through the `/keys` endpoint, then use search or scoped API keys for all other operations.

## List all API keys

Retrieve all existing API keys. This endpoint requires the admin API key.

```bash
curl \
  -X GET 'MEILISEARCH_URL/keys' \
  -H 'Authorization: Bearer MEILISEARCH_KEY'
```

The response includes each key's `uid`, `key`, `actions`, `indexes`, `expiresAt`, and timestamps.

## Create an API key

Create a new key with specific permissions. Specify which `actions` the key can perform and which `indexes` it can access.

```bash
curl \
  -X POST 'MEILISEARCH_URL/keys' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "description": "Search-only key for products index",
    "actions": ["search"],
    "indexes": ["products"],
    "expiresAt": "2026-12-31T00:00:00Z"
  }'
```

> [!WARNING]
>
> For security reasons, we do not recommend creating keys that can perform all actions (`"actions": ["*"]`). Scope each key to the narrowest set of actions it needs so that a compromised key cannot be used to take over your instance.

### Available actions

Actions define what operations a key can perform:

| Action | Description |
| :- | :- |
| `*` | All operations (admin-level access) |
| `search` | Search within allowed indexes |
| `documents.add` | Add or replace documents |
| `documents.get` | Retrieve documents |
| `documents.delete` | Delete documents |
| `indexes.create` | Create new indexes |
| `indexes.get` | Retrieve index information |
| `indexes.update` | Update index settings |
| `indexes.delete` | Delete indexes |
| `indexes.swap` | Swap two indexes |
| `tasks.get` | Retrieve task information |
| `tasks.cancel` | Cancel pending tasks |
| `tasks.delete` | Delete finished tasks |
| `settings.get` | Retrieve index settings |
| `settings.update` | Update index settings |
| `stats.get` | Retrieve instance statistics |
| `dumps.create` | Create database dumps |
| `snapshots.create` | Create database snapshots |
| `version` | Retrieve version information |
| `keys.get` | Retrieve API key information |
| `keys.create` | Create new API keys |
| `keys.update` | Update existing API keys |
| `keys.delete` | Delete API keys |

### Scope keys to specific indexes

The `indexes` field accepts an array of index UIDs. Use `["*"]` to grant access to all indexes, or specify individual ones:

```bash
curl \
  -X POST 'MEILISEARCH_URL/keys' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "description": "Documents admin for products and reviews",
    "actions": ["documents.add", "documents.get", "documents.delete"],
    "indexes": ["products", "reviews"],
    "expiresAt": null
  }'
```

Setting `expiresAt` to `null` creates a key that never expires.

> [!WARNING]
>
> The `actions`, `indexes`, and `expiresAt` fields cannot be changed after a key is created. If you create a key without an expiration date, you cannot add one later. If you need different permissions or expiration, delete the key and create a new one.

## Update an API key

You can update a key's `name` and `description`. The `actions`, `indexes`, and `expiresAt` fields cannot be modified after creation. If you need different permissions, create a new key instead.

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/keys/API_KEY_UID' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "name": "Products search key",
    "description": "Updated description for the products search key"
  }'
```

Replace `API_KEY_UID` with the key's `uid` value (not the key itself).

> [!TIP]
>
> Custom API keys are deterministic: `key` is a SHA256 hash of the `uid` and the master key. To reuse the exact same custom API keys on a new instance, launch that instance with the same master key and recreate each key with the same `uid`. This is useful when restoring from a backup, cloning an environment, or rotating instances without changing client code.

## Delete an API key

Permanently revoke a key by deleting it. Any requests using this key will be rejected immediately.

```bash
curl \
  -X DELETE 'MEILISEARCH_URL/keys/API_KEY_UID' \
  -H 'Authorization: Bearer MEILISEARCH_KEY'
```

## Key rotation

Regularly rotating API keys reduces the risk of compromised credentials. To rotate a key:

1. Create a new key with the same `actions` and `indexes` as the old one
2. Update your application to use the new key
3. Verify that the application works correctly with the new key
4. Delete the old key

Use the `expiresAt` field to enforce automatic expiration. When a key expires, all requests using it will return a `403` error.

```bash
curl \
  -X POST 'MEILISEARCH_URL/keys' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer MEILISEARCH_KEY' \
  --data-binary '{
    "description": "Rotating search key - Q1 2026",
    "actions": ["search"],
    "indexes": ["*"],
    "expiresAt": "2026-04-01T00:00:00Z"
  }'
```

> [!TIP]
>
> Set `expiresAt` to a date in the near future (for example, 90 days) and schedule key rotation before expiration. This limits the window of exposure if a key is compromised.

## Best practices

* **Use the principle of least privilege.** Give each key only the permissions it needs. A front-end search client should only have the `search` action.
* **Scope keys to specific indexes.** Avoid using `["*"]` for indexes unless the key genuinely needs access to all of them.
* **Set expiration dates.** Keys without expiration dates remain valid indefinitely, which increases security risk.
* **Never expose admin API keys.** Only use them server-side to manage API keys. Use search or scoped API keys for all other operations.
* **Rotate keys regularly.** Create new keys before old ones expire and update your applications accordingly.

## Next steps

> [card] [Security overview](https://www.meilisearch.com/docs/capabilities/security/overview)
>
> Learn about tenant tokens and multi-tenancy

> [card] [Keys API reference](https://www.meilisearch.com/docs/reference/api/keys)
>
> Full API reference for the `/keys` endpoint


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.