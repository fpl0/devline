# Devline

Devline is a native macOS app and terminal tool for searching installed developer documentation and creating projects from versioned starters. The desktop app and CLI use the same local library.

![Devline showing Python’s Path.read_text method and its code example](press/reference-python-dark.png)

## Download

Download the Apple Silicon installer from [GitHub Releases](https://github.com/fpl0/devline/releases/latest). Open the DMG, drag **Devline** into **Applications**, then open it. macOS 14 or later is required. The build uses ad hoc signing and the installer includes Apple’s first-launch guidance.

## Open the app

After installing from the DMG, open **Devline** from **Applications**. If you downloaded the ZIP, extract it and move `Devline.app` into **Applications** first.

1. Open **Library** and select the doc packs and starter releases you want. **Install Selected** copies from the catalog bundled in the app. Each installed doc pack becomes searchable in **Docs**.
2. Use **⌘F** to find a symbol or topic. Try `py:Path.read_text()` or open an installed docset to browse by category. **⌘O** opens a project folder and detects its stack.
3. Open **Runtimes** to check detected toolchains, their executable paths and versions, and what setup is needed. **Prepare Starter** installs the verified starter and related docs from the bundled catalog.
4. In **Templates**, select a starter family and choose a release from its version picker. Install it on its own or with the related docs. **New Project** opens a sheet that checks the installed bundle and required toolchain. Choose a project name and an existing parent folder. The completion screen shows where the project was created and how to run its first test.

The bundled doc packs contain selected sections from upstream language and framework references. Each pack records its sources and license. Installing a starter gives you its packaged dependencies. Its compiler, interpreter, or SDK must already be available on your Mac. **Preview** releases are in the catalog for inspection and require offline verification before they can create projects.

The offline reader preserves article headings, code examples, tables, links, and available illustrations. Links to installed articles open in Devline. **⇧⌘O** opens **On This Page**, **⌥⌘F** finds text in the article, and **⌘[** and **⌘]** move through reading history. Use **⌘D** to save a reference and **⇧⌘B** to open Saved and Recent. **Open Official Source** opens the publisher’s page in your browser.

## Use the CLI

In the app, open **Devline → Settings → Command Line** and choose **Install CLI** to link the bundled command at `~/.local/bin/devline`. If your shell needs that directory on its `PATH`, use **Copy PATH Command** and run the copied command in Terminal.

If you built and installed from source, you can also add the local launcher to your shell from the repository root.

```sh
export PATH="$(cd ../bin && pwd):$PATH"
devline library
devline runtimes
devline docs install go
devline docs search 'go:http.Handler' --pick
devline docs show 'go:http.Handler'
devline docs save 'go:http.Handler'
devline docs saved --pick
devline docs recent
```

The CLI works with the app closed. An unscoped lookup inside a project uses its detected stack when available. Prefix a query with a docset such as `go:` or `py:`, or use `--docset PACK[@VERSION]` to select an exact docset or release. Use `--stack all` to clear the project scope.

With the default pager, the interactive reader opens at the matching declaration when available. Set `DEVLINE_PAGER` or `PAGER` to choose a pager. Pipes and `--no-pager` write the stored article text without pager formatting. Use `--ids` with `docs search` for one exact article ID per line or `--json` for structured output.

Saved references and articles opened interactively are shared with the desktop app. Reopen a reference with `devline docs saved --pick` or `devline docs recent --pick`. To open a reference in the app, launch `Devline.app` once to register links, then run `devline docs show 'go:http.Handler' --open`. Use `--link` to print the versioned local link without opening it.

```sh
devline templates list --stack go
devline prepare go/chi-api@1.1.0
devline doctor go/chi-api@1.1.0
devline new go/chi-api@1.1.0 ./status-api --dry-run
devline new go/chi-api@1.1.0 ./status-api
cd status-api
./devkit-test.sh
```

`prepare` installs the selected starter and its related docs, then reports readiness. If you omit the version, `new` prefers the latest installed, verified release for your Mac’s target and then checks its toolchain. Use `--json` for structured results and `devline completions zsh` for shell completions. `doctor` and `prepare` exit with status 3 when preparation is needed. An empty search exits with status 4.

Search, reads, and creation use local files. A recipe with upstream refresh support can build a new release while connected. For example, `devline templates build go/chi-api --refresh-deps` resolves dependencies and installs the new release after its offline checks pass. Earlier releases stay available.

## Updates

Open **Updates** or choose **Devline → Check for Updates…**. Reading status stays local. **Check for Updates** contacts the official release sources. Cached versions remain available when a later check fails, with the failure shown explicitly. Filter to **Available** to see pending updates.

- Newer bundled documentation and verified starter releases install alongside existing versions. Search defaults to the newest installed revision of each docset. Pin an older release to search it, or open an existing saved reference at its original version.
- The upstream documentation action opens the publisher’s latest documentation. Updated offline snapshots are distributed through catalog releases.
- Template rebuilds resolve current dependencies within the recipe’s supported version range and run offline verification before adding an immutable release.
- Runtime updates open official installation pages so you can use your existing version manager or installer. Devline does not silently replace toolchains.
- App checks use the [public GitHub release manifest](https://github.com/fpl0/devline/releases/latest/download/release.json). An available app update opens the release page. The native settings and CLI can replace or clear this channel.

```sh
devline updates                 # cached status, no network
devline updates check           # explicit online check
devline updates check --json    # structured status for scripts
devline --offline updates      # cached status while disconnected
```

For a custom app release channel, serve `{"schema_version":1,"version":"0.3.0","url":"https://example.com/releases/0.3.0"}` over HTTPS, then run `devline updates configure https://example.com/release.json`. URLs must not include credentials, queries or fragments. `devline updates configure --clear` disables app checks.

The `devkit` command remains a compatibility alias in the app and packaged launchers. Existing `DEVKIT_HOME`, `DEVKIT_CATALOG`, `devkit://` links, and generated project filenames remain supported.

## Build from source

On an Apple Silicon Mac with Xcode and its command-line tools, Rust/Cargo, and Python 3, run these commands from the repository root.

```sh
./scripts/package.sh --no-install
./scripts/create-dmg.sh
./scripts/install.sh
cargo test --locked --manifest-path core/Cargo.toml
```

The package script uses Xcode’s native icon compiler and the checked-in catalog snapshots and may need a connection to fetch Cargo build dependencies. Without `--no-install`, it installs the local build at `~/Applications/Devline.app`, writes `../Devline-macos-arm64.zip`, and creates `../Devline.app` and `../bin/devline` launchers, with `devkit` as a compatibility alias. Bundles are signed in a temporary local directory because iCloud File Provider adds metadata to app folders in Documents.

- `macos/` contains the SwiftUI app and C bridge. Native requests cross the bridge off the UI thread.
- `core/` contains the Rust CLI, SQLite search and local library, template installation, and project creation.
- `data/` contains the versioned docs and starter bundles, upstream source notices, and catalog manifests.
- `scripts/` contains the packaging and documentation preparation tools.

[GitHub Actions](https://github.com/fpl0/devline/actions) runs Rust validation, Python ingestion tests, and Swift navigation tests, then builds the native app on Apple Silicon. [Core protocol](CONTRACT.md) · [Third-party notices](THIRD_PARTY_NOTICES.md)

The existing library stays at `~/Library/Application Support/DevKit` for compatibility with earlier builds. Set `DEVKIT_HOME` for an isolated library and `DEVKIT_CATALOG` to use a different catalog directory. The CLI also accepts `--home` and `--catalog`.

## Rebuilding documentation snapshots

The package build uses the checked-in snapshots. Rebuilding from upstream requires Python 3 and the pinned source inputs recorded in each pack’s `SOURCES.md`. Use Pandoc 3.13 when rebuilding a pack from Markdown. The installed reader uses the generated HTML directly.

```sh
python3 scripts/fetch-docs.py --help
python3 -m unittest discover -s scripts -p test_fetch_docs.py
```

The ingestion tests use only Python’s standard library. The build script accepts local archives and toolchains, and `--fetch` downloads missing pinned archives. An existing content revision is reused only when its output matches byte for byte.

## Included and verified

The catalog contains 10,358 searchable articles across 15 documentation packs and 10 starter releases for Python/FastAPI, C#/ASP.NET Core, Java/Spring Boot, Ruby, Go/Chi, Rust/Axum, and Elixir/Phoenix. The newest Go and Python starters include an in-memory task API with validation, structured errors, and lifecycle tests. The ASP.NET starter includes SQLite.

All ten starter releases passed smoke tests with external network access denied, using fresh language caches or environments and the toolchains installed on this Apple Silicon Mac. Devline detects the compiler or interpreter on the Mac. Generated projects include `devkit-test.sh`, which selects the detected runtime and companions. `DEVKIT_TOOLCHAIN` can override it. Generated projects for all seven stacks also passed with a system-only PATH. The final Rust check rebuilt with empty Cargo caches and external networking blocked.

The 32 Rust tests cover indexing, version-scoped search and offline links, atomic pack installs, saved references, starter integrity, and project creation. Clippy passed with warnings denied. The complete refreshed library was tested on both a fresh install and an upgraded library that retained its existing doc versions.

A 48-query benchmark over the full refreshed library measured 9.69 ms median and 12.76 ms at the 95th percentile. The same run measured 13.60 ms median and 34.64 ms at the 95th percentile for 0.2.0 on its original library. Measurements include CLI process startup and use warm filesystem caches on this Apple Silicon Mac.

Go's online dependency refresh, network-denied verification, and new-release publication were exercised end to end. Refresh recipes for the other stacks are included but have not yet been exercised end to end against their upstream registries. Normal lookup, installation from the bundled catalog, and project creation stay local.
