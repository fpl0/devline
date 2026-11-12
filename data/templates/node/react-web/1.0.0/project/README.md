# Planboard

A local-first project planner built with React, TypeScript, and Vite. Organise projects, capture tasks with notes/priorities/due dates, move work through a three-column board, search and filter, and export/import backups.

```sh
./devline.sh test
./devline.sh run
```

Open the URL printed by Vite. Explore the example workshop and learning projects, create a project, add a task, move its status, then reload: changes remain in this browser. Task titles open the editor. Project settings let you rename or delete a project and its tasks. Export a backup before importing, clearing browser storage, or deleting substantial work.

## Persistence and recovery

The entire validated board is saved under `planboard.v1` in localStorage before React publishes an edit. A failed quota/write leaves the existing state unchanged and shows an error. A revision check rejects edits from a stale tab. Corrupt stored data blocks changes and offers a raw export before an explicit reset; it is never silently replaced.

Exports are versioned JSON, validated on import. Imports replace the board after confirmation. Limits: 100 projects, 5000 tasks, 8000 characters per note, 5 MiB per serialized UTF-8 backup (enforced before saving, exporting, and importing). Browser storage limits may be smaller. Data belongs to this browser and origin (including the port); there is no remote account or sync. Keep the local Vite server running to serve the app; all its assets and dependencies are bundled for disconnected use.

## Code and verification

`src/model.ts` contains the Zod schema, pure domain operations, search and transactional browser-storage boundary. `App.tsx` presents the board and edit/import flows. No backend or external network is used. React renders user text safely and controls are labelled for keyboard and assistive technology.

Tests cover create/edit/move/reload through the UI, project creation, search, malformed backups, orphan/duplicate IDs, calendar-date validation, stale tabs, storage failure, and corruption recovery. `test-offline.sh` also typechecks and produces a production build with local dependencies.
