# Infrastructure References 2026.11.07

Devline snapshot: 2026.11.07+devline.4

Source: https://github.com/fpl0/devline/tree/main/scripts/infrastructure-docs/upstream

Source snapshot date: 2026-11-07. Devline's setup instructions are published separately as Devline Help.

## PostgreSQL 18.6

PostgreSQL Global Development Group. Rendered manual from the exact [PostgreSQL 18.6 documentation release archive](https://apt.postgresql.org/pub/latest/postgresql-18.6-docs.tar.gz). The [REL_18_6 copyright notice](https://github.com/postgres/postgres/blob/REL_18_6/COPYRIGHT) permits redistribution of the documentation. The per-page links use the publisher's PostgreSQL 18 documentation URLs, which may track a later 18.x patch online.

Coverage: Complete English HTML manual for 18.6, including the tutorial, SQL language, administration, interfaces, tools, extensions, and release notes.

Omissions: Separate PDF and duplicate man page formats are outside the searchable snapshot.

License: PostgreSQL. See `POSTGRES-COPYRIGHT`.

## MySQL 8.4.11

Oracle and Docker Official Images contributors. Oracle's MySQL 8.4.11 Community distribution includes GPL-2.0-only man pages at https://github.com/mysql/mysql-server/tree/99960bf74fa919347e4f4e3ca47672f333d6e91f/man. The Docker Official Image manual comes from https://github.com/docker-library/docs/tree/08674146b7122fa57ed72ac70860f74c47d68697/mysql. Its template placeholders were filled with the upstream Compose example and image name, and the logo was omitted. The Docker image manual applies across the Docker Official Image tags.

Coverage: The Docker Official Image manual and 27 MySQL 8.4.11 man pages cover initialization, configuration, command-line SQL, administration, table maintenance, logs, backup, and restore.

Omissions: Oracle's full online SQL, storage engine, replication, and connector manuals have separate redistribution terms and are linked online. NDB Cluster and standalone MySQL Router tools are outside the managed image scope.

License: GPL-2.0-only (Oracle man pages) and MIT (Docker Official Image manual). See `MYSQL-LICENSE-MANPAGES`, `MYSQL-LICENSE-DOCKER`.

## RabbitMQ 4.3.7

RabbitMQ documentation contributors and Broadcom. RabbitMQ website source commit 3a9290749b67dfb9d6658ab7f6cc3f92090ca9d4 (https://github.com/rabbitmq/rabbitmq-website/tree/3a9290749b67dfb9d6658ab7f6cc3f92090ca9d4/versioned_docs/version-4.3). The publisher's 4.3 edition covers the managed 4.3 series and labels its landing page 4.3.6. Rendered with the publisher's Docusaurus build, with article bodies and eligible local illustrations extracted, tabs flattened under their original labels, and highlighted code retained as plain code. Unversioned RabbitMQ documentation links are directed to the matching 4.3 article when the destination and any named heading are available in this snapshot. The dynamic C# examples from the publisher website’s .NET tutorials are expanded from the official RabbitMQ Tutorials source at commit f8d497dc67b4c2f92d4d443c3042a973714414b4 (https://github.com/rabbitmq/rabbitmq-tutorials/tree/f8d497dc67b4c2f92d4d443c3042a973714414b4), using the exact source-file and line ranges requested by the pinned tutorial Markdown. The sample code is Apache-2.0.

Coverage: The RabbitMQ 4.3 server manual, including configuration, AMQP, management HTTP API, reliability, security, backup and troubleshooting, plus the publisher's RabbitMQ tutorials and client library guides.

Omissions: Patch-specific 4.3.7 release notes (https://github.com/rabbitmq/rabbitmq-server/releases/tag/v4.3.7), other manual editions, the blog, separately published client API and protocol specifications, illustrations the offline renderer cannot embed, and interactive media are outside this snapshot.

License: Apache-2.0 OR MPL-2.0. See `RABBITMQ-LICENSE-Apache-2.0`, `RABBITMQ-LICENSE-MPL-2.0`, `RABBITMQ-NOTICE`, `RABBITMQ-LICENSE.txt`.

## Redis 8.10.1

Redis Ltd. and Redis contributors. Redis 8.10.1 source documentation and machine-readable command declarations come from https://github.com/redis/redis/tree/3399357e7c17b668289386b8a15a3037bc4527b1. The imported command list matches COMMAND DOCS from Devline's pinned image sha256:335368721d328df9bf4779e9af816b0c8d3a87eddf1865fad80402b6b468f426, run with external networking disabled. Command syntax was captured from redis-cli help. The source declarations supply summaries, argument metadata, history, and reply schemas. Configuration samples and captured CLI help are shown as preformatted blocks. The supplementary guides come from the 2025-06-23 snapshot at https://github.com/redis/redis-doc/tree/1044224477fceacc5af56a636b0a7495d19e533c. Devline labels these guides by year, removes Hugo presentation markup, retains included CLI examples, and resolves legacy links and anchors to the vendored articles where possible. The archived guides use the separate CC BY-SA terms.

Coverage: All 449 commands exposed by the pinned Redis image, including vector sets, with server and Sentinel configuration, CLI help, and 61 official guides on data types, persistence, transactions, scripting, security, and operations.

Omissions: The archived guides predate 8.10.1 and omit interactive client examples and illustrations. The current Redis website manual and separately shipped modules are outside this snapshot.

License: AGPL-3.0-only option (Redis 8.10.1 source reference) and CC-BY-SA-4.0 (archived Redis guides). See `REDIS-LICENSE-REDIS-SERVER`, `REDIS-COPYRIGHT-REDIS-SERVER`, `REDIS-LICENSE-REDIS-GUIDES`, `REDIS-COPYRIGHT-REDIS-GUIDES`.

## MongoDB 8.0.34

MongoDB, Inc. and Docker Official Images contributors. MongoDB server 8.0.34 (source tag https://github.com/mongodb/mongo/tree/a1cb58fa746233febef0da3e8034533ed4320c22), MongoDB Shell 2.14.0 (https://github.com/mongodb-js/mongosh/tree/b3cedd7573ce3e7954699be46fe490a25048092e), and MongoDB Database Tools 100.20.0 (https://github.com/mongodb/mongo-tools/tree/7536fd7f3d67f5115de3eb2e271b327b7ce329b9) help were captured from Devline's pinned image sha256:4ceaf589fde84b5581be60626bff57ab8d48b1eb66ab4eeb6a807908546ff2f9 with external networking disabled. Devline wrapped verbatim CLI help in preformatted blocks and presented built-in mongosh help signatures and descriptions as Markdown. The image's server build reports gitVersion 51db4515a6597aedcf6188ee4e48a27e65b065ae. The Docker Official Image manual applies across tags and comes from https://github.com/docker-library/docs/tree/08674146b7122fa57ed72ac70860f74c47d68697/mongo. Devline substituted the upstream Compose example and image name into that manual and omitted the logo.

Coverage: The image manual and exact-version server, shell, and Database Tools help cover initialization, configuration, connections, CRUD and aggregation methods, maintenance, and logical backups and restores.

Omissions: The full 8.0 database manual, query and aggregation operator guides, driver manuals, and specifications remain online under the publisher's separate documentation terms.

License: SSPL-1.0 (server CLI help), Apache-2.0 (shell and Database Tools help), and MIT (Docker Official Image manual). See `MONGODB-LICENSE-SERVER`, `MONGODB-LICENSE-MONGOSH`, `MONGODB-LICENSE-MONGOSH-I18N`, `MONGODB-LICENSE-DATABASE-TOOLS`, `MONGODB-LICENSE-DOCKER`.

## NATS 2.15.2

NATS Authors. NATS documentation commit 698b2b8a5add153062e7d8d58b5599d5a1609e3d (https://github.com/nats-io/nats.docs.v2/tree/698b2b8a5add153062e7d8d58b5599d5a1609e3d). The publisher's 2.15-series reference is generated using nats-server v2.15.0 and jsm.go v0.5.0 and applies to the managed 2.15 series. The publisher's Docusaurus build expands original client examples. Devline extracts article bodies, flattens tabs under their original labels, and retains highlighted examples as plain code. See NOTICE-EXAMPLES for source snapshots and client example licensing.

Coverage: NATS concepts, tutorials and Learn guides, plus the 2.15 server configuration, protocols, monitoring, authentication, JetStream APIs, operational reference, and version-series upgrade guides.

Omissions: Patch-specific 2.15.2 release notes (https://github.com/nats-io/nats-server/releases/tag/v2.15.2), other reference versions, linked external content, and interactive demonstrations are outside this snapshot.

License: MIT and Apache-2.0. See `NATS-LICENSE`, `NATS-LICENSE-APACHE`, `NATS-NOTICE`, `NATS-NOTICE-EXAMPLES`, `NATS-LICENSE-nats-io-nats.net-main`, `NATS-LICENSE-nats-io-nats.rs-main`.

## Mailpit 1.33.0

Ralph Slooten and Mailpit contributors. The Mailpit website source at 8d2c4706bb30744fac8cc58f72ce6e65e496aba9 (https://github.com/axllent/mailpit-website/tree/8d2c4706bb30744fac8cc58f72ce6e65e496aba9/content/docs) is an unversioned documentation snapshot from the release date of Mailpit 1.33.0. The article bodies are extracted from the publisher's Hugo v0.169.0 build after shortcode expansion. The complete REST API operation descriptions and schema are formatted from the OpenAPI 2.0 artifact shipped at the exact 1.33.0 release commit 364916b15e80a8bac1e7b0ab66a4f2b3d7d745c2 (https://github.com/axllent/mailpit/blob/364916b15e80a8bac1e7b0ab66a4f2b3d7d745c2/server/ui/api/v1/swagger.json). The static API keeps the upstream Swagger operation anchors, and an older link to the dedicated send API authentication section targets its current heading.

Coverage: Mailpit's official documentation for installation, configuration, integrations, usage and troubleshooting, plus the complete REST API v1 operation and schema reference from the managed release.

Omissions: Interactive Swagger controls and linked external content are outside this snapshot.

License: MIT. See `MAILPIT-LICENSE-WEBSITE`, `MAILPIT-LICENSE-API`.

## Meilisearch 1.56.1

Meilisearch (Meili SAS and contributors). Official publisher Markdown exports at https://www.meilisearch.com/docs/ on 2026-11-07, from the MIT documentation repository https://github.com/meilisearch/documentation/tree/0ad8e464d2a469a2fc6fd53566111a0da1dc1895. The repository’s bundled OpenAPI spec identifies version 1.56.1. Devline removed the duplicated export index preface and normalized fenced-code attributes and export links. The OpenAPI material remains in the publisher's YAML representation. Devline preserved the published OpenAPI YAML as a code block with in-document field anchors, and repaired selected internal links to their corresponding published fields and headings.

Coverage: Self-hosting, configuration, security, troubleshooting, data backup, search and indexing capabilities, SDK quick starts, errors, and the published REST endpoint reference for the 1.56.1 OpenAPI snapshot.

Omissions: Cloud operations, Cloud analytics, Enterprise sharding guides, most older release notes, hosted assets, interactive API consoles, and complete SDK API manuals are outside this snapshot.

License: MIT. See `MEILISEARCH-LICENSE`.

## SeaweedFS S3 4.52

SeaweedFS contributors. Tagged upstream sources at https://github.com/seaweedfs/seaweedfs/tree/4.52 (984eac6a790be89e2a11de175a7413e1e3e32dd3). CLI reference reproduces weed help from the publisher's Darwin ARM64 4.52 release binary (https://github.com/seaweedfs/seaweedfs/releases/tag/4.52), with detected host and temporary directory defaults replaced by placeholders. Configuration examples are the tagged files wrapped in Markdown code fences.

Coverage: Single-node and Docker setup, Filer and master HTTP REST API, S3 SDK compatibility, IAM and STS, policy, retention, lifecycle, CORS, encryption, reverse proxy behavior, backup commands, and upstream configuration examples.

Omissions: The separately hosted wiki, its full S3 operation matrix, some illustrations, and AWS SDK API references remain online.

License: Apache-2.0. See `SEAWEEDFS-LICENSE`.

## Keycloak 26.8.2

Keycloak contributors. The exact 26.8.2 release archives at https://github.com/keycloak/keycloak/releases/tag/26.8.2 supply the rendered books and the Admin REST API. Individual publisher HTML guides at https://www.keycloak.org/guides were captured on 2026-11-07 and verified against the displayed 26.8.2 version and tagged source at https://github.com/keycloak/keycloak/tree/26.8.2/docs/guides. Devline extracted article markup and tables of contents, preserved original headings and examples, and embedded selected small upstream illustrations. Devline corrected outdated internal heading links and retained generated API primitive names as text where the upstream generator linked to nonexistent definitions.

Coverage: Docker and local setup, server and provider configuration, realm and client administration, OIDC/SAML and authorization, developer guides, importing and exporting realms, deployment, high availability, observability, troubleshooting, upgrades, and the Admin REST API.

Omissions: Java API Javadoc, third-party adapter manuals, unlinked site pages, larger illustrations, and hosted interactive tools remain online.

License: Apache-2.0. See `KEYCLOAK-LICENSE`.

License: PostgreSQL; GPL-2.0-only (Oracle man pages) and MIT (Docker Official Image manual); Apache-2.0 OR MPL-2.0; AGPL-3.0-only option (Redis 8.10.1 source reference) and CC-BY-SA-4.0 (archived Redis guides); SSPL-1.0 (server CLI help), Apache-2.0 (shell and Database Tools help), and MIT (Docker Official Image manual); MIT and Apache-2.0; MIT; Apache-2.0. See the accompanying notices. Devline retained article text, code, headings, links and other structural markup. Local illustrations are embedded where available. Interactive controls and site chrome are omitted. Markdown was rendered with Pandoc 3.13. Each article links to its upstream source.
