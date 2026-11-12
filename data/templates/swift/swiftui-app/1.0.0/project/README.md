# Fieldnotes

A native macOS notes workspace for collecting research, organising it with tags, pinning current work, and archiving finished ideas. It includes search, editing, JSON backup/import, and individual Markdown exports. SwiftUI provides the split view, keyboard shortcuts, sheets, native file panels, and accessibility labels.

## Try the app

Use Xcode 16 or later with Swift 6 selected in **Xcode → Settings → Locations → Command Line Tools**. No network dependencies are required.

```sh
./run.sh
./test-offline.sh
./build-app.sh
```

1. Create a note with **⌘N**, add a title and body, and enter comma-separated tags such as `research, swift`.
2. Pin the note, then create a second one. Pinned notes stay above other notes; each group sorts by last edit.
3. Search for words from the title, body, or tags. All search terms must match. Combine search with a tag and the Notes/Archive filter.
4. Archive a note, select **Archive**, and restore it. Permanent deletion requires confirmation.
5. Export a note to Markdown, then export the whole workspace through the toolbar menu. Importing the same backup twice is safe. A changed note with the same ID stops the entire import so neither version is silently lost.
6. Quit and reopen to verify persistence.

Build produces a locally signed `.app` under `.build/apps/`. The app saves to `~/Library/Application Support/<bundle identifier>/workspace.json`; generated projects get their own bundle identifier. `FIELDNOTES_DATA_DIR` selects an isolated workspace directory for testing. The workspace menu can reload external changes. A stale writer cannot overwrite newer saved notes, and malformed files are preserved with an error. Save failures leave edits open. If the original note changed or was deleted during editing, reload and choose **Save as New** to preserve your draft without replacing the other version. The previous simple Idea Board used `ideas.json`; this app uses a separate workspace format and does not overwrite that older file.

## Design and verification

`IdeaBoardCore` is a small value-based model with explicit mutation methods, validation, search, and conflict-safe backup merging. `BoardFile` separates persistence, uses a file lock, checks the expected previous state, and atomically replaces the file. The SwiftUI view publishes a mutation only after saving succeeds. `IdeaBoard` remains the executable name so the existing build scripts and generated app lifecycle stay stable.

Tests cover edit/archive/search behavior, tags and pins, round-trip persistence, corrupt input preservation, import conflicts, and stale writers. The offline test also builds the release `.app` and verifies its property list and code signature. The app accepts titles up to 200 characters, bodies up to 200,000 characters, and twelve tags up to forty characters each.

Extend it with notebooks, Markdown preview, or a storage protocol backed by SQLite. Keep import validation and save-before-publish semantics when changing persistence.
