#!/usr/bin/env python3
"""Verify the bundled .NET starters or a generated copy from an empty cache (macOS)."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', nargs='*', type=Path, help='Project directories to test (defaults to all three starters)')
args = parser.parse_args()
sandbox = Path('/usr/bin/sandbox-exec')
if not sandbox.is_file():
    raise SystemExit('This verification requires the macOS sandbox-exec network boundary.')
tool = shutil.which('dotnet')
if not tool:
    raise SystemExit('Install a local .NET 10 SDK first; Devline does not manage runtimes.')
policy = '(version 1)(allow default)(deny network-outbound)(allow network-outbound (remote ip "localhost:*"))'
ignore_build_outputs = shutil.ignore_patterns(
    'bin', 'obj', 'TestResults', 'notes.db*', 'tasks.db*', '.devline-infra', '.devline-offline')


def verify(source):
    source = source.resolve()

    def ignore_local_state(directory, contents):
        ignored = ignore_build_outputs(directory, contents)
        if Path(directory) == source / '.devline':
            ignored.update({'cache', 'services'}.intersection(contents))
        return ignored

    with tempfile.TemporaryDirectory(prefix='devline-dotnet-offline-') as temporary:
        root = Path(temporary)
        project = root / 'project'
        shutil.copytree(source, project, ignore=ignore_local_state)
        home = root / 'home'
        packages = root / 'empty-nuget'
        home.mkdir()
        packages.mkdir()
        env = dict(os.environ, HOME=str(home), DOTNET_CLI_HOME=str(home), NUGET_PACKAGES=str(packages),
                   NUGET_HTTP_CACHE_PATH=str(root / 'http-cache'), DEVLINE_TOOLCHAIN=tool, DEVKIT_TOOLCHAIN=tool,
                   DOTNET_NOLOGO='1', DOTNET_GENERATE_ASPNET_CERTIFICATE='false')
        # Check the actual boundary as well as passing a policy argument.
        blocked = subprocess.run([str(sandbox), '-p', policy, '/usr/bin/curl', '--connect-timeout', '3',
                                  '--max-time', '5', '--silent', '--output', '/dev/null',
                                  'https://api.nuget.org/v3/index.json'], env=env)
        if blocked.returncode == 0:
            raise SystemExit('Outbound network was unexpectedly reachable. Offline proof is invalid.')
        test = ['./devline.sh', 'test'] if (project / 'devline.sh').is_file() else ['./test-offline.sh']
        subprocess.run([str(sandbox), '-p', policy, '/bin/sh', *test], cwd=project, env=env, check=True, timeout=180)
        count = len(list(packages.glob('*/*/*.nupkg')))
        print(f'PASS: {source.parent.parent.name if source.name == "project" else source.name}. '
              f'Restored {count} packages into an empty cache and passed offline verification.', flush=True)


templates = Path(__file__).resolve().parents[1] / 'data/templates/dotnet'
for source in args.project or [
    templates / name / 'project' for name in ('json-cli/1.0.0', 'aspnet-api/2.0.0', 'aspnet-mvc/1.0.0')
]:
    verify(source)
