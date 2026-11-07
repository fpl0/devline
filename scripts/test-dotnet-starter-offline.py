#!/usr/bin/env python3
"""Verify the .NET starter from an empty cache with outbound networking denied (macOS)."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', nargs='?', type=Path, default=Path(__file__).resolve().parents[1] / 'data/templates/dotnet/aspnet-api/1.2.0/project')
args = parser.parse_args()
sandbox = Path('/usr/bin/sandbox-exec')
if not sandbox.is_file():
    raise SystemExit('This verification requires the macOS sandbox-exec network boundary.')
tool = shutil.which('dotnet')
if not tool:
    raise SystemExit('Install a local .NET 10 SDK first; Devline does not manage runtimes.')
policy = '(version 1)(allow default)(deny network-outbound)(allow network-outbound (remote ip "localhost:*"))'
with tempfile.TemporaryDirectory(prefix='devline-dotnet-offline-') as temporary:
    root = Path(temporary)
    project = root / 'project'
    shutil.copytree(args.project.resolve(), project, ignore=shutil.ignore_patterns('bin', 'obj', 'TestResults', 'notes.db*', '.devline-infra'))
    home = root / 'home'
    packages = root / 'empty-nuget'
    home.mkdir()
    packages.mkdir()
    env = os.environ.copy()
    env.update(HOME=str(home), DOTNET_CLI_HOME=str(home), NUGET_PACKAGES=str(packages), NUGET_HTTP_CACHE_PATH=str(root / 'http-cache'), DEVKIT_TOOLCHAIN=tool)
    # Check the actual boundary, not only the presence of a policy argument.
    blocked = subprocess.run([str(sandbox), '-p', policy, '/usr/bin/curl', '--connect-timeout', '3', '--max-time', '5', '--silent', '--output', '/dev/null', 'https://api.nuget.org/v3/index.json'], env=env)
    if blocked.returncode == 0:
        raise SystemExit('Outbound network was unexpectedly reachable; offline proof is invalid.')
    subprocess.run([str(sandbox), '-p', policy, '/bin/sh', './test-offline.sh'], cwd=project, env=env, check=True, timeout=180)
    count = len(list(packages.glob('*/*/*.nupkg')))
    print(f'PASS: restored {count} packages from the bundled feed into an empty cache; build and all tests passed with outbound network denied.', flush=True)
