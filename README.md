# Devline

Devline is a macOS app and CLI for trying frameworks and building prototypes offline. Prepare a project while you’re online, then look up APIs, build and test your code, and start local services without a connection. The app and CLI share the same library on your Mac.

[Download Devline 0.7.0 for Apple Silicon](https://github.com/fpl0/devline/releases/download/v0.7.0/Devline-macos-arm64.dmg) · [Usage guide](USAGE.md)

## What you can do

- **Read the docs for your project.** Search official references with code examples and links between installed pages. Devline matches documentation to supported project dependencies and lets you pin an exact release.
- **Start a project or bring your own.** Create a FastAPI service, a React app, or a native SwiftUI app from a starter with dependencies included. For an existing project, **Prepare Offline** collects what’s missing and tests a copy with internet access blocked, leaving your source and lockfiles intact.
- **Run local services.** Save exact images of services like PostgreSQL, Redis, and RabbitMQ while connected. Restore and start them offline with a local Docker engine and Compose.

![Python documentation in Devline showing the Path.read_text signature and a code example](press/docs-python-dark.png)

## Get started

Devline requires an Apple Silicon Mac running macOS 14 or later. Open the DMG, drag **Devline** into **Applications**, and follow the first-launch instructions in the DMG.

Open a project folder with **⌘O**, or choose **New from Starter** in **Projects**. Install the matching toolchain if Devline reports it missing. Choose **Prepare Offline** to see what’s ready and what needs attention.

### From the terminal

In **Settings → Command Line**, choose **Install CLI**. With a compatible Go toolchain installed, prepare the Chi starter and its docs, then create and test a small API offline.

```sh
devline prepare go/chi-api@1.2.0
devline --offline docs show 'go:http.Handler'
devline --offline new go/chi-api@1.2.0 ./status-api
devline --offline project --path ./status-api run -- go test ./...
```

### Import DevDocs

In **Docs → Manage Docs → Import from DevDocs**, choose a docset and release. After it downloads, find it under **Imported**. You can also import a docset from the CLI:

```sh
devline docs import-devdocs --online --doc astro
```

For local archives and more options, see the [DevDocs import guide](USAGE.md#import-devdocs-into-your-local-library).

[Editor integrations](integrations/README.md) · [Build from source](USAGE.md#build-and-validate) · [Core protocol](CONTRACT.md) · [Third-party notices](THIRD_PARTY_NOTICES.md)
