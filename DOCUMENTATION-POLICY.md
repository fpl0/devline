# Documentation sources

Reference packs published by Devline must contain official language, framework,
or service documentation from the relevant project or vendor. This requirement
applies to bundled packs and documentation updates published by Devline.

## Reference content

- Import from the publisher's documentation repository, release archive, or
  documented distribution. Documentation generated from official source comments
  is acceptable when its source and generation method are recorded.
- Preserve the upstream explanation, examples, declarations, and attribution.
  Formatting changes for offline rendering are acceptable; generated prose or
  summaries must not replace upstream reference material. Never use AI-generated
  explanations, rewrites, summaries, or examples to replace publisher documentation
  or fill a gap in a reference pack.
- Record the publisher, source URL, exact release or commit where available,
  snapshot date, applicable license, and transformations in each pack's
  `SOURCES.md`. Article links must identify their actual upstream source.
- Verify applicable redistribution terms before putting upstream content into
  public app bundles or docset archives. Public access alone is not evidence of
  permission to redistribute it.
- Report partial coverage and omitted APIs explicitly. An unavailable official
  offline reference remains a documented gap; authored guides do not fill that
  gap for readiness or coverage claims.

## Personal DevDocs imports

Users can import snapshots published by DevDocs into their own library. Identify
the snapshot as a DevDocs import, preserve available upstream attribution and
source links, and keep it under a separate pack ID from any reference Devline
publishes. Project preparation selects its official references from Devline's
curated catalog.

These imports are local-only. The app ships the importer and excludes imported
snapshots from its public catalog, app releases, documentation channels, and
portable library exports. DevDocs packages can carry distinct terms from their
original publishers. See the [DevDocs project and attribution notes](https://github.com/freeCodeCamp/devdocs#copyright--license).

## Devline help

Instructions for using Devline, preparing dependencies, and running its templates
may be authored for Devline. Label and present them as Devline help, separately
from official language, framework, and service references. Adding an official
source link to an authored article does not make the article official.

## Service client recommendations

A service client package and an official reference pack are different content types. Devline may recommend publisher-maintained or community-maintained clients, or a runtime’s standard HTTP, SMTP, or cryptography APIs, when their scope is clearly identified. Record the maintainer source, exact package version, and support category in the client catalog. A general HTTP client must not be described as a dedicated service SDK.

Client package archives, source code, license notices, and Devline’s integration instructions do not fill a service’s missing official documentation coverage. The reference-pack requirements above continue to apply unchanged. Offline client tests prove the recorded package/build/API checks; successful live service integration requires a separate test that actually connects to that service.

## Content-depth audit — 12 November 2026

Official provenance alone does not establish completeness. The earlier importers
sampled several manuals and discarded some generated API documentation. The
following new immutable revisions use the same pinned publisher versions and
expand their content:

| Reference | Previous articles | New articles | Scope restored |
| --- | ---: | ---: | --- |
| Go | 95 | 328 | All 180 public standard-library packages for darwin/arm64, 145 verbatim publisher example files, and three language references |
| Node.js | 33 | 69 | Every page listed in the pinned official API contents, including the contents page |
| React | 34 | 187 | Learning, React and React DOM references, compiler documentation, warnings, publisher examples and available illustrations |
| TypeScript | 19 | 132 | English handbook, reference, modules, declarations and tutorials; all 137 TSConfig options |
| Vite | 15 | 39 | All guide and configuration articles, including environment APIs and plugin development |
| Playwright | 29 | 148 | JavaScript guides and all five API source directories, including test-runner and reporter APIs |
| FastAPI | 151 | 150 | Constructor parameters, `Annotated` publisher descriptions, methods, attributes and inherited Starlette declarations previously lost during extraction |

Each expanded pack includes `CONTENT-INVENTORY.json` with its selected publisher
sources and exclusions. The final pack audit compares that inventory with emitted
articles, so a short page or a failed conversion cannot silently disappear.
Importer tests also cover source examples, shared API parameter documentation,
inherited members, illustrations and anchors. Explicitly selected FastAPI members
must resolve against the pinned sources or the build fails. FastAPI excludes its
dynamically populated People community page and the home page’s sponsor-card
block; its educational content, official sponsor link, and instructional Jinja
examples are retained. Unresolved site templates in imported prose fail the build.

These are coverage improvements, not claims that every ecosystem is complete.
Go's declarations follow the selected platform; external dependencies have their
own manuals. Interactive playgrounds are retained as static publisher examples.
Known broken publisher links remain listed separately from import regressions.
The full .NET API catalog, ASP.NET MVC/Razor Pages/Blazor/SignalR/gRPC guides,
Rails API catalog, unselected Java/Ruby/Rust APIs, Ecto SQL, Phoenix LiveView,
and the separate Spring Framework reference remain gaps. Apple's website tutorials
are still outside the local SwiftUI SDK extraction. MySQL and MongoDB's full
manuals retain the redistribution restrictions recorded in their source manifests.

## Provenance audit — 7 November 2026

The current catalog records official upstream sources for all 23 bundled reference families. This is a provenance audit, not a blanket legal clearance or a byte-for-byte comparison of every transformed article.

- **SwiftUI is prepared from the selected local Apple SDK.** Devline uses Apple’s declarations and documentation comments, with examples where the SDK provides them. The separately bundled Swift language book covers the language. A SwiftUI project’s documentation-readiness result reports a missing reference until a compatible local snapshot is installed. [Apple’s SwiftUI reference](https://developer.apple.com/documentation/swiftui/)
- **Infrastructure reference and help are separate.** `infrastructure@2026.11.07+devline.4` contains official references for the ten catalog releases, including RabbitMQ’s .NET tutorial examples from its pinned sample repository. Each service’s source manifest records its exact image release, documentation provenance, scope, and omissions. The app surfaces coverage in Infrastructure. Devline’s 13 setup and portability guides live under `data/help/infrastructure/` and are presented as Devline Help, outside reference search and coverage counts. [Infrastructure source inventory](data/docs/infrastructure/2026.11.07+devline.4/SOURCES.md)
- The retired mixed infrastructure `.1` snapshot is archived in the `v0.7.0` tag. Its published source links still use the original guide paths under `scripts/infrastructure-docs/`. [Archived snapshot](https://github.com/fpl0/devline/tree/v0.7.0/data/docs/infrastructure/2026.11.07%2Bdevline.1)

### Infrastructure snapshot maintenance

For a new catalog release, update its manifest under `scripts/infrastructure-docs/upstream/` using the publisher’s redistributable material. Keep the publisher notice and exact documentation version, including when the docs follow a release series or a separately versioned client. Build a new immutable revision with Pandoc 3.13 using `python3 scripts/fetch-docs.py --cache <scratch-dir> --reference <scratch-dir> --only infrastructure`. The infrastructure build reads its checked-in sources and can run offline. Keep releases required by existing starter pins available.

Update each catalog service’s offline entry point and coverage from its source manifest, then run `python3 scripts/sync-catalog.py` to refresh the combined runtime catalog. `scripts/check-infrastructure.py` verifies the catalog and source inventory across all shards. `scripts/check-doc-packs.py` audits the rendered articles and checks examples, assets, and links in the pilot releases. It tracks known source omissions separately from regressions.

### Local SwiftUI SDK import

Devline runs Swift’s symbol graph extractor against the selected macOS SDK. It imports the public SwiftUI declarations and the SwiftUICore declarations where that module is available. It preserves the original Apple comments and examples, and formats them for offline reading. The local `SOURCES.md` records the SDK version and build, compiler, modules, and extraction method.

Some SDK symbols have declarations only. Apple Developer website articles, tutorials, and illustrations are outside this input. Keep these omissions explicit.

The extracted reference stays on the Mac that prepared it. Devline excludes it from app bundles, public documentation archives, and portable library exports. Each Mac prepares its own copy from its selected SDK.

[Apple Xcode and SDK agreement](https://www.apple.com/legal/sla/docs/xcode.pdf) · [Apple website terms](https://www.apple.com/legal/internet-services/terms/site.html) · [Xcode’s documentation viewer](https://help.apple.com/xcode/mac/current/en.lproj/devf503af6b4.html)
