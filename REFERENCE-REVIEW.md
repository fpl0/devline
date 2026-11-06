# Devline reference experience review

Reviewed on 6 November 2026 against the installed native macOS application. This is a focused functional and visual review, not an accessibility conformance certification.

## Journey and changes

1. **Find a reference — improved.** The previous `Path.read_text` search ranked a FastAPI guide above Python's API and rejected call syntax. Search now supports qualified symbols, suffixes, parentheses, docset prefixes, explicit versions, and category browsing. The result identifies the matched symbol and its article. Latest installed content is the default; older revisions remain available.
2. **Read and navigate — improved.** Flattened content and artificial article chunks lost code, structure, and context. The library now has 10,358 complete articles with semantic HTML, 876 embedded images, code-copy controls, searchable contents, readable tables, adjustable text, and local cross-reference navigation. The two panes scroll independently. Pagination appends entries, while Back restores the previous article, query, scope, selection, and reading position.
3. **Return to useful material — improved.** Saved references and recent articles are shared between the app and CLI. A saved reference carries its exact docset revision and section. Uninstalled versions remain identifiable in Saved. Reference links open the same article in Devline.
4. **Maintain an offline workspace — improved.** Documentation revisions are visible as updates and install alongside older versions. The native app, terminal, templates, runtime checks, and update discovery continue to share the Rust core. The CLI now has readable interactive output, anchored paging, explicit raw/JSON output, selection, and shell completion.

## Product ideas adapted

- [Dash's guide](https://kapeli.com/dash_guide): docset scopes, keyboard navigation, table of contents, and reference URLs.
- [DevDocs help](https://devdocs.io/help) and [scraper architecture](https://github.com/freeCodeCamp/devdocs#scraper): consistent offline reading and normalized documentation links.
- [Zeal](https://github.com/zealdocs/zeal#how-to-use): concise docset-prefix searches from the keyboard.
- [Apple Dictionary](https://support.apple.com/en-ie/guide/dictionary/dic34880/mac): a restrained native window, linked lookup, and Back/Forward navigation.
- [tealdeer](https://docs.tealdeer.org/latest/usage.html), [bat](https://github.com/sharkdp/bat/blob/master/README.md#using-less-as-a-pager), and [fzf](https://github.com/junegunn/fzf#preview-window): terminal-first reading, pager behavior, and composable output.

These are interaction references. Devline keeps its own implementation and visual identity.

## Evidence and limits

See [UX-VALIDATION.md](UX-VALIDATION.md) and [validation.json](validation.json) for exact checks, artifact status, and benchmark conditions. Screenshots in `press/` show the actual installed app.

The content audit found no unsafe markup, automatically loaded network resources, private filesystem paths, or unresolved source directives. All required source and attribution notices are included, including Java's corresponding source archive.

The library is curated, not a mirror of every upstream website. Ten stale same-page links remain; some upstream cross-page fragments and version-specific links have no matching local target. Pages outside the installed library require an explicit browser handoff. Search timings are local warm measurements on one machine, not a universal latency guarantee. Intel hardware, VoiceOver conformance, and every supported macOS version were not exercised in this review. The current public build uses an ad-hoc signature, not Developer ID notarization.
