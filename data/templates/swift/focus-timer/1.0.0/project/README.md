# Focus Sessions

A native macOS focus timer with a menu-bar countdown, pause/resume, configurable breaks, persistent session history, daily totals, and CSV export. It demonstrates a SwiftUI app with multiple scenes, shared observable state, an explicit timer state machine, and recoverable local persistence.

## Try a session

Use Swift 6 from Xcode 16 or later. There are no external packages or services.

```sh
./run.sh
./test-offline.sh
./build-app.sh
```

1. Name a task, set focus to one minute, and start it.
2. Close the dashboard window. The menu bar keeps counting down; click it to pause or resume. The expand button reopens history.
3. Pause for a few seconds and resume. Paused time is excluded from focused time.
4. Let the timer finish: a local sound and **Done** menu-bar label announce completion. Then start a break. Completed focus appears in history, and breaks never inflate focus totals.
5. Start another task and stop it early. Confirming the stop retains elapsed focus as an incomplete session.
6. Export CSV from the dashboard. Quit and reopen to see the same history and any active session.

The timer uses a wall-clock deadline and continues across sleep and app restarts. On reopening, a session whose deadline passed is recorded once at its original completion time. Paused sessions stay paused. Clock changes can affect a running deadline; this app does not claim to measure CPU time or detect whether you were working.

The app saves `sessions.json` under `~/Library/Application Support/<bundle identifier>/`. `FOCUS_DATA_DIR` selects another directory for testing. State is saved after every transition and preference change, not every displayed second. Atomic writes and a file lock protect persistence; a stale second instance must reload before saving. Corrupt files remain untouched and mutations stay disabled until a valid workspace can be loaded.

## Architecture

- `FocusCore/SessionJournal.swift` defines the value-based state machine. Every transition receives an explicit date, making pause, resume, sleep, and restart deterministic in tests.
- `FocusCore/FocusFile.swift` handles validation, optimistic concurrency, and atomic persistence.
- `FocusSessions/FocusSessionsApp.swift` hosts an observable store shared by the dashboard and `MenuBarExtra`. A store-owned task keeps the menu-bar display updated even when both content views are closed.
- Tests exercise elapsed-time accounting, completion at the deadline, restart recovery, invalid transitions, CSV quoting, corrupt files, and concurrent writers. The shell test also builds and verifies the signed release app.

Focus durations are 1–120 minutes; breaks are 1–30 minutes. History is local and retained until you remove the data file. Daily totals assign a session to its completion day in the Mac's local timezone. A useful extension is a weekly chart or user-defined project labels; preserve the clock-injected domain model.
