# Master key and API keys

> Understand the differences between master key and API keys, and how to manage them in self-hosted Meilisearch.

This guide explains the differences between the master key and API keys, and how to manage them in self-hosted Meilisearch instances.

## Master key

The master key grants full control over your Meilisearch instance. It is the only key with access to endpoints for creating and deleting API keys by default. Since the master key is not an API key, it cannot be listed or configured through the `/keys` endpoints.

> [!WARNING]
>
> Exposing the master key can give malicious users complete control over your Meilisearch instance. **Only use the master key when managing API keys**, never for regular operations.

### Setting the master key

Launch Meilisearch with a master key to protect your instance:

> [tab] Command line
>
> ```bash theme={null}
> meilisearch --master-key="your-master-key-here"
> ```

> [tab] Environment variable
>
> ```bash theme={null}
> export MEILI_MASTER_KEY="your-master-key-here"
> meilisearch
> ```

> [!NOTE]
>
> Your master key must be at least 16 bytes. Use a secure, randomly generated string.

### Resetting the master key

If your master key is compromised, reset it by relaunching your instance with a new value:

```bash
meilisearch --master-key="new-master-key-here"
```

> [!WARNING]
>
> Resetting the master key automatically invalidates all existing API keys. You will need to create new API keys after resetting.

## API keys

API keys grant access to a specific set of indexes, routes, and endpoints. You can configure them to expire after a certain date. Use the [`/keys` route](https://www.meilisearch.com/docs/reference/api/keys/list-api-keys) to create, configure, and delete API keys.

**Use API keys for all API operations except API key management.** This includes:

* Searching documents
* Adding and updating documents
* Configuring index settings
* Managing indexes

### Default API keys

When you launch Meilisearch with a master key, four default API keys are automatically created:

| Key | Purpose | Permissions |
| - | - | - |
| Default Search API Key | Frontend search queries | Search only, all indexes |
| Default Admin API Key | Backend operations | Full access except key management |
| Default Read-Only Admin API Key | Read-only access | Read-only access to all indexes, documents, and settings |
| Default Chat API Key | Frontend conversational search | Search and chat completions, all indexes |

In most cases, these default keys are sufficient:

* Use the **Default Search API Key** for client-side search
* Use the **Default Admin API Key** for server-side operations (do not expose on a public frontend)
* Use the **Default Read-Only Admin API Key** for read-only access to all indexes, documents, and settings (do not expose on a public frontend)
* Use the **Default Chat API Key** for the [Chats API](https://www.meilisearch.com/docs/capabilities/agentic_search/advanced/chats_api#find-your-chat-api-key) (can be safely used from the frontend)

### Creating custom API keys

Create custom API keys for more granular control:

```bash
curl -X POST "${MEILISEARCH_URL}/keys" \
  -H "Authorization: Bearer ${MASTER_KEY}" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Search key for products index",
    "actions": ["search"],
    "indexes": ["products"],
    "expiresAt": "2025-12-31T23:59:59Z"
  }'
```

## Best practices

1. **Never expose the master key** in client-side code or public repositories
2. **Use API keys** for all regular operations
3. **Limit API key permissions** to only what's needed
4. **Set expiration dates** on API keys when appropriate
5. **Rotate keys regularly** in production environments

## Related resources

> [card] [API keys reference](https://www.meilisearch.com/docs/reference/api/keys/list-api-keys)
>
> Full API documentation for key management

> [card] [Security overview](https://www.meilisearch.com/docs/resources/self_hosting/security/basic_security)
>
> Learn about Meilisearch security model


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.