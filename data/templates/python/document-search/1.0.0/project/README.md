# Field Notes Search

A local search engine for Markdown, plain text, and reStructuredText notes. Build an incremental SQLite FTS5 index, rank matches, inspect snippets, and print the original indexed document. No model, service, or third-party Python package is required.

The empty `wheelhouse` directory is intentional: Devline's Python adapter uses it to prepare the isolated environment offline, with no packages to install.

## Try it

```sh
./test-offline.sh
./run.sh
```

The example indexes `examples/notes` and searches for “reliable delivery”. In a generated project, index your own directory and query it:

```sh
./devline.sh exec python3 -m fieldnotes index ./examples/notes
./devline.sh exec python3 -m fieldnotes search 'distributed systems'
./devline.sh exec python3 -m fieldnotes search 'reliable delivery' --json --limit 10
./devline.sh exec python3 -m fieldnotes show 1
./devline.sh exec python3 -m fieldnotes stats
```

Run `index` again after changing or deleting notes. Unchanged content is skipped by SHA-256; changed files update the index; deleted files disappear from search. Multiple source directories can coexist in one database. The `--database PATH` option goes before the subcommand and defaults to `data/notes.sqlite3`.

The index accepts `.md`, `.txt`, and `.rst` files up to 2 MiB in UTF-8. Hidden paths and symbolic links are excluded. Unreadable, binary, and oversized files are reported under `skipped`; an already indexed file that becomes unreadable keeps its last usable version. Source files are never edited. Search treats query words literally and requires all of them; FTS operators are not exposed as a second query language.

## Design and tests

`fieldnotes/index.py` contains the storage boundary, incremental synchronization, FTS triggers, and ranking. `fieldnotes/__main__.py` provides the CLI. An index update runs in one SQLite transaction, and title matches receive a higher ranking weight than body matches. Query text is parameterized.

Tests exercise persistence, ranking, incremental edits/deletions, independent roots, Unicode, literal queries, hidden files, symlink exclusion, invalid bytes, size limits, and command validation. Use `python3 -m fieldnotes --help` directly when working in the source bundle.

A useful next extension is a small local web interface over this index. Keep the index as a reusable service class and add interface tests separately from the storage tests.
