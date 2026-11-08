"""Exercise client pack verification through the packaging CLI."""

from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
import unittest


SCRIPT = Path(__file__).with_name("seal-service-client-packs.py")


class ClientPackSealTests(unittest.TestCase):
    def setUp(self):
        scratch = self.enterContext(TemporaryDirectory())
        self.data = Path(scratch)
        self.clients = self.data / "service-clients"
        self.pack = self.clients / "artifacts" / "fixture"
        self.pack.mkdir(parents=True)
        (self.clients / "fixture.json").write_text("{}")
        (self.pack / "ready.json").write_text('{"schema_version":1,"stack":"fixture"}')
        self.artifact = self.pack / "client.zip"
        self.artifact.write_bytes(b"original")

    def run_script(self, *args):
        return subprocess.run([sys.executable, "-O", str(SCRIPT), "--data", str(self.data), *args],
                              capture_output=True, text=True)

    def test_optimized_check_rejects_same_size_changed_artifact(self):
        sealed = self.run_script("fixture")
        self.assertEqual(sealed.returncode, 0, sealed.stderr)
        valid = self.run_script("--check", "fixture")
        self.assertEqual(valid.returncode, 0, valid.stderr)
        self.artifact.write_bytes(b"tampered")
        changed = self.run_script("--check", "fixture")
        self.assertEqual(changed.returncode, 1)
        self.assertIn("fixture seal changed", changed.stderr)

    def test_unknown_and_missing_packs_fail_verification(self):
        unknown = self.run_script("--check", "missing")
        self.assertEqual(unknown.returncode, 1)
        self.assertIn("Unknown client pack stacks: missing", unknown.stderr)
        (self.clients / "missing.json").write_text("{}")
        missing = self.run_script("--check", "missing")
        self.assertEqual(missing.returncode, 1)
        self.assertIn("No completed proof marker", missing.stderr)


if __name__ == "__main__":
    unittest.main()
