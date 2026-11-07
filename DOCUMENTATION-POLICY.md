# Documentation sources

Devline's language, framework, and service reference packs must contain official
documentation published by the relevant project or vendor. This is a product
requirement, including for offline packs and future documentation updates.

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

## Devline help

Instructions for using Devline, preparing dependencies, and running its templates
may be authored for Devline. Label and present them as Devline help, separately
from official language, framework, and service references. Adding an official
source link to an authored article does not make the article official.

## Current audit — 7 November 2026

The current catalog records official upstream sources for all 23 bundled reference families. This is a provenance audit, not a blanket legal clearance or a byte-for-byte comparison of every transformed article.

- **SwiftUI is prepared from the selected local Apple SDK.** Devline uses Apple’s declarations and documentation comments, with examples where the SDK provides them. The separately bundled Swift language book covers the language. A SwiftUI project’s documentation-readiness result reports a missing reference until a compatible local snapshot is installed. [Apple’s SwiftUI reference](https://developer.apple.com/documentation/swiftui/)
- **Infrastructure reference and help are separate.** `infrastructure@2026.11.07+devline.2` contains five PostgreSQL manual pages and one SeaweedFS API reference. Devline’s 13 setup and portability guides live under `data/help/infrastructure/` and are presented as Devline Help, outside reference search and coverage counts. The other service references remain explicit offline gaps.
- The previously published mixed infrastructure `.1` snapshot stays byte-for-byte intact as a historical archive. It is excluded from the current bundled reference catalog and current update-channel selection.

### Local SwiftUI SDK import

Devline runs Swift’s symbol graph extractor against the selected macOS SDK. It imports the public SwiftUI declarations and the SwiftUICore declarations where that module is available. It preserves the original Apple comments and examples, and formats them for offline reading. The local `SOURCES.md` records the SDK version and build, compiler, modules, and extraction method.

Some SDK symbols have declarations only. Apple Developer website articles, tutorials, and illustrations are outside this input. Keep these omissions explicit.

The extracted reference stays on the Mac that prepared it. Devline excludes it from app bundles, public documentation archives, and portable library exports. Each Mac prepares its own copy from its selected SDK.

[Apple Xcode and SDK agreement](https://www.apple.com/legal/sla/docs/xcode.pdf) · [Apple website terms](https://www.apple.com/legal/internet-services/terms/site.html) · [Xcode’s documentation viewer](https://help.apple.com/xcode/mac/current/en.lproj/devf503af6b4.html)
