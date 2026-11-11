#!/usr/bin/env python3
"""Download a runtime's package shelf with its own package manager.

Usage: build-shelf.py PLAN.json OUT_DIR TOOL

PLAN.json holds {"stack", "packages": [catalog entries], "custom", "catalog_dir", "toolchain"},
and optionally "only" (names to fetch) with "previous" (the existing shelf's ready.json).
OUT_DIR may already hold an earlier shelf to extend. Only OUT_DIR, HOME and TMPDIR are writable.
Each runtime lives in shelf_<stack>.py and defines build(shelf). A package that cannot be
resolved is recorded in ready.json and does not stop the rest.
"""
import importlib.util
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import time

BASE = pathlib.Path(__file__).resolve().parent


class Shelf:
    def __init__(self, plan, out, tool):
        self.stack = plan['stack']
        self.packages = plan['packages']
        self.catalog = pathlib.Path(plan['catalog_dir'])
        self.out = pathlib.Path(out)
        self.tool = str(tool)
        self.resolved = []
        self.failed = []
        only = set(plan.get('only') or [])
        if only and plan.get('previous'):
            # Add to what is already there instead of resolving the whole shelf again.
            self.packages = [package for package in self.packages if package['name'] in only]
            self.resolved = [entry for entry in plan['previous'].get('packages', []) if entry['name'] not in only]
            self.failed = [entry for entry in plan['previous'].get('failed', []) if entry['name'] not in only]
        self.env = dict(os.environ, CI='true')
        self._root = pathlib.Path(tempfile.mkdtemp(prefix='shelf-', dir=os.environ['TMPDIR']))
        self._count = 0

    def scratch(self, name='work'):
        """A new empty directory that is discarded when the download ends."""
        self._count += 1
        path = self._root / f'{self._count}-{name}'
        path.mkdir()
        return path

    def run(self, args, cwd=None, env=None, capture=False):
        """Run a command, raising with the end of its output when it fails."""
        merged = dict(self.env, **{key: str(value) for key, value in (env or {}).items()})
        done = subprocess.run([str(arg) for arg in args], cwd=cwd, env=merged, text=True,
                              stdout=subprocess.PIPE, stderr=subprocess.STDOUT, errors='replace')
        if done.returncode != 0:
            tail = '\n'.join(done.stdout.strip().splitlines()[-6:])
            raise RuntimeError(tail or f'{args[0]} exited with {done.returncode}')
        return done.stdout if capture else None

    def each(self, download):
        """Call download(package) for every package; it returns the resolved version."""
        total = len(self.packages)
        progress = os.environ.get('DEVLINE_PROGRESS')
        for index, package in enumerate(self.packages, 1):
            name = package['name']
            if progress:
                pathlib.Path(progress).write_text(f'{index} of {total}')
            try:
                version = download(package)
                if not version:
                    raise RuntimeError('No version was resolved')
                entry = {key: value for key, value in package.items() if key not in ('summary', 'category')}
                entry['version'] = str(version)
                self.resolved.append(entry)
                print(f'[{index}/{total}] {name} {version}', flush=True)
            except Exception as error:  # one bad package must not sink the shelf
                message = str(error).strip().splitlines()[-1][:300] if str(error).strip() else type(error).__name__
                self.failed.append({'name': name, 'error': message})
                print(f'[{index}/{total}] {name} FAILED: {message}', flush=True)

    def starter_files(self, pattern):
        """Manifests of the bundled starters, so their projects can be extended offline too."""
        return sorted((self.catalog / 'templates' / self.stack).glob(pattern))

    def close(self):
        def writable(function, path, _error):
            os.chmod(path, 0o700)
            function(path)
        shutil.rmtree(self._root, onerror=writable)


def size(root):
    total = 0
    for folder, _names, files in os.walk(root):
        for name in files:
            try:
                total += os.lstat(os.path.join(folder, name)).st_size
            except OSError:
                pass
    return total


def main():
    plan = json.loads(pathlib.Path(sys.argv[1]).read_text())
    out, tool = sys.argv[2], sys.argv[3]
    stack = plan['stack']
    spec = importlib.util.spec_from_file_location('shelf_' + stack, BASE / f'shelf_{stack}.py')
    if spec is None or not (BASE / f'shelf_{stack}.py').is_file():
        raise SystemExit(f'The package shelf does not support {stack} yet.')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    shelf = Shelf(plan, out, tool)
    marker = shelf.out / 'ready.json'
    marker.unlink(missing_ok=True)
    try:
        module.build(shelf)
    finally:
        shelf.close()
    marker.write_text(json.dumps({
        'schema_version': 1,
        'stack': stack,
        'packages': shelf.resolved,
        'failed': shelf.failed,
        'custom': plan.get('custom', []),
        'prepared_at': int(time.time()),
        'toolchain': plan.get('toolchain'),
        'bytes': size(shelf.out),
    }, indent=2) + '\n')
    print(f'READY {stack}: {len(shelf.resolved)} packages, {len(shelf.failed)} failed', flush=True)


if __name__ == '__main__':
    main()
