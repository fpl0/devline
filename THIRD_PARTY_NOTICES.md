# Third-party notices

Documentation packs include source attribution and applicable notices under `data/docs/`. The app includes the same material under `Contents/Resources/catalog/docs/`.

The Java SE reference was generated from OpenJDK sources shipped with BellSoft Liberica JDK 26.0.2.1. The exact source subset, license, and generation inputs are distributed with that pack. [Java source archive](data/docs/java/26.0.2.1+devline.1/openjdk-26.0.2.1-javadoc-sources.zip) · [Documentation ingestion script](scripts/fetch-docs.py)

Starter bundles include their pinned third-party dependencies. Go and Rust retain vendored notices. Python wheels, NuGet packages, Maven artifacts, and Hex sources retain their upstream notices. Package versions and provenance are recorded in their respective manifests and lockfiles. [Maven upstream artifact and source index](third-party/MAVEN.md)

The Devline core uses Rust crates listed in `core/Cargo.lock`. [Rust dependency notices](third-party/CARGO.md)
