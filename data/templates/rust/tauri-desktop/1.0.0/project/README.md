# Fieldnotes

A local desktop notebook built with Tauri and Rust. Create and edit notes, organise them with tags, search their contents, inspect writing statistics, and export/import a portable notebook. All content stays on this Mac.

```sh
./devline.sh test
./devline.sh run
./devline.sh build
```

Choose **New note**, add a title, tags separated by commas, and your text. **Save note** or **⌘S** commits it. Search matches title, body, and tags. Switching notes warns about unsaved edits. Delete asks for confirmation. Close and reopen the app to see saved notes again.

**Export** displays the notebook JSON: copy it into a `.json` file as a backup. **Import** accepts that JSON and adds notes as new copies without replacing existing notes. The entire import is validated before writing; a malformed import changes nothing. Unsaved editor contents are not included in exports.

## Storage and design

Notes live in the OS application-data directory for `io.devline.fieldnotes` (on macOS, `~/Library/Application Support/io.devline.fieldnotes/notes.json`). Atomic snapshot writes and version checks protect saved state; a file lock prevents two app instances writing the same notebook. Corrupt data fails startup instead of being erased. Restore backups while the application is stopped.

Set `FIELDNOTES_DATA_DIR` to use a separate notebook directory, for example `FIELDNOTES_DATA_DIR="$PWD/.fieldnotes-demo" ./devline.sh run`. Both the notes and the single-instance lock live in that directory.

`src/notebook.rs` contains the domain and persistence layer. `src/main.rs` exposes narrow commands scoped to the bundled main window. Web content cannot choose filesystem paths. The frontend renders user text with `textContent`, uses a restrictive content security policy, and has no external requests, framework, or Node build step.

Limits: 1000 notes, 256 KiB per note, 12 tags per note, 16 MiB for a complete serialized notebook (and imports), ensuring every valid export can be restored. This is a single-user notebook; it does not sync across devices. Tests cover restart persistence, concurrent-writer rejection, stale writes, atomic imports, corruption, embedded assets, and IPC window scope.

The release build creates a locally signed **Fieldnotes.app** in `target/bundle/macos/`. Cargo dependencies and web assets are bundled, so build and test work offline with Rust and Apple's native build tools.
