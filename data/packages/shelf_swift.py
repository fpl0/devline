"""Swift packages kept as SwiftPM's repository cache, trimmed to the releases that were resolved."""
import json
import re
import shutil
import subprocess


def canonical(url):
    """Manifests spell one repository several ways; SwiftPM caches each spelling separately."""
    url = url.strip().lower().rstrip('/')
    return url[:-4] if url.endswith('.git') else url


def origin(repository):
    found = re.search(r'^\s*url\s*=\s*(\S+)', (repository / 'config').read_text(), re.MULTILINE)
    return found.group(1) if found else ''


def build(shelf):
    store = shelf.out / 'cache/repositories'
    store.mkdir(parents=True, exist_ok=True)
    # SwiftPM needs whole repositories to choose versions. They are cloned once here, shared by
    # every package in this download, and discarded; the shelf keeps only the chosen releases.
    mirrors = shelf.scratch('swiftpm-cache')
    modules = shelf.scratch('swift-modules')
    env = {'CLANG_MODULE_CACHE_PATH': modules / 'clang', 'SWIFTPM_MODULECACHE_OVERRIDE': modules / 'manifests',
           'GIT_TERMINAL_PROMPT': '0'}

    def xcrun(*arguments):
        try:
            return subprocess.run(['/usr/bin/xcrun', '--no-cache', *arguments], env=shelf.env, text=True,
                                  stdout=subprocess.PIPE, stderr=subprocess.DEVNULL).stdout.strip()
        except OSError:
            return ''

    sdk = xcrun('--sdk', 'macosx', '--show-sdk-path')
    if sdk:
        env['SDKROOT'] = sdk
    # The real git, not the /usr/bin shim, which warns on every call inside the sandbox.
    tool = xcrun('--find', 'git') or '/usr/bin/git'

    def git(repository, *arguments):
        done = subprocess.run([tool, '-c', 'protocol.file.allow=always', '--git-dir', str(repository), *map(str, arguments)],
                              env=dict(shelf.env, **{key: str(value) for key, value in env.items()}), text=True, errors='replace',
                              stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        if done.returncode != 0:
            raise RuntimeError(' '.join(done.stderr.split())[-300:] or f'git exited with {done.returncode}')
        return done.stdout

    def keep(pin):
        """Copy one resolved release, without history, into every cache entry of its repository."""
        wanted, revision = canonical(pin['location']), pin['state']['revision']
        for mirror in sorted((mirrors / 'repositories').iterdir()):
            if not mirror.is_dir() or canonical(origin(mirror)) != wanted:
                continue
            tags = git(mirror, 'tag', '--points-at', revision).split()
            if not tags:
                raise RuntimeError(f'No release tag for {pin["identity"]} {pin["state"].get("version", revision)}')
            target = store / mirror.name
            if not target.exists():
                shelf.run([tool, 'init', '--quiet', '--bare', target], env=env)
                git(target, 'config', 'remote.origin.url', origin(mirror))
            git(target, 'fetch', '--quiet', '--depth', '1', mirror.as_uri(), *[f'+refs/tags/{tag}:refs/tags/{tag}' for tag in tags])
            git(target, 'update-ref', '--no-deref', 'HEAD', revision)

    def download(package):
        name, url, products = package['name'], package.get('url') or '', package.get('products') or []
        if not re.fullmatch(r'https://[A-Za-z0-9._~/-]+', url) or not all(re.fullmatch(r'[A-Za-z0-9_-]+', item) for item in products):
            raise RuntimeError('Swift packages need an https repository URL and product names')
        identity = url.rstrip('/').rsplit('/', 1)[-1].removesuffix('.git')
        work = shelf.scratch('package')
        (work / 'Sources/Shelf').mkdir(parents=True)
        (work / 'Sources/Shelf/Shelf.swift').write_text('\n')
        # An open range lets SwiftPM pick the newest release this toolchain can load.
        (work / 'Package.swift').write_text(
            '// swift-tools-version: 5.9\nimport PackageDescription\nlet package = Package(name: "Shelf", dependencies: [\n'
            f'  .package(url: {json.dumps(url)}, "0.0.0"..<"1000000.0.0")\n], targets: [.target(name: "Shelf", dependencies: [\n'
            + ''.join(f'  .product(name: {json.dumps(item)}, package: {json.dumps(identity)}),\n' for item in products) + '])])\n')
        swift = [shelf.tool, 'package', '--disable-sandbox', '--manifest-cache', 'local', '--cache-path', mirrors,
                 '--scratch-path', work / '.build', '--config-path', work / 'config', '--security-path', work / 'security']

        def swiftpm(*arguments):
            done = subprocess.run([str(item) for item in swift] + list(arguments), cwd=work, text=True, errors='replace',
                                  env=dict(shelf.env, **{key: str(value) for key, value in env.items()}),
                                  stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            state = work / '.build/workspace-state.json'
            if re.search(r'\.(xcframework|artifactbundle)\b|binary artifact', done.stdout) or (
                    state.is_file() and json.loads(state.read_text()).get('object', {}).get('artifacts')):
                # Prebuilt frameworks are downloaded outside the repository cache, where the
                # project's offline check does not look.
                raise RuntimeError('Ships prebuilt binaries, which cannot be added offline yet')
            if done.returncode != 0:
                lines = [' '.join(line.split()) for line in done.stdout.splitlines() if line.strip()]
                if any('not found' in line.lower() for line in lines):
                    raise RuntimeError('Repository not found')
                errors = [line for line in lines if line.startswith('error:')]
                raise RuntimeError((errors or lines or ['SwiftPM could not resolve the package'])[-1].rstrip(':'))

        try:
            swiftpm('resolve')
            # Loading the package graph proves the listed products exist in this release.
            swiftpm('show-dependencies', '--format', 'json')
            pins = json.loads((work / 'Package.resolved').read_text())['pins']
        finally:
            shutil.rmtree(work, ignore_errors=True)
        version = next((pin['state'].get('version') for pin in pins if canonical(pin['location']) == canonical(url)), None)
        if not version:
            raise RuntimeError('No tagged release was resolved')
        for pin in pins:
            keep(pin)
        return version

    shelf.each(download)
    # Give every spelling of a repository the same releases, whichever one a project resolves.
    groups = {}
    for repository in sorted(store.iterdir()):
        if repository.is_dir():
            groups.setdefault(canonical(origin(repository)), []).append(repository)
    for repositories in groups.values():
        for target in repositories:
            for source in repositories:
                if source != target:
                    git(target, 'fetch', '--quiet', '--depth', '1', '--update-shallow', source.as_uri(), 'refs/tags/*:refs/tags/*')
