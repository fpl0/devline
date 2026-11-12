"""Checks that runtime documentation follows publisher inventories without sampling."""

import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch


spec = importlib.util.spec_from_file_location("runtime_ingest", Path(__file__).with_name("fetch-docs.py"))
ingest = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = ingest
spec.loader.exec_module(ingest)


def node_html(body):
    return (f"<html><head><title>Node.js v{ingest.NODE_VERSION} Documentation</title></head>"
            '<body><nav><a href="not-in-the-manual.html">Site navigation</a></nav>'
            f'<div role="main" id="apicontent">{body}</div></body></html>')


class CompleteRuntimeDocumentationTests(unittest.TestCase):
    def test_go_inventory_keeps_new_public_packages_and_excludes_implementation_details(self):
        with patch.object(ingest, "command", return_value="fmt\ninternal/fmtsort\nmath/big\nnew/publicapi\n"
                          "vendor/golang.org/x/net/http2\ncrypto/internal/fips140\n"):
            self.assertEqual(ingest.go_public_packages("go", Path("/go"), {}),
                             ["fmt", "math/big", "new/publicapi"])

    def test_go_examples_use_build_selected_files_and_preserve_publisher_source(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            folder = root / "src/fmt"
            folder.mkdir(parents=True)
            source = '// Publisher copyright\npackage fmt_test\nfunc ExamplePrint() { Print("sample") }\n'
            (folder / "example_test.go").write_text(source)
            (folder / "example_linux_test.go").write_text(source)
            (folder / "format_test.go").write_text("package fmt_test\n")
            with patch.object(ingest, "command", return_value="example_test.go format_test.go\n"):
                files = list(ingest.go_example_sources("go", root, "fmt", {}))
            self.assertEqual(files, [folder / "example_test.go"])
            self.assertEqual(files[0].read_text(), source)

    def test_node_inventory_uses_article_not_sidebar_or_other_releases(self):
        raw = node_html('<a href="webstreams.html">Web streams</a><a href="errors.html#errors">Errors</a>'
                        '<a href="https://nodejs.org/docs/latest/api/new.html">Other release</a>'
                        '<a href="all.html">All articles</a><a href="errors.html">Again</a>')
        self.assertEqual(ingest.node_index_pages(raw), ["errors", "index", "webstreams"])
        with self.assertRaisesRegex(RuntimeError, "does not match"):
            ingest.node_index_pages(raw.replace(ingest.NODE_VERSION, "0.0.0"))

    def test_node_missing_download_inventory_fails_instead_of_silently_omitting_api(self):
        with tempfile.TemporaryDirectory() as temp:
            cache = Path(temp)
            (cache / f"node{ingest.NODE_VERSION}-index.html").write_text(
                node_html('<a href="new-api.html">Newly published API</a>'))
            with patch.object(ingest, "NODE_PAGES", ["index"]):
                with self.assertRaisesRegex(RuntimeError, "new-api"):
                    ingest.node_pack(cache)

    def test_node_full_pages_keep_examples_and_record_inventory(self):
        with tempfile.TemporaryDirectory() as temp:
            cache = Path(temp)
            (cache / f"node{ingest.NODE_VERSION}-LICENSE").write_text("Publisher license")
            (cache / f"node{ingest.NODE_VERSION}-index.html").write_text(node_html(
                '<h1>Official API contents</h1><p>Every API article is linked from these official contents.</p>'
                '<a href="webstreams.html">Web Streams API</a>'))
            (cache / f"node{ingest.NODE_VERSION}-webstreams.html").write_text(node_html(
                '<h1 id="webstreams">Web Streams API</h1><p>The original publisher explanation survives.</p>'
                '<pre><code class="language-js">const stream = new ReadableStream();\n</code></pre>'))
            with patch.object(ingest, "NODE_PAGES", ["index", "webstreams"]):
                pack = ingest.node_pack(cache)
            docs = {doc["id"]: doc for doc in pack.documents}
            self.assertEqual(set(docs), {"index.html", "webstreams.html"})
            self.assertIn("const stream = new ReadableStream();", docs["webstreams.html"]["html"])
            inventory = json.loads(pack.notices["CONTENT-INVENTORY.json"])
            self.assertEqual(inventory["eligible_pages"], inventory["imported_documents"])
            self.assertEqual(pack.entry_id, "index.html")


if __name__ == "__main__":
    unittest.main()
