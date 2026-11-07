# Devline 0.4.0 validation

Checks ran on an Apple M2 Pro Mac with macOS 27.2. Native observations below are from the installed 0.4.0 candidate. The final artifact and update-installation checks are tracked separately.

## References, projects, and downloads

- The bundled catalog contains 21 docsets, 23 revisions, and 10,615 articles. The whole-corpus audit checked 69,786 code examples and 322,901 local links. It found zero render errors and 85 previously recorded missing local fragments in the source snapshots. The reader applies 53 verified fragment aliases across 37 source URLs. It follows an alias only when the replacement anchor exists in the same installed docset revision. [Fragment navigation repair](https://github.com/fpl0/devline/commit/9c7b29aabe243418c7db71a5ea9ba3c80af39dc9)
- A fresh CLI library installed the 19 infrastructure articles and ranked the image and volume portability article first for `infrastructure:docker save`. A separate fresh library installed the Node.js, TypeScript, Playwright, React, and Vite packs and ranked `fs.promises.readFile`, `Page.goto`, `useState`, Narrowing, and `build.target` first in their docsets. The five packs contain 130 official-source articles and 4,347 code examples.
- All 26 independently published docset archives, totaling 169,713,454 bytes, were downloaded over public HTTPS and matched their recorded sizes and SHA-256 checksums. The manifest matched the staged file byte for byte. [Versioned archives](https://github.com/fpl0/devline/releases/tag/docsets-2026.11.07) · [Public manifest](https://github.com/fpl0/devline/releases/download/docsets-v1/docs-manifest.json)
- A fresh Devline 0.4 CLI library with an empty bundled catalog discovered 21 docsets and 26 releases through the public channel. It installed Go revision 2, discovered and installed revision 3, then rolled back offline. Both revisions remained searchable. Downloaded Node.js and infrastructure references also passed offline lookups.
- Project reference tests include resolved framework versions, an incompatible release excluded from automatic search, an intentional pin retained across scopes, and global rollback precedence.
- A local HTTPS test interrupted a docset transfer and resumed at byte 16,384. The completed archive passed size and checksum checks. Rollback kept older revisions addressable.
- VS Code packaged and installed the extension in an isolated profile. Its command and URL encoding passed automated tests. Neovim’s headless test exercised cursor lookup, explicit queries, and visual selection. Full native editor handoff is still being checked.

## Starters, jobs, and transfer

- All 28 bundled releases across 22 starter families passed offline verification. A single readiness check of the installed library matched all 28 exact releases to their receipts. The matrix covers six Go, three Rust, four Python, three Ruby, three .NET, three Java, three Elixir, and three Node.js releases. [Versioned starter catalog](data/templates-catalog.json)
- The seven Python and Ruby releases installed and verified in the local library with external networking denied and local loopback available. The four new Python and Ruby kits also passed dependency refresh and offline verification in isolated libraries. Their service tests exercise client construction and SQLite flows. The data and task automation starters exercise file inputs and local HTTP fixtures.
- The four new Python and Ruby starters passed generated project creation and readiness under the stricter write-and-network sandbox. The Ruby service kit uses the selected Xcode or Command Line Tools compiler and SDK directly during native gem builds.
- Existing Go, Rust, Python, and Ruby project fixtures prepared their dependencies, preserved their source manifests and lockfiles, and ran commands against their managed offline dependency caches.
- Playwright 1.65.0 and its bundled Apple Silicon WebKit 27.2 build passed strict offline project creation with a real form interaction, screenshot, and API request to a local server. The same starter refreshed and verified as a new local release in a separate library. The browser build requires macOS 26.5 or later.
- A cancelled build stopped a descendant that ignored termination, removed staging files, retained the prior starter, and preserved its log. A timed-out build passed the same cleanup checks.
- A Go/Chi bundle exported, inspected, imported, and created a project with external networking denied. It was 1,350,600 bytes compressed and 3,896,287 bytes unpacked. The destination library on this Mac passed the generated smoke test and reported the matching docs, starter, and runtime ready. A separate CLI smoke test covered a docs-only bundle and an offline install job.

## Infrastructure and a complete project

- Registry metadata verified the exact index, Linux arm64 manifest, image configuration digest, and layer sizes for all ten listed services. Their combined compressed layers total 1,441,258,366 bytes. The checked-in catalog test also checks the connection templates and exact runbook IDs without contacting a registry.
- The `go/chi-taskboard@1.0.0` starter created a project offline and passed its starter test with an empty build cache and external networking denied. Prepared PostgreSQL 18.6 and Redis 8.10.1 started in a managed Compose stack and reported healthy.
- The taskboard built and passed its application integration test with external networking denied and local loopback available. It created and read a task over HTTP, showed the first read from PostgreSQL and the next from Redis, and still displayed the task after the app restarted. The disposable stack was then stopped.
- All ten archived Linux arm64 images restored from Devline's vault with their Docker cache entries absent and reached a healthy local stack. The test stack stopped with its data retained. A copy from a stopped PostgreSQL data volume contained `PG_VERSION`.
- Source review matched the native **Project**, **Infrastructure**, and **Ready Offline** models to the core's catalog, status, archive, and stack responses. Version rows use exact references. The UI distinguishes a present archive from a verified one, and its source inspector shows publisher, license, image tag, and platform digest.

## Search measurements

The release CLI ran seven times per query against the installed library with external networking denied. The 49 timings include sandbox and CLI process startup. Each query returned the same number of results in all seven runs.

- `py:Path.read_text()` had a 19.12 ms median and a 208.91 ms maximum, with two results.
- `go:http.Handler` had a 14.95 ms median and a 20.71 ms maximum, with one result.
- `node:fs.promises.readFile` had a 15.55 ms median and a 17.89 ms maximum, with one result.
- `playwright:Page.goto` had a 15.37 ms median and a 17.45 ms maximum, with one result.
- `react:useState` had a 20.15 ms median and a 35.31 ms maximum, with five results.
- `rust:std::fs::read` had a 15.73 ms median and a 20.22 ms maximum, with five results.
- `sql postgres` had a 27.88 ms median and a 106.31 ms maximum, with five results.

## Native candidate and release checks

- The native search distinguished the two Python `Path.read_text` symbols and landed at the selected signature. The pinned Contents pane supported filtering and keyboard selection. After testing Focus Reading, a relaunch restored the query, article, scroll position, reading mode, and Contents state.
- Final installed-app checks for layout width changes, project versions and pins, Activity, and Ready Offline are pending.
- The candidate app passed strict code signature verification with Sparkle 2.10.1 embedded. Final installer verification and an installed Sparkle upgrade are still open. Distribution targets Apple Silicon on macOS 14 or later and uses ad hoc signing. Sparkle requires Ed25519 signatures on the appcast and archive.

The post-rebase core passed 65 Rust tests, formatting checks, and Clippy with warnings denied. The ingestion and distribution suite passed 15 Python tests. The [0.3.0 evidence](validation.json) records the earlier release’s checks.
