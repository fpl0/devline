# Using Devline

The macOS app and CLI share the same local library. See the [README](README.md) for an overview and a first project.

## Install

Download the Apple Silicon DMG from [GitHub Releases](https://github.com/fpl0/devline/releases/latest), then drag **Devline** into **Applications**. macOS 14 or later is required. The DMG includes first-launch instructions for the ad hoc-signed build.

Devline’s **d** icon stays in the macOS menu bar while the app runs, even when you hide the app or close its window. Click it and choose **Open Devline** to bring the workspace back, or **Quit Devline** to exit.

## Get ready before you go offline

**Home** shows one checklist for the whole Mac. Work through it while you’re connected.

- **Documentation and starters** come inside the app. **Install** adds all of them to your library in under a minute and needs no connection.
- **Toolchains** are the compilers and interpreters already on your Mac. Devline finds them and never changes them. For one that is missing, **How to Install** opens its page in **Library → Toolchains** with the command for mise or Homebrew, whichever your Mac has, and a link to the official installer. Choose **Rescan** afterward.
- **Docker** is needed only for projects that use a database, queue, or another service. **Setup Guide** lists the engines Devline works with.
- **Service images** are saved from their registries. **Download All** saves the images the starters use, one after another.
- **Packages** are the 100 most used packages for each runtime you have. **Download All** fetches them, with their dependencies, for every toolchain on your Mac.

When every line is checked, the heading reads **Ready to work offline**. While your Mac has no connection, the sidebar says so and the download buttons wait.

```sh
devline runtimes                      # what was found, and how to install what wasn’t
devline packages                      # each runtime’s package shelf
devline packages download go          # download or refresh one shelf
```

## Add packages to a project offline

On a project’s page, choose **Add Packages…** beside **Dependencies**. Search the shelf, check what you need, and choose **Add Package**. Devline edits the manifest and lock file in a private copy of the project, tests that copy with the network blocked, and only then changes your files. If the shelf lacks a version your project’s other dependencies require, the project is left as it was and **Try With the Network** fetches the rest.

**Library → Packages** lists every shelf with its versions and size. **Check for Newer Versions** refreshes one, and the field at the bottom keeps an extra package of your choice on the shelf.

```sh
devline packages list python
devline packages download python --also polars
devline --offline packages add --path ./service httpx rich
devline packages remove python
```

A shelf belongs to the toolchain that downloaded it: Python wheels match that interpreter, for example. Download again after a major toolchain upgrade. Swift packages are listed in the catalog with their repository and products, so extra Swift packages are added there.

## Read and look up references

In **Docs → Manage Docs → Available**, choose a docset and select **Install**. Use **⌘F** to search installed publisher and SDK references, or try `py:Path.read_text()` to search Python directly. Choose **Imported** to search your DevDocs snapshots. Links to installed pages open in Devline, and links marked **Online** open in your browser.

Use **⇧⌘O** to search Contents and pin it beside the article, **⇧⌘R** for Focus Reading, and **⌥⌘F** to find text within an article. **⌘[** and **⌘]** navigate reading history. Use **⌘D** to save a reference and **⇧⌘B** to open Saved and Recent. The app remembers the current reference, scroll position, and reading layout between launches.

In **Settings → Lookup from Any App**, choose a global shortcut. The default **⌃⌥D** opens lookup and **⌃⌥⌘D** looks up selected text when Accessibility access is enabled. You can also select text and use **Services → Look Up in Devline**. [Visual Studio Code and Neovim integrations](integrations/README.md) open the selected symbol with the editor’s project context.

## Match documentation to a project

Open a project folder with **⌘O**. Its page brings together its references, readiness, local services, and app controls. Open projects stay listed under **Projects** in the sidebar. Use **Project References** in **Documentation** to see detected dependencies, chosen docset versions, and any mismatch. Devline reads supported manifests and lockfiles for Python, C# and .NET, Java, Ruby, Go, Rust, Elixir, Node.js, and Swift. Resolved framework versions take priority over manifest constraints. When no installed docset matches exactly, Devline uses the closest release on the same release line and says so, rather than leaving the project without documentation. Pin choices stay in the local library and are shared with the CLI.

Return through the sidebar, **Home → Your projects**, or **File → Open Recent**. **Browse Project Docs** clears unrelated searches and opens the project's reference scope. A generated project's origin links back to its exact starter release; the starter's reference links use its included versions.

```sh
devline project --path ./service
devline docs search 'Path.read_text' --project ./service
devline project --path ./service pin fastapi@0.162.0+devline.1
devline project --path ./service unpin fastapi
```

An unscoped lookup inside a project uses compatible installed references. Use a docset prefix such as `go:`, an exact `--docset PACK[@VERSION]`, or `--stack all` to override that selection. Computed versions and unsupported constraints can require a manual pin.

### Prepare an existing project

Choose **Prepare for Offline** on the project’s page while connected. Devline prepares the project's matching or pinned documentation, dependency closure, and configured image archives together. Opening a project starts preparation when needed; progress stays in the project screen, with logs in **Activity**. The final result covers every component and identifies anything Devline cannot prepare, such as a missing compiler. Preparing archives does not start services.

Dependency preparation uses a staged copy and tests it with external networking denied. Existing local dependencies are tried first. When needed, Devline prepares a cache under `.devline/cache/` for new starters or `.devline-offline/` for projects using the older layout. It records verification after the checks pass. Your original source files, manifests, and lockfiles stay intact. Changes to the source, lockfiles, toolchain, or cache can require another check. An explicit `project run` command can still use the prepared dependencies during development.

```sh
devline project --path ./service prepare --background
devline --offline toolbox check --path ./service
devline --offline project --path ./service run -- python -m pytest -q
```

`devline toolbox prepare --path ./service` runs the same complete preparation. Add `--offline` to require local resources; missing downloads are reported rather than fetched. `project readiness` checks dependency evidence only, while `toolbox status` and `toolbox check` include references, toolchains, and services. Removing the original starter from the library does not invalidate an already created project.

Preparation supports conventional single-runtime projects using Go modules, Cargo, Python registry requirements or static project dependencies, Bundler, Maven, NuGet, Mix/Hex, npm with `package-lock.json`, or Swift packages with `Package.swift`. Swift packages can use external source dependencies: keep their versions locked in `Package.resolved`, and Devline prepares the source repositories in the project's managed cache for offline builds. The selected adapter checks its prerequisites. Install the matching toolchain and any reported native build tools first. Open independently locked projects at their individual roots.

### Edit and run a project

On the project’s page, choose **Open in** your editor or **Project Guide**, make your changes, then choose **Run**. Devline starts configured services from their saved image archives and runs the app with prepared dependencies. It supplies the project's local service connection settings automatically. Internet access is blocked for the app; connections to local services remain available.

The panel shows **Starting** while the app builds or waits for its configured readiness check, and the app’s output appears underneath as it is written. **Open App** becomes available when the local web app is ready. **Stop** stops the app process; services and their data stay available until you stop them separately in **Local Services**. Running an app does not change its offline readiness: what it writes while it runs (a database, logs, uploads) is left out when the files are listed in the project's `.gitignore`.

In **Services → This Project**, each service shows its local ports; click one to copy its address. Services with a page of their own, such as Mailpit's inbox, RabbitMQ's management console, and Keycloak's admin console, also have a button that opens it in your browser while the services are running.

New projects include `devline.sh` at the root. Run `./devline.sh --help` to see the commands. The script uses the project folder even when called from another directory.

```sh
./devline.sh test
./devline.sh run
./devline.sh start
./devline.sh status
./devline.sh logs --tail 100
./devline.sh stop
./devline.sh prepare --offline
./devline.sh check
```

`test` runs the starter's local tests with the bundled dependencies. Starters with dedicated helpers also provide commands such as `build`, `setup`, or `smoke`. These commands use the local toolchain. The other commands use the installed Devline CLI. When a starter has a configured app, `run` launches it in the foreground and `start` manages it in the background. `prepare` can fetch missing dependencies, docs, and service images unless you pass `--offline`. Use `./devline.sh services up`, `down`, `status`, or `logs` to manage configured services. Use `./devline.sh exec COMMAND ...` to run a command with prepared dependencies and external networking denied.

The app and CLI share the same managed app process. Use these commands from outside the project or with a project that predates `devline.sh`.

```sh
devline --offline project --path ./service start
devline project --path ./service app-status
devline project --path ./service logs --tail 100
devline project --path ./service stop
devline project --path ./service commands
```

Use `devline --offline project --path ./service run` to run the configured app in your terminal. An explicit command after `--` uses the prepared dependencies and connection settings and leaves service startup to you. For example, `devline --offline project --path ./service run -- ./devline.sh test` runs a new starter's tests with external networking denied.

In a generated project, `.devline/config.json` declares commands and the optional local web address. A web app configuration can look like this.

```json
{
  "schema_version": 1,
  "run": {
    "command": ["/bin/sh", "./.devline/scripts/run.sh"],
    "url": "http://127.0.0.1:5080",
    "health_path": "/health/ready"
  },
  "test": { "command": ["./devline.sh", "test"] }
}
```

Commands are argument arrays and run from the project folder. Keep the app in the foreground so Devline can track and stop it. The optional `url` must use HTTP with a loopback address and an explicit port. Devline passes the loopback origin as `DEVLINE_APP_URL` and as `ASPNETCORE_URLS` for ASP.NET Core. Configure the app to listen on that address. The optional `health_path` must return HTTP 200 before **Open App** is enabled; without it, Devline checks that the port accepts connections. Omit both fields for command-line or native desktop apps.

Devline also reads `devline.json` in older projects. For projects without a launch configuration, it looks for `run.sh`, then a .NET project in the selected folder, and recognizes `test-offline.sh` for testing. Devline uses the compatible toolchain installed on your Mac.

## Build a desktop app

The **Fieldnotes** apps provide the same useful notes-workspace domain through two different desktop stacks: Rust with Tauri and native SwiftUI. They include editing, search, local persistence, backup/import, and tests. Tauri uses its bundled web assets and Cargo dependencies; SwiftUI uses the compiler, macOS SDK, and Swift Testing from Xcode 16 or later. No frontend development server is needed for the Tauri app.

**Focus Sessions** is a second Swift app: a native dashboard and menu-bar timer with pause/resume, breaks, session history, and CSV export. Its timer recovers across sleep and restarts.

```sh
devline --offline new rust/tauri-desktop ./rust-notes
devline --offline new swift/swiftui-app ./swift-notes
devline --offline new swift/focus-timer ./focus-sessions
./swift-notes/devline.sh run
./swift-notes/devline.sh build
./focus-sessions/devline.sh test
```

The desktop projects provide `./devline.sh run`, `./devline.sh build`, and `./devline.sh test`. Build output stays under `target/` for Tauri and `.build/` for Swift. Generated `.app` bundles are signed locally for development. Select Xcode in **Settings → Locations → Command Line Tools**, or set `DEVELOPER_DIR`. Devline discovers these tools with `xcrun`; it does not install them.

The desktop documentation includes an official Tauri snapshot and **The Swift Programming Language**. SwiftUI's API reference is prepared from the selected local SDK.

### Prepare the SwiftUI SDK reference

In **Docs → Manage Docs → Installed**, find **SwiftUI · Apple API Reference** and choose **Prepare from Xcode**. **Prepare for Offline** on a project’s page includes this import when the project needs it. The import reads the selected macOS SDK locally and works offline.

```sh
devline docs local
devline --offline docs install swiftui --background
devline --offline docs search 'swiftui:View'
```

Follow the import in **Activity** or with `devline jobs status JOB_ID --follow`. Search after the job succeeds. Use the same card or import command after selecting a newer Xcode SDK. The card shows **Refresh from Xcode** when the selected snapshot is already installed. Devline keeps earlier installed snapshots available for project pins.

The reference comes from public declarations and original documentation comments in Apple’s SwiftUI module and, where available, SwiftUICore. It includes the examples found in those comments. Some APIs have a declaration only. Apple Developer website tutorials, articles, and illustrations are outside the SDK snapshot. Devline stores the extracted content locally and excludes it from app releases, downloadable docsets, and portable exports. Prepare it separately on each Mac. [Apple’s SwiftUI reference](https://developer.apple.com/documentation/swiftui/)

## Use the terminal

In **Devline → Settings → Command Line**, choose **Install CLI** to link the bundled command at `~/.local/bin/devline`. Use **Copy PATH Command** if your current shell needs that directory on its PATH. For a source install, the packaging script also creates a launcher in `../bin`.

```sh
devline docs install go@1.27.3+devline.3
devline docs search 'go:http.Handler' --pick
devline docs show 'go:http.Handler'
devline docs save 'go:http.Handler'
devline docs saved --pick
devline docs recent
```

The CLI works with the app closed. With the default pager, the reader opens at the matching declaration when available. Set `DEVLINE_PAGER` or `PAGER` to choose a pager. Pipes and `--no-pager` use the stored article text. Use `--ids` with search for exact IDs or `--json` for structured output. Use `docs show --open` to open an article in Devline or `--link` to print its versioned local link.

The CLI accepts `--offline` for operations that must stay local. An empty search exits with status 4, and readiness checks that require preparation exit with status 3. Run `devline completions --help` for shell setup. Earlier installations can still use the `devkit` command and open saved `devkit://` links.

### Import DevDocs into your local library

Open **Docs → Manage Docs → Import from DevDocs**, then choose a docset and a release. The importer shows its snapshot date, size estimate, and progress. In the CLI, use `--latest` to import one current release per docset. The CLI also uses that mode when **Settings → Local Library → Keep only the latest version after updates** is on. [DevDocs](https://devdocs.io/)

```sh
devline docs import-devdocs --online --doc astro
devline --offline docs search --docset devdocs-astro 'Actions'
devline docs import-devdocs --online --latest
```

With that setting off, run `devline docs import-devdocs --online` to import every available release. A full import downloads several gigabytes and builds a local search index.

You can also import a local DevDocs archive or a directory of downloaded docsets. Use the local `public/docs` directory from a DevDocs checkout after running `bundle exec thor docs:download --all`. [DevDocs download instructions](https://github.com/freeCodeCamp/devdocs#quick-start)

```sh
devline --offline docs import-devdocs ./astro.tar.gz
devline --offline docs import-devdocs /path/to/devdocs/public/docs
```

In the app, open **Imported** in Docs to browse or search these snapshots. The CLI also searches the installed imports. Imported snapshots are kept separate from Devline's project references and are excluded from portable library exports. Run the same import again after a failure to retry. Search works offline after installation.

## Prepare a starter for offline use

**Home → Start something new** lists every starter by runtime; choose one, name the project, and choose **Create and Open**. Creating installs the starter and its documentation if needed. **Library → Starters** shows a starter’s available releases, contents, dependency versions, and verification. **Library → Toolchains** detects the compilers, interpreters, and SDKs on this Mac, and shows how to install the ones it does not find. In **Library → Contents**, select docsets, starters, and images, then choose **Check Readiness**. Use **Selection → Select Installed** to pick everything installed or archived. With a project open, the check also covers its references, toolchains, dependencies, and configured service images.

**Install** adds a starter and its related docs without creating a project. **Verify Offline Now** reruns the installed starter’s test with external networking denied. Background work appears in **Activity** with progress, logs, and cancellation.

```sh
devline templates inspect go/chi-api@1.2.0
devline prepare go/chi-api@1.2.0 --background
devline jobs status JOB_ID --follow
devline jobs cancel JOB_ID
devline offline status --starter go/chi-api@1.2.0
devline new go/chi-api@1.2.0 ./status-api
```

Use the ID printed by `prepare` when following or cancelling a job. Creating a project prepares its selected documentation and required service archives, then verifies the generated files with external networking denied. `devline --offline new` requires those service archives to be present already. A recipe can refresh its supported upstream dependencies online and verify the new release offline before installing it.

The app shows creation progress and lets you cancel while it prepares downloads and verifies the project. Use `devline new STARTER@VERSION ./project --background` for the same durable job in the CLI, followed by `devline jobs status JOB_ID --follow` or `devline jobs cancel JOB_ID`. A cancelled attempt keeps completed library downloads for the next attempt.

```sh
devline templates build go/chi-api --refresh-deps --background
devline templates verify go/chi-api@1.2.0 --background
```

### Generated project layout

- `devline.sh` is the entry point for project commands.
- `.devline/config.json` holds the launch configuration, and `.devline/origin.json` records the exact starter release.
- `.devline/scripts/` holds the starter's lifecycle scripts. `.devline/packages/` holds bundled packages that the framework can load from a custom path. `.devline/cache/` holds dependencies prepared later as needed.
- `.devline/services/` holds local Compose settings and private credentials when you configure services.

Source files, manifests, lockfiles, and convention-based directories such as Go's `vendor/` and Node's `node_modules/` stay in their standard locations. Older projects retain their existing layout.

### Representative application catalog

The catalog contains three complete applications for each of the nine runtime families. Choose an app by the workflow you want to explore. Shared domains expose the differences between ecosystems, while each runtime also has applications that suit its particular strengths. Existing starter IDs stay stable where the replacement evolves the same project slot; the display names describe the new apps.

| Runtime | Applications and workflows |
| --- | --- |
| **Go** | **File Vault** (`go/file-index`): concurrent integrity snapshots and change verification. **Issue Desk** (`go/chi-api`): persistent API with workflow, history, and version conflicts. **Watchtower** (`go/chi-taskboard`): HTTP availability monitoring with a server-rendered dashboard. |
| **Rust** | **Logbook** (`rust/jsonl-report`): streaming request-log analytics. **Issue Desk** (`rust/axum-api`): typed HTTP API with durable state and workflow rules. **Fieldnotes** (`rust/tauri-desktop`): a persistent desktop notes workspace. |
| **Python** | **Order Ledger** (`python/fastapi-api`): transactional order and stock API with idempotency. **Sales Observatory** (`python/data-automation`): validated DuckDB ingestion, data-quality checks, and reports. **Field Notes Search** (`python/document-search`): local document indexing and SQLite full-text search. |
| **Ruby** | **Workshop Desk** (`ruby/rails-web`): a Rails service-job and estimate workflow. **Dispatch Inbox** (`ruby/service-kit`): signed, idempotent webhook receipt and retry processing. **Snapshot Vault** (`ruby/task-automation`): backup, verification, and explicit restoration through Thor and Rake. |
| **C# / .NET** | **Reconciliation** (`dotnet/json-cli`): match invoices and payments with exact arithmetic and actionable reports. **Issue Desk** (`dotnet/aspnet-api`): EF Core/SQLite API with audit history and optimistic concurrency. **Purchase Desk** (`dotnet/aspnet-mvc`): authenticated Razor workflow with roles, decisions, and audit history. |
| **Java** | **Reconciliation** (`java/json-tool`): invoice/payment matching and a packaged CLI. **Issue Desk** (`java/spring-api`): Spring MVC/JDBC API with transactional persistence and version conflicts. **Purchase Desk** (`java/spring-mvc`): Spring Security and Thymeleaf workflow with persisted decisions and audit history. |
| **Elixir** | **Incident Room** (`elixir/phoenix-api`): real-time incident coordination through Phoenix Channels. **Report Queue** (`elixir/otp-worker`): supervised durable jobs with retry handling. **Pulse Monitor** (`elixir/pulse-monitor`): bounded concurrent HTTP checks and retained history. |
| **Node.js / TypeScript** | **Planboard** (`node/react-web`): an offline-first React planning app with backup/import. **Issue Desk** (`node/service-kit`): persistent Fastify/SQLite API and event history. **Site Audit** (`node/playwright-automation`): browser-based checks, screenshots, and an inspectable report. |
| **Swift** | **Fieldnotes** (`swift/swiftui-app`): native notes with tags, pins, archive, and backup/import. **Expense Ledger** (`swift/expense-summary`): an exact-money journal with atomic imports and monthly budgets. **Focus Sessions** (`swift/focus-timer`): native menu-bar timer and persistent session history. |

Each README gives a complete example workflow, explains storage and failure behavior, and points to the idiomatic language and framework patterns in the code. Every project includes a runnable default, local dependencies, and meaningful offline tests. The apps use local persistence and do not require a stack of containers just to try the primary workflow. Local-service images and client packages remain available separately when extending an app.

Use `devline templates inspect STARTER@VERSION` to inspect an exact payload. Creating a project tests a staged copy with isolated caches and external networking denied before publishing it. In a generated project, `./devline.sh test` repeats the app's verification; `./devline.sh run` starts its default workflow, and `./devline.sh run ARGUMENTS` passes arguments to the app. Web apps expose **Open App** after their health check succeeds. Native apps open a window, and finite command-line demos print a result. The Playwright application includes an Apple Silicon WebKit browser requiring macOS 26.5 or later.

```sh
devline --offline new python/data-automation ./sales-observatory
./sales-observatory/devline.sh run
./sales-observatory/devline.sh test
```

The repository's full generated-project check is `python3 scripts/test-representative-apps.py`. It exercises all 27 apps through the CLI, verifies the outbound network denial, validates the generated verification receipt, and checks launch behavior. Use `--stack swift` or `--id go/file-index` for a focused run. Results and generated projects are retained under `.build/representative-apps/`; desktop interaction is reviewed separately from build tests.

### Transfer a library

In **Library → Contents**, use **Show All Versions** to find older releases. Select the exact docset and starter releases, then choose **Export Selected… → Documentation and Starters…**. On the other Mac, use **Import Bundle…** to inspect and import the compressed bundle. The destination needs its own toolchain and local verification for imported starters.

```sh
devline bundle export ./travel.devlinebundle \
  --doc go@1.27.3+devline.3 --doc chi@5.2.4+devline.1 --starter go/chi-api@1.2.0
devline bundle inspect ./travel.devlinebundle
devline bundle import ./travel.devlinebundle
```

Inspect unfamiliar bundles before trusting their executable starter files. The import requires `--trust-starters` for starters outside Devline’s bundled catalog.

## Prepare local services

Open **Infrastructure** to browse PostgreSQL, MySQL, RabbitMQ, Redis, MongoDB, NATS, Mailpit, Meilisearch, SeaweedFS S3, and Keycloak. Each release shows its image publisher, license, exact tag, platform digest, and offline runbook. **Download for Offline Use** saves a verified arm64 archive in Devline's image vault independently of Docker's cache. Install and start a compatible local Docker engine with Compose before using the vault to run services.

Choose services when creating a project, or use **Add Services** on the project’s page later. Devline gives the stack exact release pins, loopback ports, local credentials, and dedicated data volumes. Adding a service preserves existing selections, credentials, ports, and data. The app then prepares the project's dependencies, documentation, and image archives together. Start, stop, and view service logs from the same workspace. Stopping keeps the project's data. Use **Library → Contents** to check the offline readiness of selected image archives alongside docsets and starters.

Service networks disable outbound NAT and use local DNS settings by default while keeping their published ports reachable from your Mac. **Run** separately denies the app process external networking. Custom Docker routing or engine configuration can change container connectivity; the app's network boundary does not rely on those Docker defaults.

A service image runs the server. A client library lets your code talk to that server. The service picker shows client suggestions for the project's runtime, including package versions, offline availability, and anything already included. Optional libraries start unchecked: select **Add … to project** only when you want Devline to add that dependency. Selecting an image alone does not change your code dependencies. Built-in capabilities and unsupported combinations are identified without offering an unsuitable package.

The representative applications start with local storage. Add a service when extending an app's architecture, then use its runtime's selected client library in the application. Selecting infrastructure prepares and launches the service; it does not rewrite the app's repository or workflow to use that service.

For a starter that does not already declare a supported client, add an explicit selection such as `--client dotnet:postgres` to `devline new`. New client choices modify the project's dependency declarations and prepare their offline closure. Leave the flag out to keep the starter's dependencies unchanged.

To see suggestions for an existing project, use `devline infra clients ./project`; use `devline infra clients --stack dotnet` to browse without a project. You can add an image and its client together, or add a client to a project that already has its services:

```sh
devline infra clients ./service
devline infra add ./service --service postgres@18.6 --client dotnet:postgres
devline infra add ./service --client dotnet:rabbitmq
devline toolbox prepare --path ./service
```

An explicit client addition resolves dependencies in a staged copy, prepares their local vendor directory or cache, and verifies with external networking denied before publishing the changes. Use `--offline` to require locally available artifacts. Adding a service without `--client` only changes its service configuration; `toolbox prepare` saves the selected image archives and checks the complete project.

Add `--background` to `infra add` for a cancellable job. The app's **Add & Prepare** action shows this progress in the project, then prepares the matching docs and image archives after the additions succeed. Preferred ports are selected automatically when free; edit a port only when you need a specific value.

```sh
devline infra catalog
devline infra prepare postgres@18.6
devline infra prepare redis@8.10.1
devline --offline new go/chi-api ./issue-desk
devline infra add ./issue-desk --service postgres@18.6 --client go:postgres
./issue-desk/devline.sh services up
./issue-desk/devline.sh check
./issue-desk/devline.sh services down
```

Each generated project's README describes its actual storage. For a project you extend with services, readiness checks the selected local resources and reports whether its dependencies were verified. Use `devline toolbox check --library-only --image postgres@18.6` to check an archive independently of a project.

### Transfer images

Use **Infrastructure → Transfer** or export selected images to a local file. Import verifies the archive against the catalog's exact image identity. Restore loads it into Docker without pulling from a registry.

```sh
devline infra export ./project-images.devlineimages --image postgres@18.6 --image redis@8.10.1
devline infra import ./project-images.devlineimages --inspect
devline infra import ./project-images.devlineimages
devline --offline infra restore postgres@18.6
```

The destination needs Docker, the project toolchains, and dependencies. Importing an image pin outside the known catalog requires explicit trust. Move service data and credentials separately. [Volume backup guide](data/help/infrastructure/portable-toolbox.md)

## Updates

Open **Library → Updates** and choose **Check for Updates**. The list shows newer releases for the starters and service images saved in your offline library. Choose **Update** for one item or **Update All** for the whole list. Devline keeps project pins and, by default, previous releases. Unfinished updates stay in the list with a **Retry** button. If a publisher check fails, open **Check Details**; updates already confirmed can still be applied.

To browse downloadable docsets, read release notes, or switch the active version, use **Docs → Manage Docs**. Toolchain publisher results are in **Toolchains → Publisher Releases**. To check for a newer Devline app, use **App Updates and Downloads…** in the Devline menu or **App Updates** in Settings.

The **Documentation** tab downloads independently published snapshots. Devline checks archive size and SHA-256 and resumes interrupted downloads where the server supports it. The details show sources, coverage, and known omissions. You can roll back to an earlier installed release.

```sh
devline docs updates                 # cached channel status
devline docs updates check           # fetch the channel manifest
devline docs fetch go@1.27.3+devline.3 --background
devline docs rollback go@1.27.3+devline.2
devline updates check --background   # check all content sources
devline updates apply --background   # prepare checked content updates
devline updates apply template:go/chi-api # update or retry one item
devline --offline updates            # read saved status
```

Follow either background operation with `devline jobs status JOB_ID --follow`. Completed work is retained if another item fails, and the result identifies the remaining items. Retry after resolving the reported prerequisite. Updating library content does not rewrite projects, change their service pins, or start their containers.

Runtime updates open official installation pages. Devline 0.7.0 uses direct downloads: the app-update section and CLI link to the checked public release. Install or upgrade using the GitHub DMG. The signed Sparkle updater remains disabled in this build because no signed appcast is published. A future build can enable it only with its signing key and Ed25519-signed feed and archives. [Download Devline 0.7.0](https://github.com/fpl0/devline/releases/download/v0.7.0/Devline-macos-arm64.dmg)

Infrastructure also checks publishers for newer compatible images. `devline infra updates check` refreshes observations, and `devline infra update SERVICE` prepares the checked release. App updates can include newer catalog entries.

### Purge older releases

In **Settings → Local Library**, turn on **Keep only the latest version after updates** to purge older releases of an item when Devline installs its newest local version. The setting is off by default. Use **Purge Older Versions…** to clean existing docsets, starters, and image archives in one pass, regardless of the setting.

Cleanup applies to Devline’s local library. Each project keeps its exact pins and may need a purged release prepared again before it can work offline.

```sh
devline library retention                         # show the current setting
devline library retention --keep-latest true      # enable automatic cleanup
devline library retention --keep-latest false     # disable automatic cleanup
devline library purge                             # purge older local releases now
```

## Devline help and reference coverage

Select a service in **Infrastructure** and choose **Read Offline Reference**. Devline installs the bundled reference if needed and opens the publisher’s article. **Offline Reference Coverage** describes the bundled scope and its omissions. **Publisher Documentation** opens the service’s manual online.

The Infrastructure References pack covers all ten catalog services. It includes the complete PostgreSQL 18.6 HTML manual and RabbitMQ’s official .NET tutorial samples. MySQL and MongoDB include upstream command-line and image references, with their full database manuals linked online. Redis includes command help for the pinned image and archived publisher guides. The pack records the source version, license, and scope for each service. [Infrastructure source inventory](data/docs/infrastructure/2026.11.07+devline.4/SOURCES.md)

```sh
devline --offline docs install infrastructure --version 2026.11.07+devline.4
devline --offline docs search JetStream --docset infrastructure@2026.11.07+devline.4
devline --offline docs show 'infrastructure@2026.11.07+devline.4/reference/postgres/index'
```

Use **Read Devline Guide** or `devline guide` for setup instructions specific to Devline. [Documentation policy](DOCUMENTATION-POLICY.md)

## Source layout

- [`core/`](core/) builds the `devline` CLI and the shared `devline_core` Rust library. The core owns library storage, reference search, starter verification, project preparation, and local infrastructure. `toolbox` coordinates full project preparation and `tasks` tracks background operations. Integration tests live in `core/tests/`.
- [`macos/`](macos/) contains the SwiftUI app, grouped by feature under `Sources/DevlineApp/`. `Sources/DevlineBridge/` exposes the shared core through a C header. Native tests live in `Tests/` and the production app icon lives in `Assets/`. [Core protocol](CONTRACT.md)
- [`integrations/`](integrations/) contains the VS Code and Neovim adapters and their tests. The adapters pass the selected symbol and its context to Devline in a `devline://search` URL.
- [`data/`](data/) contains the offline library shipped with the app. `docs/` holds publisher reference packs, `help/` holds Devline guides, and `templates/` holds versioned starter payloads and build recipes. `project-adapters/` and `service-clients/` support offline project preparation. `packages/` lists each runtime’s package shelf and holds the scripts that download it. After editing `docs-catalog.json` or `templates-catalog.json`, run `python3 scripts/sync-catalog.py` to refresh the combined runtime catalog.
- [`scripts/`](scripts/) contains the tools and tests for content ingestion, packaging, and releases. `infrastructure-docs/upstream/` holds the publisher inputs used to rebuild the infrastructure reference. [CI workflow](.github/workflows/validate.yml)
- [`press/`](press/) holds the README screenshots. [`third-party/`](third-party/) holds redistribution notices and package inputs used by the release build.

## Build and validate

Documentation ingestion tests use Pandoc. Install it with `brew install pandoc` before running the Python tests. It is not needed to use the app or build projects from its starters.

On Apple Silicon, use Xcode and its command-line tools, Rust/Cargo, and Python 3.

```sh
./scripts/package.sh --no-install
./scripts/create-dmg.sh
bash ./scripts/install.sh
cargo test --locked --manifest-path core/Cargo.toml
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/sync-catalog.py --check
python3 scripts/check-doc-packs.py
python3 scripts/check-infrastructure.py
python3 scripts/seal-service-client-packs.py --check
python3 scripts/check-packages.py
```

Use the ignored `.build/` directory for local verification outputs. For example, `python3 scripts/test-service-client-journeys.py --output .build/service-client-journeys` keeps the fixtures, logs, and results outside the tracked source tree.

To exercise the six Java and C# starters through the CLI, run `cargo build --release --locked --manifest-path core/Cargo.toml` and `python3 scripts/test-java-dotnet-templates.py`. The run needs the two toolchains and uses self-contained local databases. Use `--kind cli`, `--kind api`, or `--kind mvc` for a focused run.

The build uses checked-in catalog snapshots and may download locked Rust dependencies and Sparkle. The app uses `~/Library/Application Support/Devline` for new libraries and can reopen a library from an earlier installation. Use `DEVLINE_HOME` or `--home` to isolate the library and `DEVLINE_CATALOG` or `--catalog` to select another catalog. [Storage and catalog](CONTRACT.md#storage-and-catalog)

The bundled library includes 23 docsets and keeps earlier Go and Chi revisions for pinned projects. Its Node ecosystem references cover Node.js core APIs, TypeScript, Playwright, React, and Vite. The starter catalog spans nine runtimes. [Core protocol](CONTRACT.md) · [Third-party notices](THIRD_PARTY_NOTICES.md)

Documentation source versions and licenses are recorded in each pack’s `SOURCES.md`. Rebuild Markdown snapshots with Pandoc 3.13 and `scripts/fetch-docs.py`. `scripts/fetch-desktop-docs.py --help` describes the pinned inputs for rebuilding Tauri and Swift packs into a fresh staging directory. `scripts/doc-release.py` builds and verifies independently downloadable pack archives. [GitHub Actions](https://github.com/fpl0/devline/actions)
