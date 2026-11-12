"""Check complete web-reference ingestion and preservation of publisher teaching material."""

import importlib.util
import io
import json
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest


spec = importlib.util.spec_from_file_location("complete_web_ingest", Path(__file__).with_name("fetch-docs.py"))
ingest = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = ingest
spec.loader.exec_module(ingest)


def archive(root, name, files):
    with tarfile.open(root / name, "w:gz") as output:
        for path, value in files.items():
            content = value.encode() if isinstance(value, str) else value
            member = tarfile.TarInfo("upstream/" + path)
            member.size = len(content)
            output.addfile(member, io.BytesIO(content))


class CompleteWebDocumentationTests(unittest.TestCase):
    def test_react_imports_nested_reference_and_keeps_diagrams_and_sandbox_code(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive(root, "react-docs-2e6dca1.tar.gz", {
                "LICENSE-DOCS.md": "CC BY 4.0",
                "src/content/learn/index.md": "---\ntitle: Quick Start\n---\nThe complete official introduction.",
                "src/content/reference/react-dom/client/newMethod.md": '''---
title: <NewMethod>
---
<DeepDive>

A detailed publisher explanation.

<Diagram name="component_tree" alt="The publisher's component tree">

The publisher's caption.

</Diagram>

<Sandpack>

```js App.js
export default function App() { return <Example />; }
```

</Sandpack>

</DeepDive>
''',
                "public/images/docs/diagrams/component_tree.png": b"\x89PNG\r\n\x1a\nfixture",
                "src/content/blog/release.md": "A news post outside the reference scope.",
            })
            pack = ingest.react_pack(root)
            article = next(page for page in pack.markdown if page[0] == "reference/react-dom/client/newMethod")
            _, _, body, _, _, markup = pack.render_markdown(article)
            self.assertIn("<NewMethod>", body)
            self.assertIn("The publisher's caption.", body)
            self.assertIn("A detailed publisher explanation.", body)
            self.assertIn("export default function App() { return <Example />; }", body)
            self.assertIn("App.js", body)
            self.assertIn("data:image/png;base64,", markup)
            self.assertNotIn("<Sandpack", markup)
            inventory = json.loads(pack.notices["CONTENT-INVENTORY.json"])
            self.assertEqual(inventory["source_count"], 2)
            self.assertEqual(inventory["excluded_sources"], ["src/content/blog/release.md"])

    def test_typescript_includes_reference_modules_and_every_config_option(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            prefix = "packages/documentation/copy/en/"
            archive(root, "typescript-docs-a3ed0be.tar.gz", {
                "LICENSE": "CC BY 4.0",
                prefix + "reference/Utility Types.md": "---\ntitle: Utility Types\npermalink: /docs/handbook/utility-types.html\n---\nOfficial utility type explanations and examples.",
                prefix + "modules-reference/theory.md": "---\ntitle: Modules theory\npermalink: /docs/handbook/modules/theory.html\n---\nOfficial module resolution explanations and examples.",
                prefix + "modules-reference/diagrams/theory.md": "Diagram authoring source.",
                "packages/tsconfig-reference/copy/en/options/newOption.md": "---\ndisplay: New option\n---\nThe complete publisher explanation of a newly documented compiler option.",
            })
            pack = ingest.typescript_pack(root)
            self.assertEqual({page[0] for page in pack.markdown}, {
                "docs/handbook/utility-types.html", "docs/handbook/modules/theory.html", "tsconfig/options",
            })
            options = next(page for page in pack.markdown if page[0] == "tsconfig/options")
            _, _, body, _, _, markup = pack.render_markdown(options)
            self.assertIn('id="newOption"', markup)
            self.assertIn("complete publisher explanation", body)
            inventory = json.loads(pack.notices["CONTENT-INVENTORY.json"])
            self.assertEqual(inventory["source_count"], 3)
            self.assertEqual(len(inventory["excluded_sources"]), 1)

    def test_vite_preserves_new_api_guides_migration_badges_and_included_code(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive(root, "vite-v8.3.6.tar.gz", {
                "LICENSE": "MIT",
                "docs/guide/index.md": "# Getting started\n\nThe complete publisher introduction.",
                "docs/guide/api-environment-plugins.md": '# Environment API\n\n## [Compatibility](https://example.org) <Badge text="NRV" type="warning" />\n\nOfficial plugin guidance.\n\n<<< @/guide/example.yaml\n',
                "docs/guide/example.yaml": "name: publisher-workflow\non: push\n",
                "docs/config/ssr-options.md": "# SSR options\n\nThe complete publisher configuration reference.",
                "docs/changes/per-environment-apis.md": "# Environment changes\n\nThe complete publisher migration explanation.",
                "docs/blog/announcement.md": "An announcement outside the reference scope.",
            })
            pack = ingest.vite_pack(root)
            self.assertEqual(len(pack.markdown), 4)
            article = next(page for page in pack.markdown if page[0] == "guide/api-environment-plugins")
            _, _, body, _, _, markup = pack.render_markdown(article)
            self.assertIn("Compatibility NRV", body)
            self.assertIn("name: publisher-workflow\non: push", markup)
            self.assertIn('id="compatibility-nrv"', markup)
            self.assertNotIn("<<<", body)

    def test_typescript_cli_links_retain_publisher_named_row_anchors(self):
        source = '<p><a href="#strict">strict compiler mode</a></p>\n<table><tr name="strict"><td>Enable every strict type-checking option.</td></tr></table>'
        raw = ingest.prose_markdown(source, "typescript")
        body, markup = ingest.html_from_markdown(raw, "https://www.typescriptlang.org/docs/handbook/compiler-options.html", "typescript", {})
        self.assertIn('id="strict"', markup)
        self.assertIn('href="#strict"', markup)
        self.assertIn("Enable every strict type-checking option.", body)

    def test_react_controls_do_not_modify_fenced_jsx_examples(self):
        source = '```jsx\n<Diagram name="literal" />\n<CodeStep step={1}>value</CodeStep>\n```\n'
        self.assertEqual(ingest.prose_markdown(source, "react"), source)


if __name__ == "__main__":
    unittest.main()
