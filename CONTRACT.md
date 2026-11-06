# Devline core protocol

The SwiftUI app and Rust CLI share a Rust core. The macOS app sends JSON to the C bridge on a background task. The CLI calls the same core directly.

## Storage and catalog

The library defaults to `~/Library/Application Support/DevKit`. The `DEVKIT_HOME` and `DEVKIT_CATALOG` environment variables override the library and catalog directories. Requests may also supply `home` and `catalog` explicitly. The app passes its bundled `Resources/catalog` path by default.

The catalog contains `catalog.json`, `docs/`, and `templates/`. `catalog.json` has `schema_version: 1` and combines the `docs` and `templates` arrays from `docs-catalog.json` and `templates-catalog.json`.

- A documentation pack specifies `id`, `name`, `stack`, `version`, `description`, `source_url`, `license`, and `path`. Its directory contains `documents.json`, with `id`, `title`, `body`, `source_url`, optional self-contained sanitized `html`, and optional `aliases` per record. Include the upstream license and source notices in the pack directory. Installed record IDs take the form `<pack>@<version>/<record>`. Long references may have `::part-N` continuations. Changes to a pack's content require a new version.
- A pack can declare `markdown` path selectors. A selector is an exact record path or a pattern with one `*` wildcard. For example, `"markdown": ["guides/*.md"]` marks guide bodies for CommonMark and GFM rendering when opened. The response retains the original body. The renderer omits raw HTML and images and limits generated links to HTTP, HTTPS, and local anchors. An empty selector list marks the pack as plaintext. Older bundled catalogs can use built-in format defaults.
- A starter specifies `id`, `name`, `stack`, `version`, `description`, `app_type`, `path`, `toolchain`, `minimum_version`, `test_command`, `docs`, `verified`, and `target`. Its directory contains `project/` and can include a `build-latest.sh` recipe. Document references can pin versions as `<pack>@<version>`. Use relative dependency paths and `__PROJECT_NAME__` for the generated name. Release installation and creation verify the local payload, and project creation requires a verified release for the current target.

## C bridge

`char *devkit_request(const char *json)` accepts a NUL-terminated UTF-8 JSON request and returns an owned NUL-terminated JSON response. Release it exactly once with `devkit_free(char *response)`.

Every request has a `command`. A success response contains `{"ok":true,"data":...}`. A failure contains `{"ok":false,"error":"…"}`. Optional request fields may be omitted.

### Library and documentation

- `catalog` returns `{docs, templates, home}`. Catalog entries include an `installed` flag for their exact version. The templates array also includes locally rebuilt releases.
- `docs.install` takes `{id, version?}` and returns `{id, version, count}`. `docs.remove` takes `{id, version}` and returns `{removed}`.
- `docs.search` takes `{query?, stack?, pack_id?, version?, limit?, offset?}`. It returns an array of `{id, pack_id, title, stack, version, source_url, snippet, match_text?}`. An empty query browses the scoped library in title order. Only browse requests use `offset`. The default `limit` is 30 and the maximum is 100.
- `docs.show` takes `{id, query?, stack?, pack_id?, version?}`. `id` can be a canonical record ID or a resolvable reference. It returns `{id, pack_id, title, stack, version, source_url, body, html, focus_text?}`. `html` may be null. `focus_text` is a suggested reader jump target.
- `project.context` takes `{path}` and returns `{path, stacks, stack}`. The core looks for project manifests in the folder and its ancestors. `stack` is null when there is no single match.

### Starters and runtimes

Starter commands accept `id` as a family such as `go/chi-api` or a specific release such as `go/chi-api@1.1.0`.

- `templates.list` returns starter releases with their `installed` flags. `templates.install` takes `{id}` and installs from the local catalog. `templates.build` takes `{id}` and runs the online dependency refresh and offline verification before publishing a new local release. Both return `{id, version, path, verified, installed}`.
- `doctor` takes `{id}` and returns `{ready, checks, template}`. Each check contains `{name, ok, detail}`. `prepare` takes `{id}`, installs the starter and its related doc packs, and returns `{template, docs, doctor}`.
- `new` takes `{id, destination, dry_run?}` and returns `{path, template, version, test_command, toolchain_path, dry_run}`. The parent directory must exist, and the target directory must be available. A dry run performs readiness checks without creating the project.
- `runtime.list` returns `{id, name, toolchain, minimum_version, available, path, detail, docs, templates}` for each stack represented in the starter catalog. `minimum_version` describes the starter baseline. `detail` reports the detected executable or the specific setup requirement.

### Updates

`updates.status` uses installed releases, the local catalog, and cached observations. `updates.check` explicitly contacts upstream sources and records the results. `updates.configure` takes `{url}` and sets the HTTPS app manifest URL. An empty string disables app checks. Configure does not fetch the manifest.

All three commands return `{checked_at, items}`, with `checked_at` in Unix seconds or null. Each item contains `{id, kind, name, current_version, available_version, state, detail, action, reference, url}`. `kind` is `docs`, `template`, `runtime`, or `app`. `state` is `current`, `available`, `unchecked`, `error`, or `unconfigured`. `action` is `install_docs`, `install_template`, `rebuild_template`, `open_url`, or null. The first three actions use `reference`. `open_url` uses `url`.

An app release manifest is `{"schema_version":1,"version":"0.2.0","url":"https://example.com/releases/0.2.0"}`. Use HTTPS URLs without credentials, queries, or fragments. The default channel is the `release.json` asset of the [latest Devline GitHub release](https://github.com/fpl0/devline/releases/latest).
