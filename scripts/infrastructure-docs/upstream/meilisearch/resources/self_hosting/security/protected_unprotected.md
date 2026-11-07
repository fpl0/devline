# Protected and unprotected Meilisearch projects

> This article explains the differences between protected and unprotected Meilisearch projects and instances.

This article explains the differences between protected and unprotected Meilisearch projects and instances.

## Protected projects

In protected projects, all Meilisearch API routes and endpoints can only be accessed by requests bearing an API key. The only exception to this rule is the `/health` endpoint, which may still be queried with unauthorized requests.

**Meilisearch Cloud projects are protected by default**. Self-hosted instances are only protected if you launch them with a master key.

Consult the [basic security tutorial](https://www.meilisearch.com/docs/resources/self_hosting/security/basic_security) for instructions on how to communicate with protected projects.

## Unprotected projects

In unprotected projects and self-hosted instances, any user may access any API endpoint. Never leave a publicly accessible instance unprotected. Only use unprotected instances in safe development environments.

Meilisearch Cloud projects are always protected. Meilisearch self-hosted instances are unprotected by default.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.