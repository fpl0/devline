"""Exercise a DevDocs checkout and its archives through Devline's real library.

Set DEVLINE_TEST_CORE to a cdylib built from devline_core to also exercise
the same link resolver used by the Mac reader.
"""

import ctypes
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
DEVLINE = Path(os.environ.get("DEVLINE_TEST_BINARY", ROOT / "core/target/debug/devline")).resolve()


def write_json(path, data):
    path.write_text(json.dumps(data), encoding="utf-8")


def archive_files(path, entries):
    with tarfile.open(path, "w:gz") as archive:
        for name, data in entries.items():
            payload = json.dumps(data).encode("utf-8")
            member = tarfile.TarInfo(name)
            member.size = len(payload)
            archive.addfile(member, io.BytesIO(payload))


def core_request(library, payload):
    core = ctypes.CDLL(library)
    core.devline_request.argtypes = [ctypes.c_char_p]
    core.devline_request.restype = ctypes.c_void_p
    core.devline_free.argtypes = [ctypes.c_void_p]
    core.devline_free.restype = None
    response = core.devline_request(json.dumps(payload).encode("utf-8"))
    try:
        return json.loads(ctypes.string_at(response))
    finally:
        core.devline_free(response)


class DevDocsImportTests(unittest.TestCase):
    def run_command(self, *args, success=True):
        result = subprocess.run(args, capture_output=True, text=True, timeout=60)
        if success:
            self.assertEqual(result.returncode, 0, f"{result.stdout}\n{result.stderr}")
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout)
        return result.stdout

    def test_offline_import_search_link_and_private_versions(self):
        with tempfile.TemporaryDirectory(prefix="devline-test-devdocs-") as temporary:
            root = Path(temporary)
            library = root / "library"
            source = root / "docs"
            astro = source / "astro"
            astro.mkdir(parents=True)
            astro_meta = {
                "name": "Astro", "slug": "astro", "release": "5.16.5", "type": "simple",
                "links": {"home": "https://docs.astro.build/"},
            }
            go_meta = {
                "name": "Go", "slug": "go~1.27", "release": "1.27.3", "version": "1.27",
                "type": "go", "links": {"home": "https://go.dev/"},
            }
            write_json(source / "docs.json", [astro_meta, go_meta])
            write_json(astro / "meta.json", astro_meta)
            write_json(astro / "index.json", {"entries": [
                {"name": "AstroActionFixture()", "path": "guides/actions/index#server", "type": "Actions"},
                {"name": "Options", "path": "reference/options/index", "type": "Reference"},
            ], "types": []})
            actions_url = "https://docs.astro.build/en/guides/actions/"
            options_url = "https://docs.astro.build/en/reference/options/"
            pages = {
                "guides/actions/index": '<h1>Actions</h1><h2 id="server">Server</h2>'
                '<p>starlightxylophone</p><pre><code>defineAction()</code></pre>'
                '<a href="../../reference/options/index#configure">Options</a>'
                '<a href="javascript:alert(1)" onclick="alert(1)">Bad</a><script>alert(1)</script>'
                f'<div class="_attribution"><a class="_attribution-link" href="{actions_url}">Source</a></div>',
                "reference/options/index": '<h1>Options</h1><h2 id="configure">Configure</h2>'
                f'<div class="_attribution"><a class="_attribution-link" href="{options_url}">Source</a></div>',
            }
            write_json(astro / "db.json", pages)
            archive_files(source / "go~1.27.tar.gz", {
                "./meta.json": go_meta,
                "./index.json": {"entries": [{"name": "GoFixture", "path": "pkg/fixture", "type": "Packages"}]},
                "./db.json": {"pkg/fixture": "<h1>Fixture Package</h1><p>Offline archive article.</p>"},
            })

            cli = [str(DEVLINE), "--offline", "--json", "--home", str(library),
                   "--catalog", str(ROOT / "data")]
            importer = [*cli, "docs", "import-devdocs", str(source)]
            plain_importer = [arg for arg in importer if arg != "--json"]

            def query(*args):
                return json.loads(self.run_command(*cli, *args))["data"]

            report = json.loads(self.run_command(*importer, "--doc", "astro"))
            self.assertEqual(report["installed"][0]["count"], 2)
            self.assertEqual(report["failed"], [])
            installed = query("docs", "list", "--installed")
            self.assertEqual([pack["id"] for pack in installed], ["devdocs-astro"])
            release = installed[0]["version"]
            self.assertIn("5.16.5", release)
            self.assertTrue(installed[0]["local_only"])
            self.assertEqual(query("docs", "search", "starlightxylophone")[0]["title"], "Actions")
            hit = query("docs", "search", "AstroActionFixture()", "--docset", "devdocs-astro")[0]
            article = query("docs", "show", hit["id"])
            self.assertEqual(article["source_url"], actions_url)
            self.assertIn("defineAction()", article["body"])
            self.assertIn(f'href="{options_url}#configure"', article["html"])
            self.assertNotIn("javascript:", article["html"])
            self.assertNotIn("onclick=", article["html"])
            self.assertNotIn("<script", article["html"])
            options = query("docs", "show", "Options", "--docset", "devdocs-astro")
            self.assertEqual(options["source_url"], options_url)
            self.assertIn('id="configure"', options["html"])
            if core := os.environ.get("DEVLINE_TEST_CORE"):
                resolved = core_request(core, {"command": "docs.resolve", "id": hit["id"],
                                               "url": f"{options_url}#configure", "home": str(library),
                                               "catalog": str(ROOT / "data")})
                self.assertTrue(resolved["ok"], resolved)
                self.assertEqual(resolved["data"]["id"], options["id"])
                self.assertEqual(resolved["data"]["anchor"], "configure")

            self.assertIn("Imported 2 docsets", self.run_command(*plain_importer))
            installed = query("docs", "list", "--installed")
            self.assertEqual(len(installed), 2)
            self.assertEqual(next(pack["version"] for pack in installed if pack["id"] == "devdocs-astro"), release)
            self.assertIn("Offline archive article", query("docs", "show", "GoFixture")["body"])
            go_release = next(pack["version"] for pack in installed if pack["id"] == "devdocs-go")
            old_go = {**go_meta, "slug": "go~1.9", "release": "1.9.5", "version": "1.9"}
            archive_files(source / "go~1.9.tar.gz", {
                "./meta.json": old_go,
                "./index.json": {"entries": [{"name": "OldGoFixture", "path": "index"}]},
                "./db.json": {"index": "<h1>Earlier Go docs</h1>"},
            })
            self.run_command(*importer, "--doc", "go", "--doc", "go")
            self.assertIn("Offline archive article", query("docs", "show", "GoFixture")["body"])
            query("docs", "remove", "devdocs-go", "--version", go_release)
            self.assertEqual(query("docs", "install", "devdocs-go")["version"], go_release)
            output = root / "private.devlinebundle"
            self.run_command(*cli, "bundle", "export", str(output), "--doc", f"devdocs-astro@{release}", success=False)
            self.assertFalse(output.exists())

            pages["guides/actions/index"] = pages["guides/actions/index"].replace("starlightxylophone", "updatedfixture")
            write_json(astro / "db.json", pages)
            self.run_command(*importer, "--doc", "astro")
            versions = {pack["version"] for pack in query("docs", "list", "--installed") if pack["id"] == "devdocs-astro"}
            self.assertEqual(len(versions), 2)
            self.assertIn(release, versions)
            self.assertIn("starlightxylophone", query("docs", "show", hit["id"])["body"])
            self.assertEqual(query("docs", "search", "updatedfixture", "--docset", "devdocs-astro")[0]["title"], "Actions")

            archive_files(source / "broken.tar.gz", {"../db.json": {}, "./index.json": {"entries": []}})
            report = json.loads(self.run_command(*importer, success=False))
            self.assertEqual({entry["slug"] for entry in report["installed"]}, {"astro", "go~1.27", "go~1.9"})
            self.assertEqual([entry["slug"] for entry in report["failed"]], ["broken"])
            self.assertFalse((root / "db.json").exists())

            shutil.rmtree(source)
            latest = (versions - {release}).pop()
            selected = f"devdocs-astro@{latest}"
            self.assertTrue(query("offline", "status", "--doc", selected)["ready_offline"])
            query("docs", "remove", "devdocs-astro", "--version", latest)
            query("docs", "install", selected)
            self.assertTrue(query("offline", "status", "--doc", selected)["ready_offline"])
            query("library", "purge")
            retained = {pack["id"]: pack["version"] for pack in query("docs", "list", "--installed")}
            self.assertEqual(retained, {"devdocs-astro": latest, "devdocs-go": go_release})


if __name__ == "__main__":
    unittest.main()
