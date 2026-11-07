# Devline 0.4.0 validation

Checks ran on an Apple M2 Pro Mac with macOS 27.2. Native observations below are from the installed 0.4.0 candidate.

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
- The final installed app created and verified a Chi Taskboard, started its exact PostgreSQL and Redis releases, reported both healthy on custom loopback ports, and stopped the stack while retaining data. Activity displayed job output and completion. Runtime details showed the resolved executable and detection-only policy.
- Article scrolling worked, and C# list scrolling loaded additional pages from 80 to 240 articles. Light and dark appearances were inspected. The selected offline library reported Ready Offline. An explicit Chi version pin made in the native app appeared in the CLI; automatic matching was then restored.
- The final app and the app mounted from its DMG passed strict code signature verification with Sparkle 2.10.1 embedded. Sparkle requires Ed25519 signatures on the appcast and archive. The user waived the Keychain-dependent end-to-end Sparkle installation test, notarization, the physical global-shortcut check, and VoiceOver. A signed appcast is unavailable for 0.4.0. The GitHub DMG targets Apple Silicon on macOS 14 or later and uses ad hoc signing.

The post-rebase core passed 65 Rust tests, formatting checks, and Clippy with warnings denied. The ingestion and distribution suite passed 15 Python tests. The [0.3.0 evidence](validation.json) records the earlier release’s checks.

The exact application code at `17cf7015d8d1bf2fdabff83cd84a95f2ecdce017` passed GitHub validation, including the native macOS build. The final archive installed successfully, and the installed CLI reported 0.4.0. [GitHub validation](https://github.com/fpl0/devline/actions/runs/42373786445)


## Desktop starters — 0.5.0 (7 November 2026)

- The Tauri 2.12.5 text inspector includes 392 locked, unmodified vendored Cargo packages and embedded HTML/CSS/JavaScript. Four tests cover Unicode/limits, embedded assets, the actual IPC response, and rejection of an unauthorized window. A fresh arm64 release app built with external networking denied, passed strict signature and plist checks, and declares a macOS 14 minimum. Native interaction verified the sample counters (9 words, 2 lines, 45 Unicode scalars, 56 UTF-8 bytes) and Clear.
- The SwiftUI Idea Board has no third-party packages. Four Swift Testing tests cover model editing, filters, and JSON persistence. Fresh-cache verification and the release app build passed with external networking denied. Native interaction covered add/edit, completion, filters, search in notes, and persistence after relaunch. This starter requires local Xcode 16 or later; Command Line Tools alone lacks its test framework.
- The dependency-free SwiftPM adapter reports unsupported package dependencies before claiming readiness. Swift detection preserves the compiler driver alias, handles the separate Swift driver version, and discovers the selected SDK. Tauri dependency detection reads Cargo manifests and resolved locks. Generated build artifacts leave readiness intact while source changes remain tracked.
- Official desktop references include 102 Tauri articles and 43 chapters of The Swift Programming Language. Infrastructure revision 2 contains only six publisher articles, while the 13 Devline guides use a separate help flow. Historical revision 1 bytes remain unchanged and are excluded from current bundled references and channel refreshes. The public corpus has 10,747 articles, 72,418 examples, 5,917 tables, and 949 embedded images; its audit found no new rendering or anchor regressions.
- SwiftUI now imports official declarations and original documentation comments from the selected local Xcode SDK. A fresh import produced 8,796 declarations, including 6,313 with comments, with outbound networking denied and writes confined to staging. Extraction includes SwiftUI and SwiftUICore and excludes synthesized members and inherited duplicates. An independent DocC comparison preserved all 2,147 code examples and resolved 5,408 local links to 1,195 symbols. The final CLI verification covered unqualified and qualified View, State, Binding, Text, App, NavigationStack, and ConcentricRectangle lookups, cached reuse, library readiness, and continued cached use without an available SDK. The imported reference stays local; app packaging, public archives, and portable bundles reject it. Apple’s website articles, tutorials, and illustrations remain outside this snapshot.
- Core and CLI: 77 tests passed, with Clippy warnings denied. All 21 Python ingestion/distribution tests and the native reference-navigation checks passed. Package guards rejected nested and parent symlinks, forged SwiftUI metadata, and local-only packs in isolated fixtures. Runtime tests ran on Apple Silicon macOS 27.2 with Xcode’s Swift 6.4.2 and Rust 1.99.1; macOS 14 was checked as the declared binary deployment target, not on a separate physical host.

- Installed-app verification: the native Prepare from Xcode action imported 8,796 SwiftUI symbols; State rendered original code examples and navigated to View locally. Documentation Updates displayed the selected SDK and indexed count. Devline Help opened a Keycloak guide and followed its local portability link within the help sheet. With outbound networking denied, the installed CLI reported both the generated SwiftUI and Tauri projects Ready Offline, including their documentation.
