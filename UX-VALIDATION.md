# Devline 0.4.0 validation

Checks ran on an Apple M2 Pro Mac with macOS 27.2. Native observations below are from the installed 0.4.0 candidate. The final artifact and update-installation checks are tracked separately.

## References, projects, and downloads

- The current bundled catalog contains 21 docsets, 23 revisions, and 10,615 articles. The whole-corpus audit checked 69,786 code examples and 322,901 local links. It found zero render errors and 85 previously recorded missing local fragments. Pack metadata identifies known omissions.
- A fresh CLI library installed the 19 infrastructure articles and ranked the image and volume portability article first for `infrastructure:docker save`. A separate fresh library installed the Node.js, TypeScript, Playwright, React, and Vite packs and ranked `fs.promises.readFile`, `Page.goto`, `useState`, Narrowing, and `build.target` first in their docsets. The five packs contain 130 official-source articles and 4,347 code examples.
- All 26 independently prepared docset archives passed size and checksum verification. Their exact revisions are present in the local library, alongside older installed snapshots. The independent channel is staged for publication.
- Project reference tests include resolved framework versions, an incompatible release excluded from automatic search, an intentional pin retained across scopes, and global rollback precedence.
- A local HTTPS test interrupted a docset transfer and resumed at byte 16,384. The completed archive passed size and checksum checks. Rollback kept older revisions addressable.
- VS Code packaged and installed the extension in an isolated profile. Its command and URL encoding passed automated tests. Neovim’s headless test exercised cursor lookup, explicit queries, and visual selection. Full native editor handoff is still being checked.

## Starters, jobs, and transfer

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

The release CLI was measured against the freshly imported 10,453-record library with warm filesystem caches and external networking denied. Timings include process startup and JSON output.

| Query scope | Samples | Median | 95th percentile |
| --- | ---: | ---: | ---: |
| All installed docsets, six representative symbols | 48 | 5.54 ms | 8.09 ms |
| A Go/Chi project, two representative symbols | 24 | 5.50 ms | 5.99 ms |

## Native candidate and release checks

- The native search distinguished the two Python `Path.read_text` symbols and landed at the selected signature. The pinned Contents pane supported filtering and keyboard selection. After testing Focus Reading, a relaunch restored the query, article, scroll position, reading mode, and Contents state.
- The native retest still covers layout width changes, project versions and pins, Activity, and Ready Offline. System-wide hotkey delivery and a VoiceOver pass are also open.
- The candidate app passed strict code signature verification with Sparkle 2.10.1 embedded. Final installer verification and an installed Sparkle upgrade are still open. Distribution targets Apple Silicon on macOS 14 or later and uses ad hoc signing. Sparkle requires Ed25519 signatures on the appcast and archive.

The taskboard integration snapshot passed 52 Rust tests and Clippy with warnings denied. The ingestion and distribution suite passed 15 Python tests. The [0.3.0 evidence](validation.json) records the earlier release’s checks.
