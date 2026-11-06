# Devline core protocol

The SwiftUI app and Rust CLI share a Rust core. The macOS app sends JSON to the C bridge on a background task. The CLI calls the same core directly.

## Storage and catalog

The library defaults to `~/Library/Application Support/DevKit`. The `DEVKIT_HOME` and `DEVKIT_CATALOG` environment variables override the library and catalog directories. Requests may also supply `home` and `catalog` explicitly. The app passes its bundled `Resources/catalog` path by default.

The catalog contains `catalog.json`, `docs/`, and `templates/`. `catalog.json` has `schema_version: 1` and combines the `docs` and `templates` arrays from `docs-catalog.json` and `templates-catalog.json`.

- A documentation pack specifies `id`, `name`, `stack`, `version`, `description`, `source_url`, `license`, and `path`. Its directory contains `documents.json`, with `id`, `title`, `body`, `source_url`, optional self-contained sanitized `html`, `aliases`, and `category` per record. A pack can declare `shards` and continue with `documents-2.json` and subsequent numbered files. Include the upstream license and source notices in the pack directory. Installed record IDs take the form `<pack>@<version>/<record>`. Long references may have `::part-N` continuations. Changes to content require a new version. Use `<upstream>+devline.N` for a pack revision that retains its upstream version.
- A pack can declare `markdown` path selectors. A selector is an exact record path or a pattern with one `*` wildcard. For example, `"markdown": ["guides/*.md"]` marks guide bodies for CommonMark and GFM rendering when opened. The response retains the original body. The renderer omits raw HTML and images and limits generated links to HTTP, HTTPS, and local anchors. An empty selector list marks the pack as plaintext. Older bundled catalogs can use built-in format defaults.
- A starter specifies `id`, `name`, `stack`, `version`, `description`, `app_type`, `path`, `toolchain`, `minimum_version`, `test_command`, `docs`, `verified`, and `target`. Its directory contains `project/` and can include a `build-latest.sh` recipe. Document references can pin versions as `<pack>@<version>`. Use relative dependency paths and `__PROJECT_NAME__` for the generated name. Release installation and creation verify the local payload, and project creation requires a verified release for the current target.

## C bridge

`char *devkit_request(const char *json)` accepts a NUL-terminated UTF-8 JSON request and returns an owned NUL-terminated JSON response. Release it exactly once with `devkit_free(char *response)`.

Every request has a `command`. A success response contains `{"ok":true,"data":...}`. A failure contains `{"ok":false,"error":"…"}`. Optional request fields may be omitted.

### Library and documentation

- `catalog` returns `{docs, templates, home}`. Catalog entries include an `installed` flag for their exact version. The templates array also includes locally rebuilt releases.
- `docs.install` takes `{id, version?}` and returns `{id, version, count}`. `docs.remove` takes `{id, version}` and returns `{removed}`.
- `docs.search` takes `{query?, stack?, project_stack?, pack_id?, version?, category?, limit?, offset?}`. It returns an array of `{id, pack_id, title, stack, version, source_url, category, snippet, match_kind, symbol?, match_text?}`. The categories are `guide`, `api`, `compatibility`, and `reference`. Omit `category` or use `all` to include them all. `match_kind` is `symbol` or `article`, and `match_text` can identify the matched declaration. An empty query browses in title order. Only browse requests use `offset`. The default `limit` is 30 and the maximum is 100.
- Search and symbolic lookup use the latest installed release of each eligible pack by default. A recognized query prefix such as `py:Path.read_text`, `go,chi:Router`, or `fastapi@0.162.0:First Steps` limits the lookup. These prefixes intersect explicit filters and take precedence over the fallback `project_stack`. A full record ID can open any installed revision it identifies.
- `docs.show` takes `{id, query?, focus_text?, stack?, project_stack?, pack_id?, version?, terminal_format?}`. It returns `{id, pack_id, title, stack, version, source_url, category, body, html, focus_text?}` and, for split articles, `{part, parts, previous_id?, next_id?}` with one-based part numbers. `html` may be null. `focus_text` is a suggested reader jump target. If `terminal_format` is true and the source is Markdown, the response also includes `terminal_markdown` with any fence needed to render a continuation.
- `docs.resolve` takes `{url, id?, pack_id?, version?}` and returns `{id, anchor?, focus_text?}` or null when the link cannot be resolved to an installed destination. For stored HTML, a fragment must match a local anchor, a verified replacement, or a standalone article addressed by that fragment. Provide the originating article as `id` to resolve a relative link or stay within its installed release when following links to the same pack.
- `project.context` takes `{path}` and returns `{path, stacks, stack}`. The core looks for project manifests in the folder and its ancestors. `stack` is null when there is no single match.

### Saved and recent references

- `reading.list` returns `{saved, recent}`, each an array of `{id, title, pack_id, stack, version, source_url, installed, saved_at, visited_at, anchor, focus_text}`. Timestamps are Unix seconds or null. Anchor and focus text are nullable. Each list is newest first, and Recent holds the last 100 distinct references.
- `reading.save` and `reading.visit` take `{id, anchor?, focus_text?}` for an installed record. They keep the saved location and the last visit independent. `reading.remove` takes `{id}` and removes only the saved reference. Each mutation returns the current `{saved, recent}` snapshot. Removing a pack leaves its stored references identifiable with `installed: false`.

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
