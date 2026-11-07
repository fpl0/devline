"""Regression checks for publisher content in the Tauri offline pack."""

import importlib.util
import json
from pathlib import Path
import re
import sys
import tempfile
import unittest
from unittest.mock import patch


spec = importlib.util.spec_from_file_location("desktop_docs", Path(__file__).with_name("fetch-desktop-docs.py"))
desktop = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = desktop
spec.loader.exec_module(desktop)


class TauriContentTests(unittest.TestCase):
    def test_tabs_keep_prose_lists_and_fenced_examples(self):
        raw = """<Tabs>
    <TabItem label = "Manual">

        Use this sequence.

        1. Run the command:

            ```sh frame=none
            printf '</TabItem>'
            ```

    </TabItem>
</Tabs>

An indented example:

    ```json
    {"local": true}
    ```
"""
        source = "https://v2.tauri.app/plugin/example/"
        markdown = desktop.tauri_markdown(raw, Path("example.mdx"), set(), {})
        body, html = desktop.tauri_html(markdown, source, None)
        self.assertIn("<p>Use this sequence.</p>", html)
        self.assertIn("<strong>Manual</strong>", html)
        self.assertIn("<ol>", html)
        self.assertIn("<pre><code>printf '&lt;/TabItem&gt;'</code></pre>", html)
        self.assertIn('<pre><code class="language-json">{"local": true}</code></pre>', html)
        self.assertNotIn("```", body)

    def test_plugin_platforms_and_notes_come_from_pinned_workspace(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            workspace = root / "tauri-plugins"
            plugin = workspace / "plugins/example"
            plugin.mkdir(parents=True)
            (workspace / "Cargo.toml").write_text('[workspace.package]\nrust-version = "1.90"\n')
            (plugin / "Cargo.toml").write_text("""[package]
name = "tauri-plugin-example"
rust-version = "1.92"
[package.metadata.platforms.support.macos]
level = "partial"
notes = "See [the limitation](https://example.com/note)."
[package.metadata.platforms.support.windows]
level = "none"
""")
            raw = "---\nplugin: example\n---\n<Compatibility plugin={frontmatter.plugin} />\n<TableCompatibility />"
            with patch.object(desktop, "root", root, create=True):
                tables = desktop.tauri_compatibility()
                rendered = desktop.tauri_markdown(raw, root / "example.mdx", set(), tables)
            self.assertEqual(rendered.count("Partial support"), 2)
            self.assertEqual(rendered.count("https://example.com/note"), 2)
            self.assertIn("at least **1.92**", rendered)
            self.assertIn("| example | 1.92 |", rendered)
            self.assertIn("| system-tray | 1.90 |", rendered)
            self.assertNotIn("<Compatibility", rendered)

    def test_schema_keeps_defaults_and_nested_constraints(self):
        property_schema = {
            "description": "Upstream option description.",
            "allOf": [{"anyOf": [{"$ref": "#/definitions/Mode"}, {"type": "null"}]}],
            "default": {"long": "x" * 220},
            "deprecated": True,
        }
        source = {
            "description": "Upstream configuration description.",
            "properties": {"option": property_schema},
            "definitions": {"Mode": {"enum": ["one", "two"], "description": "Upstream variants."}},
        }
        rendered = desktop.tauri_configuration(source)
        self.assertIn(property_schema["description"], rendered)
        self.assertIn("[`Mode`](#mode)", rendered)
        self.assertIn("## Mode {#mode}", rendered)
        blocks = [json.loads(code) for code in re.findall(r"```json\n(.*?)\n```", rendered, flags=re.S)]
        self.assertIn({name: value for name, value in property_schema.items() if name != "description"}, blocks)
        self.assertIn({"enum": ["one", "two"]}, blocks)


if __name__ == "__main__":
    unittest.main()
