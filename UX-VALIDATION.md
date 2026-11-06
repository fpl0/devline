# Devline 0.2.0 experience verification

The installed native app was exercised on macOS 27.2 on this Apple Silicon Mac after packaging. Automatic near-end pagination was exercised on this system; the macOS 14 inline Load More fallback was compiled but was not tested on a macOS 14 host.

| Flow | Verified result |
| --- | --- |
| Browse docs | All 15 installed packs appear as clickable groups; a pack opens its exact version. |
| Search and read | `http.Handler` selects the Go declaration in the correct continuation. Prose wraps as paragraphs; declarations and examples remain monospaced. |
| Read Markdown | FastAPI and C# render headings, paragraphs, inline code, code blocks and lists. The FastAPI outline jumps to Recap, and the body scrolls independently. |
| Load more articles | FastAPI loads 80 → 131 and reaches its final article; C# loads 80 → 160 → 240 → 320. The selected article remains open and the list stays in place after layout settles and the reader scrolls. |
| Choose a starter | Ten releases appear as seven starter families, with an exact release picker. |
| Create a project | Readiness starts with a compact summary. A Go project created in the app passed its generated test with external networking denied and a system-only PATH. |
| Inspect runtimes | Seven detected toolchains show versions and relevant docs/starter actions; executable paths are under a disclosure. |
| Check updates | Cached status opens locally. The Available filter shows verified upstream differences and distinguishes source links from install/rebuild actions. |

[Documentation screenshot](press/devline-docs.png) · [Templates](press/templates.png) · [Runtimes](press/runtimes.png) · [Updates](press/updates.png)

The Rust suite has 25 passing tests and Clippy passes with warnings denied. Source and installed CLI both report 0.2.0; the packaged `devkit` alias launches `devline`. The DMG was mounted read-only and its application passed strict signature verification.

Markdown is rendered from the installed pack at read time, without reimporting it. Some older snapshots lost links during ingestion; restoring those links requires a new pack snapshot.

This initial release is ad hoc signed and not notarized. Runtime updates use official installers or the user's version manager. Upstream documentation availability does not imply that a new offline snapshot has been published. Java/Erlang comparisons report their version precision explicitly. The other template refresh recipes have not yet been exercised against their upstream registries; Go refresh has been tested end to end.
