from pathlib import Path
import tempfile
import unittest
from fieldnotes.index import Index


class IndexTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.notes = self.root / "notes"
        self.notes.mkdir()
        self.index = Index(self.root / "search.db")

    def tearDown(self):
        self.temp.cleanup()

    def test_incremental_sync_ranking_update_delete_and_restart(self):
        first = self.notes / "first.md"
        first.write_text("# Distributed systems\nQueues and reliable delivery.\n")
        (self.notes / "second.txt").write_text(
            "Journal\nToday I studied distributed systems.\n"
        )
        self.assertEqual(self.index.sync(self.notes)["added"], 2)
        results = self.index.search("distributed systems")
        self.assertEqual(results[0]["path"], "first.md")
        self.assertEqual(self.index.sync(self.notes)["unchanged"], 2)
        first.write_text("# Databases\nTransactions and constraints.\n")
        self.assertEqual(self.index.sync(self.notes)["updated"], 1)
        self.assertEqual(len(self.index.search("distributed")), 1)
        (self.notes / "second.txt").unlink()
        self.assertEqual(self.index.sync(self.notes)["removed"], 1)
        reopened = Index(self.root / "search.db")
        self.assertEqual(reopened.search("transactions")[0]["path"], "first.md")
        self.assertEqual(reopened.search("distributed"), [])

    def test_unicode_literals_hidden_files_and_symlinks(self):
        (self.notes / "café.md").write_text("# Café\nReading with coffee.\n")
        (self.notes / ".secret.md").write_text("private coffee")
        (self.notes / "link.md").symlink_to(self.notes / "café.md")
        self.index.sync(self.notes)
        self.assertEqual(len(self.index.search('"café" OR')), 0)
        self.assertEqual(len(self.index.search("café")), 1)
        self.assertEqual(self.index.stats()[0]["documents"], 1)

    def test_invalid_and_oversize_documents_are_reported(self):
        (self.notes / "binary.txt").write_bytes(b"\xff\x00")
        (self.notes / "large.md").write_bytes(b"a" * (2 * 1024 * 1024 + 1))
        result = self.index.sync(self.notes)
        self.assertEqual(len(result["skipped"]), 2)
        self.assertEqual(result["added"], 0)
        with self.assertRaises(ValueError):
            self.index.search("*")
        with self.assertRaises(ValueError):
            self.index.document(999)
        with self.assertRaises(ValueError):
            self.index.search("hello", 0)

    def test_roots_are_independent(self):
        second = self.root / "other"
        second.mkdir()
        for folder in [self.notes, second]:
            (folder / "note.md").write_text("Shared title\nDifferent roots.\n")
            self.index.sync(folder)
        (self.notes / "note.md").unlink()
        self.index.sync(self.notes)
        self.assertEqual(self.index.stats()[0]["root"], str(second.resolve()))


if __name__ == "__main__":
    unittest.main()
