"""NuGet packages kept as a flat folder feed of .nupkg archives."""
import json
import os
import re
import shutil
from xml.sax.saxutils import quoteattr

NUGET = 'https://api.nuget.org/v3/index.json'
VERSIONS = 'https://api.nuget.org/v3-flatcontainer/{}/index.json'
# How many older releases to try when the newest one does not support the installed SDK.
OLDER_ATTEMPTS = 5


def order(version):
    return tuple(int(part) for part in re.findall(r'\d+', version.split('+')[0]))


def build(shelf):
    feed = shelf.out / 'feed'
    feed.mkdir(exist_ok=True)
    # .NET keeps cross-process locks under /tmp/.dotnet and creates that directory through a
    # temporary sibling, which the download sandbox forbids. Creating it directly is allowed.
    for shared in ('/private/tmp/.dotnet', '/private/tmp/.dotnet/shm', '/private/tmp/.dotnet/lockfiles'):
        try:
            os.mkdir(shared)
            os.chmod(shared, 0o1777)
        except OSError:
            pass
    home = shelf.scratch('dotnet-home')
    restored = shelf.scratch('nuget-packages')
    env = {'DOTNET_CLI_HOME': home, 'XDG_DATA_HOME': home / '.local/share', 'NUGET_PACKAGES': restored,
           'NUGET_HTTP_CACHE_PATH': shelf.scratch('nuget-http'), 'DOTNET_CLI_TELEMETRY_OPTOUT': 1,
           'DOTNET_NOLOGO': 1, 'DOTNET_GENERATE_ASPNET_CERTIFICATE': 'false',
           'DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE': 'true', 'DOTNET_EnableDiagnostics': 0,
           'DOTNET_CLI_USE_MSBUILD_SERVER': 0, 'MSBUILDDISABLENODEREUSE': 1, 'UseSharedCompilation': 'false',
           'NuGetAudit': 'false'}
    sdk = re.search(r'^(\d+)\.\d+\.\d+', shelf.run([shelf.tool, '--version'], env=env, capture=True), re.MULTILINE)
    if not sdk:
        raise SystemExit('Could not read the installed .NET SDK version.')
    major = int(sdk.group(1))
    framework = f'net{major}.0'
    used = set()

    def restore(name, version):
        """Restore one package for the SDK's framework; returns the version it resolved to."""
        work = shelf.scratch('project')
        (work / 'Shelf.csproj').write_text(
            '<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>' + framework
            + '</TargetFramework></PropertyGroup><ItemGroup><PackageReference Include=' + quoteattr(name)
            + ' Version=' + quoteattr(version) + ' /></ItemGroup></Project>\n')
        try:
            # The earlier shelf is a source too, so only new archives come from the network.
            shelf.run([shelf.tool, 'restore', '--use-lock-file', '--source', feed, '--source', NUGET,
                       '--verbosity', 'quiet', '-tl:off', '-p:NuGetAudit=false'], cwd=work, env=env)
        except RuntimeError as error:
            # The first sentence says why; the rest lists the sources that were searched.
            reason = re.search(r'error (NU\d+: .*?)(?:\. | \[|$)', str(error), re.MULTILINE)
            raise RuntimeError(reason.group(1) if reason else str(error)) from None
        lock = json.loads((work / 'packages.lock.json').read_text())
        resolved = None
        graph = set()
        for entries in lock['dependencies'].values():
            for identity, entry in entries.items():
                if not entry.get('resolved'):
                    continue
                graph.add((identity.lower(), entry['resolved'].lower()))
                if identity.lower() == name.lower() and entry.get('type') == 'Direct':
                    resolved = entry['resolved']
        # NuGet accepts a package whose assemblies are all for a newer .NET when it also carries
        # build files or analyzers. Such a release restores cleanly and then cannot be compiled against.
        assets = json.loads((work / 'obj/project.assets.json').read_text())
        for key, library in assets.get('libraries', {}).items():
            if key.lower() != f'{name}/{resolved}'.lower():
                continue
            ships = any(path.startswith(('lib/', 'ref/')) for path in library.get('files', []))
            usable = any(target.get(key, {}).get(kind) for target in assets.get('targets', {}).values()
                         for kind in ('compile', 'runtime'))
            if ships and not usable:
                raise RuntimeError(f'NU1202: {name} {resolved} has no assemblies for {framework}')
        used.update(graph)
        shutil.rmtree(work, ignore_errors=True)
        return resolved

    def older(name):
        """Earlier stable releases worth trying: the newest of each release line, newest first."""
        listing = json.loads(shelf.run(['/usr/bin/curl', '-fsSL', '--max-time', '30', VERSIONS.format(name.lower())],
                                       capture=True))
        stable = sorted((version for version in listing['versions'] if '-' not in version), key=order, reverse=True)
        if not stable:
            return []
        lines = {}
        for version in stable[1:]:
            lines.setdefault(order(version)[:2], version)
        lines.pop(order(stable[0])[:2], None)
        candidates = list(lines.values())[:OLDER_ATTEMPTS]
        # Packages versioned with .NET itself always have a line for this SDK; never skip it.
        own = next((version for version in lines.values() if order(version)[0] == major), None)
        if own and own not in candidates:
            candidates[-1] = own
        return candidates

    def download(package):
        # One package per restore: unrelated packages never have to agree on shared versions.
        name = package['name']
        try:
            return restore(name, '*')
        except RuntimeError as newest:
            # NU1202: the newest release targets a newer .NET than the installed SDK builds for.
            if not str(newest).startswith('NU1202'):
                raise
            for version in older(name):
                try:
                    return restore(name, f'[{version}]')
                except RuntimeError as error:
                    if not str(error).startswith('NU1202'):
                        raise
            raise newest

    shelf.each(download)
    # The feed is exactly what the successful restores used: copy what is new and drop the rest,
    # including releases that turned out not to support this SDK. Bundled starters carry their
    # own pinned feed inside the project, so none is seeded here.
    current = set()
    for identity, version in used:
        for archive in (restored / identity / version).glob('*.nupkg'):
            current.add(archive.name)
            if not (feed / archive.name).exists():
                shutil.copy(archive, feed / archive.name)
    for archive in feed.glob('*.nupkg'):
        if archive.name not in current:
            archive.unlink()
