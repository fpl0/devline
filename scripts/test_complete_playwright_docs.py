"""Completeness and publisher-language regressions for the Playwright importer."""

import importlib.util
import io
import json
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("fetch_complete_playwright", Path(__file__).with_name("fetch-docs.py"))
ingest = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = ingest
spec.loader.exec_module(ingest)


class CompletePlaywrightTests(unittest.TestCase):
    def test_source_inventory_includes_every_api_family_and_shared_js_guides(self):
        for path in (
            "docs/src/auth.md", "docs/src/test-sharding-js.md", "docs/src/chrome-extensions-js-python.md",
            "docs/src/test-api/class-test.md", "docs/src/test-reporter-api/class-reporter.md",
            "docs/src/electron-api/class-electron.md", "docs/src/mobile-api/class-android.md",
        ):
            self.assertTrue(ingest.playwright_source_page(path), path)
        for path in ("docs/src/api/params.md", "docs/src/test-assertions-csharp-java-python.md",
                     "docs/src/intro-python.md", "docs/src/images/example.md"):
            self.assertFalse(ingest.playwright_source_page(path), path)

    def test_language_overrides_keep_shared_method_options(self):
        raw = """# class: BrowserContext
* since: v1.8

## async method: BrowserContext.storageState
* since: v1.8
- returns: <[Object]>

Official shared explanation.

## async method: BrowserContext.storageState
* langs: csharp, java
- returns: <[string]>

### option: BrowserContext.storageState.indexedDB
- `indexedDB` ?<boolean>

Official shared option explanation.

## method: BrowserContext.onlyJava
* langs: java

Java-only method.

### param: BrowserContext.onlyJava.name
- `name` <[string]>

Java-only parameter.

## method: BrowserContext.close

```js
// * langs: python
await context.close();
```

```bash lang=python
python example.py
```
"""
        result = ingest.playwright_javascript(raw)
        self.assertIn("Official shared option explanation.", result)
        self.assertIn("await context.close();", result)
        self.assertIn("// * langs: python", result)
        for omitted in ("returns: <[string]>", "Java-only", "python example.py"):
            self.assertNotIn(omitted, result)

    def test_guide_language_filters_apply_to_subheadings(self):
        raw = """## Shared

Shared text.

## Python section
* langs: python

Not applicable.

### Nested Python section

Also not applicable.

## JavaScript section
* langs: js

Keep this.
"""
        result = ingest.playwright_javascript(raw)
        self.assertIn("Shared text.", result)
        self.assertIn("Keep this.", result)
        self.assertNotIn("applicable", result)

    def test_empty_heading_language_marker_does_not_remove_shared_guide_prose(self):
        raw = """### Google Chrome & Microsoft Edge

Keep the publisher's shared browser guidance.

######
* langs: python

Python-only library instructions.

#### Installing browsers

Keep the shared installation guidance.
"""
        result = ingest.playwright_javascript(raw)
        self.assertIn("Keep the publisher's shared browser guidance.", result)
        self.assertIn("Keep the shared installation guidance.", result)
        self.assertNotIn("Python-only", result)

    def test_versioned_inline_templates_preserve_js_options_and_references(self):
        params = ingest.playwright_parameters("""## options-list-v1.8
- %%-js-option-%%
- %%-java-option-%%

## js-option
* langs: js
- `timeout` <[float]>

Exact publisher option prose with [`method: Test.setTimeout`].

## java-option
* langs: java
- `other` <[string]>

Java-only prose.
""")
        result = ingest.playwright_expansion(
            "### option: Page.goto.-inline- = %%-options-list-v1.8-%%\n* since: v1.8\n", params,
            {"Test": "api/class-test"})
        self.assertIn("### option: Page.goto.timeout", result)
        self.assertIn("Exact publisher option prose", result)
        self.assertIn("https://playwright.dev/docs/api/class-test#test-set-timeout", result)
        self.assertNotIn("Java-only", result)
        self.assertNotIn("%%", result)
        self.assertEqual(ingest.playwright_expansion("[`param: handler`]", params, {}), "`handler`")
        with self.assertRaisesRegex(RuntimeError, "Missing Playwright parameter"):
            ingest.playwright_expansion("%%-absent-%%", params, {})

    def test_nested_api_fields_get_real_destination_anchors(self):
        raw = """### option: Browser.newContext.recordHar
- `recordHar` <[Object]>
  - `path` <[string]> Exact publisher field text.
"""
        result = ingest.playwright_property_anchors(raw)
        self.assertIn('id="browser-new-context-option-record-har-path"', result)
        self.assertIn('id="browser-new-context-record-har-option-path"', result)
        self.assertIn("Exact publisher field text.", result)
        self.assertEqual(ingest.playwright_anchor("event", "Page.console"), "page-event-console")
        self.assertEqual(ingest.playwright_anchor("method", "Test.(call)"), "test-call")

    def test_pack_imports_test_runner_reporter_authentication_and_source_inventory(self):
        files = {
            "LICENSE": "Official publisher license fixture.",
            "docs/src/api/params.md": "## timeout\n- `timeout` <[float]>\n\nOfficial timeout text.\n",
            "docs/src/intro-js.md": "---\nid: intro\ntitle: Getting started\n---\n\nOfficial getting started text with sufficient content.\n",
            "docs/src/auth.md": "---\nid: auth\ntitle: Authentication\n---\n\nOfficial authentication text with sufficient content.\n",
            "docs/src/test-api/class-test.md": "# class: Test\n* langs: js\n\nOfficial test runner explanation.\n\n## method: Test.(call)\n\nPublisher test declaration.\n",
            "docs/src/test-reporter-api/class-reporter.md": "# class: Reporter\n* langs: js\n\nOfficial reporter explanation.\n\n## optional method: Reporter.onBegin\n\nPublisher reporter callback.\n",
            "docs/src/api/class-requestoptions.md": "# class: RequestOptions\n* langs: java\n\nJava-only official class.\n",
        }
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with tarfile.open(root / "playwright-v1.65.0.tar.gz", "w:gz") as archive:
                for key, text in files.items():
                    payload = text.encode()
                    entry = tarfile.TarInfo("playwright-fixture/" + key)
                    entry.size = len(payload)
                    archive.addfile(entry, io.BytesIO(payload))
            pack = ingest.playwright_pack(root)
        pages = {page[0]: page for page in pack.markdown}
        self.assertIn("auth.md", pages)
        self.assertIn("test-api/class-test.md", pages)
        self.assertIn("test-reporter-api/class-reporter.md", pages)
        self.assertNotIn("api/class-requestoptions.md", pages)
        self.assertEqual(pages["test-api/class-test.md"][3], "https://playwright.dev/docs/api/class-test")
        self.assertIn("{#reporter-on-begin}", pages["test-reporter-api/class-reporter.md"][2])
        inventory = json.loads(pack.notices["CONTENT-INVENTORY.json"])
        self.assertEqual(len(inventory["included_pages"]), 4)
        self.assertEqual(inventory["omitted_non_javascript_api_pages"], ["docs/src/api/class-requestoptions.md"])
        self.assertTrue(all(len(page["sha256"]) == 64 for page in inventory["included_pages"]))


if __name__ == "__main__":
    unittest.main()
