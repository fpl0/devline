"""The final emitted pack must cover every selected publisher source."""

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


spec = importlib.util.spec_from_file_location("doc_audit", Path(__file__).with_name("check-doc-packs.py"))
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


class ContentInventoryTests(unittest.TestCase):
    def test_omitted_short_article_and_unrecorded_article_fail(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "CONTENT-INVENTORY.json").write_text(json.dumps({"documents": [
                {"source_path": "official/start.md", "document_id": "start"},
                {"source_path": "official/new-api.md", "document_id": "new-api"},
            ]}))
            errors = audit.check_inventory(root, "fixture@1", {"start", "unrecorded"})
            self.assertTrue(any("new-api: publisher article omitted" in error for error in errors))
            self.assertTrue(any("unrecorded: article missing" in error for error in errors))
            self.assertEqual(audit.check_inventory(root, "fixture@1", {"start", "new-api"}), [])

    def test_inventory_is_required_for_expanded_releases(self):
        with tempfile.TemporaryDirectory() as temp:
            self.assertIn("missing publisher content inventory",
                          audit.check_inventory(Path(temp), "fastapi@0.162.0+devline.2", set())[0])


if __name__ == "__main__":
    unittest.main()
