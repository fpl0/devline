# Securing self-hosted Meilisearch

> Understand the Meilisearch security model, from master keys to API keys, and learn how to protect your instance.

Meilisearch uses a key-based authentication system to protect your data. Understanding how keys work is the first step to securing your instance.

## How authentication works

Meilisearch's security model has three layers:

1. **Master key**: a secret you set at launch. It is never used directly in API requests, but generates the default API keys
2. **API keys**: credentials used to authenticate API requests. Meilisearch creates two default keys (admin and search) when you set a master key
3. **Tenant tokens**: short-lived, client-side tokens derived from API keys. They enforce per-user search rules without exposing your API keys

```mermaid
flowchart LR
    MK[Master key] --> AK[API keys]
    AK --> TT[Tenant tokens]
    MK -.->|set at launch| MS[Meilisearch instance]
    AK -->|authenticate requests| MS
    TT -->|scoped search| MS
```

## Security checklist

For production self-hosted instances:

* [ ] Set a [master key](https://www.meilisearch.com/docs/resources/self_hosting/security/master_api_keys) of at least 16 bytes
* [ ] Set the [environment to `production`](https://www.meilisearch.com/docs/resources/self_hosting/configuration/reference#environment)
* [ ] Use HTTPS via a [reverse proxy](https://www.meilisearch.com/docs/resources/self_hosting/deployment/running_production) or [direct SSL](https://www.meilisearch.com/docs/resources/self_hosting/security/http2_ssl)
* [ ] Use the **search API key** (not the admin key) in front-end applications
* [ ] Consider [tenant tokens](https://www.meilisearch.com/docs/capabilities/security/overview) for multi-tenant search
* [ ] Restrict network access with firewall rules

## Next steps

> [card] [Master key and API keys](https://www.meilisearch.com/docs/resources/self_hosting/security/master_api_keys)
>
> Understand the difference between master key and API keys, and how to manage them.

> [card] [Secure your project](https://www.meilisearch.com/docs/resources/self_hosting/security/basic_security)
>
> Step-by-step tutorial for setting up authentication on your instance.

> [card] [Protected and unprotected instances](https://www.meilisearch.com/docs/resources/self_hosting/security/protected_unprotected)
>
> Learn what happens when your instance has no master key.

> [card] [HTTP/2 and SSL](https://www.meilisearch.com/docs/resources/self_hosting/security/http2_ssl)
>
> Configure HTTPS directly on Meilisearch without a reverse proxy.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.