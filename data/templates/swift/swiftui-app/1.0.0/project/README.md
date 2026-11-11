# Idea Board

A small native macOS app for keeping track of experiments and side projects. Add an idea, write a few notes, mark it done, and find it again by searching its title or notes. The board saves locally as JSON in Application Support.

Select a local Xcode 16 or later with Swift 6 and the macOS SDK. The test suite uses Xcode’s Swift Testing framework. The app targets macOS 14 or later. Check the selected tools with `xcodebuild -version` and `xcrun --sdk macosx --show-sdk-path`.

```sh
./run.sh                  # build and open Idea Board
./build-app.sh            # create a locally signed .app under .build/apps/
./test-offline.sh         # fresh SwiftPM scratch space, model tests, and bundle verification
```

Choose **Run** in Devline to launch and manage the window. In a generated project, use `./devline.sh start`, `./devline.sh stop`, `./devline.sh build`, and `./devline.sh test`. All commands use local SwiftPM targets and Apple's SDK. The app has zero third-party packages.

- `Sources/IdeaBoard` contains the SwiftUI window, sidebar, search, and editor.
- `Sources/IdeaBoardCore` contains the idea model and JSON file storage.
- `Tests/IdeaBoardCoreTests` covers editing, filtering, and persistence using Swift Testing.

Open `Package.swift` in Xcode to work on the source. Use `./run.sh` to run the app bundle with its project-specific storage location. The local signature is for running the app on your Mac.

[SwiftUI App](https://developer.apple.com/documentation/swiftui/app) · [Swift Package Manager](https://docs.swift.org/swiftpm/documentation/packagemanagerdocs/) · [Swift Testing](https://developer.apple.com/documentation/testing)
