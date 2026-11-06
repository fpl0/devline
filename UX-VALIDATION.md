# Devline 0.3.0 validation

The offline reference checks below ran on Apple Silicon with macOS 27.2. The final installed 0.3.0 build passed a focused reader and navigation retest.

## Documentation and terminal checks

- The release catalog contains 15 docsets with 10,358 articles and 10 starter releases. The corpus audit parsed the generated HTML and counted 876 embedded images. It found no unsafe tags, attributes, or automatic resource URLs. A small set of upstream anchors still has no installed target. Some links with upstream query parameters open the official page.
- With network access denied, the bundled 0.3.0 CLI imported all 15 current source docsets into an empty library. Go and Python lookups returned declaration text, and saving and removing a reference persisted. A separate upgrade installed six updated docsets alongside their earlier revisions. Probes verified source examples, exact-version search, and offline links with anchors in Rails and Go.
- Pseudo-terminal checks opened `go:http.Handler` at its declaration, including via search selection and Saved and Recent. The FastAPI example rendered as formatted terminal text. Redirected output matched `--no-pager`. Empty `--ids` output exited with status 4. The bundled CLI reported 0.3.0 and the `devkit` alias remained in the bundle.
- The source passed 32 Rust tests, Clippy with warnings denied, 11 ingestion tests, and the Swift reference-navigation harness. The Swift harness covers back and forward, scroll-position restoration, search coalescing, and the history limit.

## Search measurements

Each result used 48 interleaved warm-cache lookups over six symbols on the same Mac. CLI timings include process startup and JSON output.

| Build and library | Median | 95th percentile |
| --- | ---: | ---: |
| 0.2.0 with the original library | 13.60 ms | 34.64 ms |
| 0.3.0 with a fresh library | 9.69 ms | 12.76 ms |
| 0.3.0 with retained versions | 9.65 ms | 14.12 ms |

These timings describe this machine and the tested libraries.

## Native checks

- `py:Path.read_text()` returned the pathlib method first and opened directly at its signature. The Contents shortcut opened a searchable list of 103 sections. Searching for `read_bytes` and pressing Return jumped to its signature. The saved entry appeared in the native shelf, and `devline docs saved --json` contained the exact `pathlib.Path.read_bytes` anchor.
- Following `os.path` opened the installed article from the same Python revision. Back restored the previous position, and Forward reopened the linked article. A link to the missing VS Code extension page presented **Page Not Installed**, with **Open in Browser** and **Cancel**.
- The 0.3.0 candidate's C# article list grew from 80 to 160 rows without a jump. After reading **A tour of C#**, switching to Go and going Back restored the 160-row list, selected article, and scroll fraction of 0.1601562. The C# Guides filter reported 104 results.
- FastAPI First Steps showed formatted code and a Tip callout in light mode. **Copy** placed the complete Python example on the clipboard. A diagram in Python’s documentation rendered from the installed article.
- **Install CLI** in Settings made `devline 0.3.0` available on the login shell’s existing PATH. Templates and Runtimes opened successfully.

[Python API](press/reference-python-dark.png) · [Contents search](press/reference-contents-dark.png) · [FastAPI guide](press/reference-fastapi-light.png) · [Embedded diagram](press/reference-images-light.png)

## Distribution

The final ZIP and DMG checksums and sizes are recorded in [validation.json](validation.json). The DMG was mounted read-only. The app passed strict signature verification, the executable was confirmed as arm64, and the image contained the Applications shortcut and installation guide.

The prior 0.2.0 release verified native starter selection, offline Go project creation, toolchain inspection, Updates, and automatic pagination. Distribution targets Apple Silicon with macOS 14 or later and uses ad hoc signing. Manual checks did not cover macOS 14 pagination, CLI-to-desktop link opening, or desktop refresh after a CLI save.
