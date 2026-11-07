# __PROJECT_NAME__

A native macOS app built with Tauri 2 and a bundled HTML, CSS, and JavaScript interface. The Local Text Inspector counts words, lines, Unicode scalars, and UTF-8 bytes through a Rust command. It keeps the input in memory.

## Run offline

Requires Apple Silicon, macOS 14 or later, Rust and Cargo 1.99.1 or later, and the Apple Command Line Tools or Xcode. Cargo uses `Cargo.lock` and the included `vendor/` directory. The frontend uses system WebKit and needs no Node.js, package manager, dev server, or Tauri CLI.

```sh
./devkit-test.sh  # Devline's selected toolchain in a generated project
./run.sh          # Run from source with Cargo on PATH
./build-app.sh    # Build an ad hoc signed macOS app bundle
```

`./test-offline.sh` also works directly when Cargo is on `PATH`. Set `DEVKIT_TOOLCHAIN` to the full path of another compatible Cargo. The bundle is written to `target/bundle/macos/Local Text Inspector.app` unless you override `CARGO_TARGET_DIR`.

The local bundle can be opened in Finder. Public distribution has separate code-signing, notarization, and [third-party notice](THIRD_PARTY_NOTICES.md) requirements. Update the app metadata in `tauri.conf.json` and `packaging/Info.plist` when you rename the app or choose a new bundle identifier.

## Project layout

- `src/main.rs` validates input, computes counts, and registers the Tauri command.
- `web/` holds the embedded assets. The frontend invokes only `inspect_text` through Tauri's local IPC bridge.
- `capabilities/main.json` grants that command to the `main` window. `tauri.conf.json` sets the window and content security policy.
- `vendor/` contains the locked Cargo sources with their original licenses and notices.

Devline's build-latest recipe resolves updated semver-compatible dependencies while connected and verifies the resulting starter with external networking blocked.
