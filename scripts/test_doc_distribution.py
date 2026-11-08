"""Regression tests for deterministic docset releases and the corpus audit."""

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


def script(name):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


releases = script("doc-release")
audit = script("check-doc-packs")
ASSET_BASE = "https://github.com/example/devline/releases/download/docsets-2026.11.07"


def fixture(data, markup, *, pack_id="fixture", version="1.0.0+devline.1"):
    pack = {
        "id": pack_id, "version": version, "name": "Fixture",
        "path": f"docs/{pack_id}/{version}", "upstream_version": "1.0.0",
        "snapshot_date": "2026-11-07", "coverage": ["Fixture"], "omissions": ["Other pages"],
    }
    source = data / pack["path"]
    source.mkdir(parents=True, exist_ok=True)
    (data / "docs-catalog.json").write_text(json.dumps([pack]))
    (source / "documents.json").write_text(json.dumps([{
        "id": "article", "title": "An example", "body": "An example of code",
        "source_url": "https://example.org/article", "category": "guide", "html": markup,
    }]))
    (source / "SOURCES.md").write_text("Official fixture documentation")
    return source


class DocDistributionTests(unittest.TestCase):
    def test_reproducible_assets_and_immutable_prior_manifest(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            data, first, second = root / "data", root / "first", root / "second"
            fixture(data, "<p>Test</p>")
            prior = releases.build(data, first, ASSET_BASE)
            produced = releases.build(data, second, ASSET_BASE, previous=prior)
            self.assertEqual(prior.read_bytes(), produced.read_bytes())
            releases.verify(first)
            releases.verify(second)
            file = next(first.glob("*.tar.gz"))
            original = file.read_bytes()
            fixture(data, "<p>Changed without revision</p>")
            with self.assertRaisesRegex(ValueError, "Immutable"):
                releases.build(data, first, ASSET_BASE, previous=prior)
            self.assertEqual(file.read_bytes(), original)

    def test_channel_refresh_drops_retired_mixed_infrastructure_release(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            data, historical, current = root / "data", root / "historical", root / "current"
            fixture(data, "<p>Historical help</p>",
                    pack_id="infrastructure", version="2026.11.07+devline.1")
            catalog = data / "docs-catalog.json"
            retired = json.loads(catalog.read_text())
            with patch.object(releases, "RETIRED_FROM_CHANNEL", set()):
                previous = releases.build(data, historical, ASSET_BASE)
            historical_bytes = previous.read_bytes()

            fixture(data, "<p>Publisher reference</p>",
                    pack_id="infrastructure", version="2026.11.07+devline.2")
            catalog.write_text(json.dumps(retired + json.loads(catalog.read_text())))
            manifest = releases.build(data, current, ASSET_BASE, previous=previous)
            versions = [release["pack"]["version"] for release in json.loads(manifest.read_text())["releases"]]
            self.assertEqual(versions, ["2026.11.07+devline.2"])
            self.assertEqual(previous.read_bytes(), historical_bytes)
            releases.verify(historical)
            releases.verify(current)

    def test_new_asset_cannot_shadow_a_prior_release(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            data, historical, current = root / "data", root / "historical", root / "current"
            fixture(data, "<p>Original</p>", pack_id="fixture", version="a-b")
            previous = releases.build(data, historical, ASSET_BASE)
            fixture(data, "<p>New</p>", pack_id="fixture-a", version="b")
            with self.assertRaisesRegex(ValueError, "same asset name"):
                releases.build(data, current, ASSET_BASE, previous=previous)
            self.assertFalse(current.exists())

    def test_rejects_links_and_mismatched_asset_bytes(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            data, output = root / "data", root / "assets"
            source = fixture(data, "<p>Test</p>")
            link = source / "bad"
            link.symlink_to(root)
            with self.assertRaisesRegex(ValueError, "Links are unsupported"):
                releases.build(data, output, ASSET_BASE)
            link.unlink()
            releases.build(data, output, ASSET_BASE)
            next(output.glob("*.tar.gz")).write_bytes(b"tampered")
            with self.assertRaisesRegex(ValueError, "checksum or size"):
                releases.verify(output)

    def test_local_sdk_content_cannot_be_published_even_with_forged_flag(self):
        for pack_id, local_only in (("swiftui", False), ("fixture", True)):
            with self.subTest(pack_id=pack_id), tempfile.TemporaryDirectory() as scratch:
                root = Path(scratch)
                data, output = root / "data", root / "assets"
                fixture(data, "<p>Private SDK material</p>")
                catalog = json.loads((data / "docs-catalog.json").read_text())
                catalog[0].update(id=pack_id, local_only=local_only)
                (data / "docs-catalog.json").write_text(json.dumps(catalog))
                with self.assertRaisesRegex(ValueError, "Local SDK documentation"):
                    releases.build(data, output, ASSET_BASE)
                self.assertFalse(list(output.glob("*.tar.gz")))

    def test_rejects_symlinked_catalog_pack_parent(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            data, output = root / "data", root / "assets"
            source = fixture(data, "<p>Device-only content</p>")
            cache = data / "device-cache"
            source.parent.rename(cache)
            source.parent.symlink_to(cache, target_is_directory=True)
            with self.assertRaisesRegex(ValueError, "Links are unsupported in docset paths"):
                releases.build(data, output, ASSET_BASE)
            self.assertFalse(list(output.glob("*.tar.gz")))

    def test_audit_counts_examples_and_regressions_beyond_baseline(self):
        with tempfile.TemporaryDirectory() as scratch:
            data = Path(scratch)
            fixture(data, '<h1 id="api.read">Read</h1><a href="#api.read">Read</a>'
                    '<a href="#missing">Missing</a><a href="?view=older#old">Older release</a>'
                    '<pre><code>read()</code></pre>'
                    '<img src="https://example.org/img.png"><script>alert(1)</script>')
            report = audit.audit(data, require_examples=False)
            self.assertEqual(report["counts"]["examples"], 1)
            self.assertEqual(report["counts"]["missing_same_page_anchors"], 1)
            self.assertEqual(report["counts"]["links_outside_snapshot"], 1)
            baseline = {"missing_anchors": [["fixture@1.0.0+devline.1", "article", "example.org/article", "missing"]]}
            errors, anchors = audit.regressions(report, baseline)
            self.assertEqual(anchors, [])
            self.assertTrue(any("non-offline image" in error for error in errors))
            self.assertTrue(any("active markup" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
