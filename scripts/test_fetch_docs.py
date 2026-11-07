"""Regression checks for semantic, safe, offline reference ingestion."""

import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch


spec = importlib.util.spec_from_file_location("fetch_docs", Path(__file__).with_name("fetch-docs.py"))
ingest = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = ingest
spec.loader.exec_module(ingest)


class ReferenceIngestionTests(unittest.TestCase):
    def test_infrastructure_links_keep_upstream_github_filenames(self):
        base = "https://github.com/example/service/blob/v1/docs/setup.md"
        target = ingest.link_destination("./backup.md#restore", base, "infrastructure", {})
        self.assertEqual(target, "https://github.com/example/service/blob/v1/docs/backup.md#restore")

    def test_empty_and_nested_examples_keep_valid_offline_markup(self):
        upstream = ('<main><pre><code></code></pre><pre id="kept"><code> </code></pre>'
                    '<pre><details><summary>Example note</summary><pre><code>call()</code></pre></details></pre></main>')
        _, node, _ = ingest.article_from_html(upstream, "main", "Fixture")
        body, markup = ingest.prepared_html(node, "https://example.org/guide", "java")
        self.assertEqual(markup.count("<pre"), 1)
        self.assertIn('<span id="kept"></span>', markup)
        self.assertIn("<pre><code>call()</code></pre>", markup)
        self.assertIn("Example note", body)

    def test_html_keeps_code_table_anchors_and_safe_links(self):
        upstream = """<main><h1 id="topic" onclick="bad()">Topic</h1>
        <aside class="admonition note"><p>Useful note</p></aside>
        <dl><dt id="api.Test"><code>Test</code></dt><dd>Definition</dd></dl>
        <pre><code>if x &lt; y &amp;&amp; a &gt; b:
    print(x)

    print(y)</code></pre>
        <table><tr><td>Mode</td><td>Value</td></tr></table>
        <a href="#api.Test">Jump</a><a href="../reference/method.html#method">Read more</a>
        <a href="javascript:alert(1)">Unsafe</a><a href="https://name:secret@example.org/">Credentials</a>
        <img src="remote.png" alt="Diagram"><script>alert('unsafe')</script><nav>Site chrome</nav></main>"""
        _, node, _ = ingest.article_from_html(upstream, "main", "Fixture")
        body, markup = ingest.prepared_html(node, "https://example.org/guide/topic.html", "fixture")
        for fragment in ('<h1 id="topic">', '<dt id="api.Test">', "<table>", "Useful note",
                         "if x &lt; y &amp;&amp; a &gt; b:", "print(x)\n\n    print(y)",
                         'href="#api.Test"', 'href="https://example.org/reference/method.html#method"',
                         '<a href="https://example.org/guide/remote.png">Diagram</a>'):
            self.assertIn(fragment, markup)
        for fragment in ("onclick=", "javascript:", "secret@", "<script>", "Site chrome", "<img"):
            self.assertNotIn(fragment, markup)
        self.assertIn("if x < y && a > b:", body)

    def test_only_safe_local_images_are_embedded(self):
        self.assertTrue(ingest.image_url(b'<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0"/></svg>')
                        .startswith("data:image/svg+xml;base64,"))
        self.assertIsNone(ingest.image_url(b'<svg xmlns="http://www.w3.org/2000/svg"><script>bad()</script></svg>'))
        self.assertIsNone(ingest.image_url(b'<svg xmlns="http://www.w3.org/2000/svg" style="fill:url(https://x.example)"/>'))
        self.assertIsNone(ingest.image_url(b'<!DOCTYPE svg [<!ENTITY x SYSTEM "file:///etc/passwd">]><svg>&x;</svg>'))
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "pages").mkdir()
            (root / "assets").mkdir()
            (root / "assets/diagram.svg").write_text('<svg xmlns="http://www.w3.org/2000/svg"/>')
            load = ingest.file_assets(root / "pages/index.md", root)
            self.assertIsNotNone(load("../assets/diagram.svg"))
            self.assertIsNone(load("https://example.org/assets/diagram.svg"))
            self.assertIsNone(load("../../outside.svg"))
        self.assertIsNone(ingest.asset_path("//example.org/diagram.svg"))

    def test_fastapi_examples_and_nested_material_directives(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "docs_src").mkdir()
            (root / "docs_src/example.py").write_text('print("snippet")\n')
            missing = set()
            raw = ingest.include_fastapi_examples(
                "//// tab | Python\n\n/// tip | Try it\n\n{* ../../docs_src/example.py hl[1] *}\n\n///\n\n////\n",
                root, "fixture.md", missing,
            )
            result = ingest.material_blocks(raw)
            self.assertFalse(missing)
            self.assertIn("**Python**", result)
            self.assertIn("**Try it**", result)
            self.assertIn('print("snippet")', result)
            self.assertNotIn("////", result)

    def test_percent_encoded_upstream_operator_ids_match_browser_fragments(self):
        raw = '<main><section id="%7C%3E/2"><h2>Pipe</h2><a href="#%7C%3E/2">Pipe</a></section></main>'
        _, node, aliases = ingest.article_from_html(raw, "main", "Kernel")
        _, markup = ingest.prepared_html(node, "https://hexdocs.pm/elixir/Kernel.html", "elixir")
        self.assertEqual(aliases, ["|>/2"])
        self.assertIn('id="|&gt;/2"', markup)
        self.assertIn('href="#%7C%3E/2"', markup)

    def test_source_categories_separate_guides_api_and_compatibility(self):
        for pack, key, expected in (
            ("python", "library/pathlib.html", "api"),
            ("python", "tutorial/classes.html", "guide"),
            ("csharp", "core/compatibility/core-libraries/10.0/time.md", "compatibility"),
            ("csharp", "csharp/language-reference/statements/selection.md", "reference"),
            ("fastapi", "reference/dependencies.md", "api"),
            ("aspnet", "tutorials/first-web-api.md", "guide"),
            ("elixir", "Kernel.html", "api"),
        ):
            self.assertEqual(ingest.document_category(pack, key), expected)

    def test_docfx_resolves_local_and_nested_examples_and_selects_moniker(self):
        with tempfile.TemporaryDirectory() as temp:
            repo = Path(temp)
            base = repo / "docs"
            base.mkdir()
            (repo / "includes").mkdir()
            (repo / "samples").mkdir()
            (repo / "includes/note.md").write_text(
                ':::code source="../samples/Example.cs" id="snippet_Hello" language="csharp":::\n'
            )
            (repo / "samples/Example.cs").write_text(
                '// <snippet_Hello>\nConsole.WriteLine("hello");\n// </snippet_Hello>\n'
            )
            pack = ingest.Pack("csharp", "C#", "dotnet", "1", "", "https://learn.microsoft.com/en-us/dotnet/", "", "")
            raw = """:::moniker range=">= net-10.0 < net-11.0"
[!INCLUDE [Note](../includes/note.md)]
Use <xref:System.String>.
:::moniker-end
:::moniker range="= net-8.0"
Older instructions
:::moniker-end
[!INCLUDE [Missing](missing.md)]
"""
            result = ingest.docfx_includes(raw, base / "start.md", base, repo, pack)
            self.assertIn('Console.WriteLine("hello");', result)
            self.assertIn('[`System.String`](xref:System.String)', result)
            self.assertNotIn("Older instructions", result)
            self.assertIn("Included content was unavailable", result)
            self.assertIn("https://github.com/dotnet/docs/blob/", result)
            self.assertEqual(pack.missing, {"docs/start.md: file absent: missing.md"})

    def test_docfx_paths_and_regions_match_original_sample_formats(self):
        self.assertEqual(ingest.docfx_target('samples/Example.cs#1 "Caption"'), "samples/Example.cs#1")
        self.assertEqual(ingest.docfx_target(r"~\samples\Example.cs"), "~/samples/Example.cs")
        for opening, closing in (
            ("//< Sample>", "// </  Sample>"),
            ("<!-- <snippet_Sample> -->", "<!-- </snippet_Sample> -->"),
            ("@* <Sample> *@", "@* </Sample> *@"),
            ('#Region "Sample"', "#End Region"),
        ):
            self.assertEqual(ingest.docfx_region(f"{opening}\nexample\n{closing}\n", "Sample"), "example\n")
        self.assertEqual(
            ingest.docfx_region("#region Sample\nfirst\n#region nested\ninside\n#endregion\nlast\n#endregion\n", "Sample"),
            "first\n#region nested\ninside\n#endregion\nlast\n",
        )

    def test_docfx_preserves_quoted_examples_and_spacing_variants(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            (base / "Example.cs").write_text("// <Example>\nUseSample();\n// </Example>\n")
            pack = ingest.Pack("aspnet", "ASP.NET", "dotnet", "1", "", "https://example.org/", "", "")
            raw = (
                '> :::code source="Example.cs" id ="Example" language="csharp":::\n'
                '> :::image type="content" source="diagram.png" alt-text="Diagram"\n'
                'Use <xref:System.String > and :::no-loc text=""word"":::.\n'
                ':::zone target="docs" pivot="visualstudio"\n'
                'In Visual Studio, create a project.\n'
                ':::zone-end\n'
            )
            rendered = ingest.docfx_includes(raw, base / "article.md", base, base, pack)
            self.assertIn("> UseSample();", rendered)
            self.assertIn('> <img src="diagram.png" alt="Diagram">', rendered)
            self.assertIn('[`System.String`](xref:System.String) and "word".', rendered)
            self.assertIn("**Visualstudio**", rendered)
            self.assertNotIn(":::zone", rendered)
            self.assertFalse(pack.missing)

    def test_inert_site_navigation_retains_heading_text(self):
        for pack, link in (("python", '<a class="toc-backref" href="#id1">'), ("csharp", '<a href="#tab/net-cli">')):
            parser = ingest.DocumentParser()
            parser.feed(f'<h2 id="keep">{link}Install with CLI</a></h2>')
            _, markup = ingest.prepared_html(parser.root, "https://example.org/setup", pack)
            self.assertEqual(markup, '<h2 id="keep">Install with CLI</h2>')

    def test_reference_anchors_match_the_upstream_rails_ruby_and_go_syntax(self):
        rails = ingest.DocumentParser()
        rails.feed(
            '<h2 id="one">One</h2><h3 id="first">map! and []?</h3>'
            '<h2 id="two">Two</h2><h3 id="second">map! and []?</h3>'
        )
        ingest.rails_headings(rails.root)
        _, markup = ingest.prepared_html(rails.root, "https://example.org/rails", "rails")
        self.assertIn('id="one-map-bang-and-squarebrackets-questionmark"', markup)
        self.assertIn('id="two-map-bang-and-squarebrackets-questionmark"', markup)
        self.assertIn('id="second"', markup)
        ruby = ingest.DocumentParser()
        ruby.feed('<h2 id="rescued-exceptions">Rescued Exceptions</h2><a href="#label-Rescued+Exceptions">See it</a>')
        _, markup = ingest.prepared_html(ruby.root, "https://example.org/ruby", "ruby")
        self.assertIn('href="#rescued-exceptions"', markup)
        go = ingest.DocumentParser()
        go.feed('<pre class="ebnf">MethodName = identifier .</pre>')
        ingest.go_grammar_anchors(go.root)
        _, markup = ingest.prepared_html(go.root, "https://go.dev/ref/spec", "go")
        self.assertIn('<span id="MethodName">MethodName</span> = identifier .', markup)

    def test_shards_are_bounded_and_do_not_split_articles(self):
        documents = [{"id": str(i), "body": "A" * 20} for i in range(5)]
        with tempfile.TemporaryDirectory() as temp, patch.object(ingest, "SHARD_BYTES", 110):
            path = Path(temp)
            shards, size = ingest.write_shards(path, documents)
            self.assertEqual(shards, 3)
            names = ["documents.json", "documents-2.json", "documents-3.json"]
            self.assertEqual(size, sum((path / name).stat().st_size for name in names))
            self.assertEqual(documents, [row for name in names for row in json.loads((path / name).read_text())])


if __name__ == "__main__":
    unittest.main()
