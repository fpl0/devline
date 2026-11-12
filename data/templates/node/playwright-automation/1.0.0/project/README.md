# Site Audit

A Playwright application that crawls a small website, checks content and browser behaviour, captures desktop/mobile screenshots, and writes a portable HTML/JSON report. It runs a complete local example without a network connection.

```sh
./devline.sh test
./devline.sh run
open reports/latest/index.html
```

The built-in workshop site includes a healthy page, an intentionally broken page, and a 404. The report shows findings and screenshots for all three. These deliberate findings are expected; they make the audit workflow visible.

## Audit your project

```sh
./devline.sh run --url http://127.0.0.1:3000 --output reports/my-app --max-pages 10
./devline.sh run --url http://127.0.0.1:3000 --fail-on-findings
```

The crawler follows same-origin links, excludes common downloadable/media paths, and stops at 10 pages by default (maximum 50). Third-party resources and service workers are blocked for a reproducible, bounded audit. Use `--timeout-ms` for slower pages. Exit **0** means a report was written, **1** means setup/input failed, and **2** means findings were present with `--fail-on-findings`. `report.truncated` records whether the page limit left links unvisited.

Checks cover HTTP failures, missing page titles/language/h1, missing image alt attributes, apparently unlabelled controls/buttons, duplicate IDs, uncaught JavaScript errors, and horizontal overflow at 1280/390 px. These are transparent heuristics, **not a complete accessibility or security assessment**. Screenshots may contain private page content: inspect reports before sharing.

## Structure and offline browser

`src/audit.mjs` owns the bounded crawl and report; `fixture.mjs` owns the demonstration website; `cli.mjs` owns options and exit policy. Tests drive the real bundled WebKit browser, check a form workflow, and assert the audit findings and generated artifacts. The browser archive is checksummed and extracted to a temporary directory for each command, then removed. Node dependencies and the Apple Silicon browser are included. This pinned WebKit requires macOS 26.5 or newer.

Use the same output directory to replace its current report; older screenshots from larger previous runs may remain but are not linked by the new report. Use a new output folder to retain separate audits. The HTML report references only local screenshots and opens directly from disk.
