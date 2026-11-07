# Third-party notices

Documentation packs include source attribution and applicable notices under `data/docs/`. The app includes the same material under `Contents/Resources/catalog/docs/`.

The Java SE reference was generated from OpenJDK sources shipped with BellSoft Liberica JDK 26.0.2.1. The exact source subset, license, and generation inputs are distributed with that pack. [Java source archive](data/docs/java/26.0.2.1+devline.2/openjdk-26.0.2.1-javadoc-sources.zip) · [Documentation ingestion script](scripts/fetch-docs.py)

Local Infrastructure includes selected PostgreSQL 18 manual entries and the SeaweedFS 4.52 HTTP API guide. Their original notices accompany the offline pack. Service images are fetched from their publishers' registries when prepared. Image identity, publisher, source, and license links are recorded per release. [Infrastructure source notices](data/docs/infrastructure/2026.11.07+devline.2/SOURCES.md) · [Image provenance](data/infrastructure.json)

Starter bundles include their pinned third-party dependencies. Go and Rust retain vendored notices. Python wheels, Ruby gems, NuGet packages, Maven artifacts, Hex sources, and npm packages retain upstream license information. Package versions and provenance are recorded in their respective manifests and lockfiles. [Maven upstream artifact and source index](third-party/MAVEN.md) · [npm upstream notices and native package sources](third-party/NPM.md)

The Playwright starter includes Microsoft's Apple Silicon WebKit 27.2 build 2388 for Playwright 1.65.0. The browser archive preserves the upstream executable and frameworks. Exact source instructions, the corresponding Playwright patch, and the WebKit, ANGLE, WebRTC, and other upstream license notices ship alongside it. [Browser provenance](data/templates/node/playwright-automation/1.0.0/project/BROWSER.json) · [Browser sources and notices](data/templates/node/playwright-automation/1.0.0/project/browser-archive/SOURCES.md) · [WebKit licensing](https://webkit.org/licensing-webkit/)

The Devline core uses Rust crates listed in `core/Cargo.lock`. [Rust dependency notices](third-party/CARGO.md)

The macOS app embeds [Sparkle 2.10.1](https://github.com/sparkle-project/Sparkle/releases/tag/2.10.1) for authenticated app updates. [Sparkle and bundled component licenses](third-party/sparkle/LICENSE)

The Tauri documentation snapshot retains MIT and Apache-2.0 notices from its pinned sources. **The Swift Programming Language** snapshot retains Apache 2.0 with the Swift Runtime Library Exception and Swift.org attribution. [Tauri sources](data/docs/tauri/2.12.5+devline.1/SOURCES.md) · [Swift sources](data/docs/swift/6.4.0+devline.1/SOURCES.md)

SwiftUI imports use Apple’s original declarations, documentation comments, and available examples from the selected local macOS SDK. The local `SOURCES.md` records the SDK build, compiler, modules, and extraction method. Devline stores these snapshots on the Mac that generated them and excludes them from releases and portable exports. [Documentation policy](DOCUMENTATION-POLICY.md) · [Apple Xcode and SDK agreement](https://www.apple.com/legal/sla/docs/xcode.pdf)

The Tauri desktop starter keeps the complete Cargo-vendored source and notices for its locked dependencies, including target-specific crates. The SwiftUI starter has no third-party package dependencies and uses the user’s installed Xcode SDK. [Tauri starter provenance](data/templates/rust/tauri-desktop/1.0.0/project/THIRD_PARTY_NOTICES.md)
