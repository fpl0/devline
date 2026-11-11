#!/usr/bin/env python3
"""Apply explicitly selected clients in Devline's disposable project stage."""
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
from manifest_edits import add_maven_dependencies
from xml.sax.saxutils import escape

stack, destination, selection_path, offline_value, tool = sys.argv[1:]
project = Path(destination).resolve()
selection = json.loads(Path(selection_path).read_text())
catalog = Path(selection['catalog_dir']).resolve()
artifacts = catalog / 'service-clients/artifacts' / stack
managed_relative = os.environ.get('DEVLINE_PROJECT_CACHE', '.devline-offline')
managed = project / managed_relative
bundled = project / '.devline/packages' if managed_relative == '.devline/cache' else project
offline = offline_value == 'true'
# The package shelf in the library is one more local store, read in place: a project takes
# from it only what it resolves, never the whole shelf. It is offered whenever it exists, for
# shelf packages and bundled clients alike, so the bundled pack stays a source beside it.
shelf = Path(selection['shelf_dir']).resolve() if selection.get('shelf_dir') else None
scratch = Path(os.environ.get('TMPDIR') or project.parent).resolve()
packages = {package['name']: package for item in selection['integrations'] if not item.get('builtin') for package in item['packages']}
env = os.environ.copy()
env['DEVLINE_OFFLINE'] = 'true' if offline else 'false'
env['PATH'] = str(Path(tool).parent) + os.pathsep + env.get('PATH', '')

finish = None

def run(command, **kwargs):
    try:
        subprocess.run(command, **{'cwd': project, 'env': env, 'check': True, **kwargs})
    except subprocess.CalledProcessError as error:
        # The tool has already said what went wrong; a Python traceback would only bury it.
        hint = ' Something these packages need is not on this Mac yet. Connect once and add them again.' if offline else ''
        raise SystemExit(f'The project could not be built with these packages (status {error.returncode}).{hint}') from None

# Hex keeps its registry as one file: a magic line, a CRC, and a list of {key, value} entries
# whose keys name the repository and package. Join two of them package by package.
HEX_REGISTRY_JOIN = r"""
[target, base, extra | names] = System.argv()
magic = "HEX REGISTRY CACHE 1"
read = fn path ->
  case File.read(path) do
    {:ok, <<^magic::binary-size(20), sum::32, payload::binary>>} ->
      if :erlang.crc32(payload) == sum, do: :erlang.binary_to_term(payload), else: raise("Damaged Hex registry cache: " <> path)
    {:ok, _} -> raise("Unfamiliar Hex registry cache: " <> path)
    {:error, _} -> nil
  end
end
owner = fn {key, _} -> if is_tuple(key) and tuple_size(key) >= 3, do: {elem(key, 1), elem(key, 2)} end
count = fn entries -> Map.new(for {{:versions, repo, name}, versions} <- entries, do: {{repo, name}, length(versions)}) end
added = read.(extra) || raise("Missing Hex registry cache: " <> extra)
kept = read.(base) || Enum.filter(added, &(owner.(&1) == nil))
if List.keyfind(kept, :version, 0) != List.keyfind(added, :version, 0), do: raise("Hex registry cache versions differ")
# A name=version argument also brings the packages that release depends on, optional ones
# included: Hex reads their entries when it checks the lock offline.
linked = for item <- names, [name, version] <- [String.split(item, "=")], {{:deps, _, ^name, ^version}, deps} <- added, dep <- deps, do: elem(dep, 1)
wanted = MapSet.new(Enum.map(names, &hd(String.split(&1, "="))) ++ linked)
added = Enum.filter(added, fn entry -> package = owner.(entry); package != nil and (names == [] or elem(package, 1) in wanted) end)
{old, new} = {count.(kept), count.(added)}
# For a package both sides know, the side that lists more releases is the fresher one.
wins = fn package -> Map.get(new, package, 0) >= Map.get(old, package, 0) end
entries = Enum.reject(kept, fn entry -> package = owner.(entry); package != nil and Map.has_key?(new, package) and wins.(package) end) ++
  Enum.filter(added, fn entry -> wins.(owner.(entry)) end)
payload = :erlang.term_to_binary(entries)
File.mkdir_p!(Path.dirname(target))
File.write!(target, magic <> <<:erlang.crc32(payload)::32>> <> payload)
"""

def merge(source, destination, created=None):
    if not source.is_dir():
        return
    if created is not None and not destination.exists(): created.append(destination)
    destination.mkdir(parents=True, exist_ok=True)
    for item in source.iterdir():
        target = destination / item.name
        if item.is_symlink():
            raise SystemExit('Dependency cache contains an unsupported symbolic link: ' + str(item))
        if item.is_dir():
            merge(item, target, created)
        elif not target.exists():
            shutil.copy2(item, target)
            if created is not None: created.append(target)
        elif item.name.endswith(('.gem', '.jar', '.pom', '.tar')) and target.read_bytes() != item.read_bytes():
            raise SystemExit('Prepared artifact conflicts with an existing cached file: ' + str(target))

if stack == 'ruby':
    manifest = project / 'Gemfile'
    # A script-only project gets its first Gemfile when a shelf package is added.
    text = manifest.read_text() if shelf is None or manifest.exists() else 'source "https://rubygems.org"\n'
    for name, package in packages.items():
        if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._-]*' if shelf else r'[A-Za-z0-9_-]+', name): raise SystemExit('Invalid gem name')
        if not re.search(r'\bgem\s*[ (]\s*[\'"]' + re.escape(name) + r'[\'"]', text):
            text += '\ngem ' + json.dumps(name) + ', ' + json.dumps('= ' + package['version']) + '\n'
    manifest.write_text(text)
    vendor = project / 'vendor/cache'
    merge(managed / 'vendor/cache', vendor)
    if shelf is None: merge(artifacts / 'vendor/cache', vendor)
    env.update(BUNDLE_IGNORE_CONFIG='1', BUNDLE_GEMFILE=str(manifest), BUNDLE_USER_HOME=str(managed / 'ruby-resolver'), BUNDLE_DISABLE_VERSION_CHECK='1')
    before = project / 'Gemfile.lock'
    old = before.read_text() if before.exists() else None
    command = [tool, str(catalog / 'service-clients/lock-ruby-offline.rb')] if offline else [tool, '-S', 'bundle', 'lock']
    if shelf is None:
        run(command)
    else:
        # Resolve against links to every shelf gem, then keep only the gems the lock names.
        pool = scratch / 'gem-pool'
        pool.mkdir()
        for source in (shelf / 'vendor/cache', artifacts / 'vendor/cache', vendor):
            for gem in sorted(source.glob('*.gem')):
                link = pool / gem.name
                link.unlink(missing_ok=True)
                link.symlink_to(gem.resolve())
        local = [tool, str(catalog / 'service-clients/lock-ruby-offline.rb')]
        pooled = dict(env, BUNDLE_CACHE_PATH=str(pool))
        if offline:
            run(local, env=pooled)
        elif subprocess.run(local, cwd=project, env=pooled, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode != 0:
            # Connected: the registry fills in whatever the shelf lacks.
            run(command)
        listing = 'puts Bundler::LockfileParser.new(File.read(ARGV[0])).specs.map(&:full_name)'
        locked = subprocess.check_output([tool, '-rbundler', '-e', listing, str(before)], cwd=project, env=env, text=True).split()
        vendor.mkdir(parents=True, exist_ok=True)
        def keep_locked(store):
            for name in locked:
                source, target = store / (name + '.gem'), vendor / (name + '.gem')
                if source.exists() and not target.exists():
                    shutil.copy2(source.resolve(), target)
        keep_locked(pool)
        shutil.rmtree(pool)
        # Gems the registry supplied land in the managed cache during preparation.
        finish = lambda: keep_locked(managed / 'vendor/cache')
    if old is not None:
        previous = managed / 'previous-gem-lock'
        previous.parent.mkdir(parents=True, exist_ok=True)
        previous.write_text(old)
        run([tool, '-rbundler', '-e', 'before,after=ARGV.map{|p| Bundler::LockfileParser.new(File.read(p)).specs.group_by(&:name)}; changed=before.keys.intersection(after.keys).reject{|n|after[n].all?{|s|before[n].any?{|o|o.version==s.version}}}; abort "Adding clients would change pinned gems: #{changed.join(", ")}" unless changed.empty?', str(previous), str(before)])
        previous.unlink()
elif stack == 'java':
    add_maven_dependencies(project / 'pom.xml', packages)
    repository = managed / 'java/repository'
    merge(bundled / 'maven-repository', repository)
    if shelf is None:
        merge(artifacts / 'repository', repository)
    else:
        # Let Maven read the shelf as a mirror and copy only what this build resolves. The mirror
        # is named central so a later plain offline build accepts what it fetched.
        settings = scratch / 'shelf-settings.xml'
        settings.write_text('<settings xmlns="http://maven.apache.org/SETTINGS/1.0.0"><mirrors><mirror><id>central</id><mirrorOf>*</mirrorOf><url>'
                            + escape((shelf / 'repository').as_uri()) + '</url></mirror></mirrors></settings>\n')
        # Maven's own settings file cannot be parsed when the home path contains "--".
        global_settings = scratch / 'global-settings.xml'
        global_settings.write_text('<settings xmlns="http://maven.apache.org/SETTINGS/1.0.0"/>\n')
        options = f'-Duser.home="{os.environ["HOME"]}" -Djava.io.tmpdir="{scratch}"'
        mirrored = dict(env, JDK_JAVA_OPTIONS=(env.get('JDK_JAVA_OPTIONS', '') + ' ' + options).strip())
        def build_from_shelf():
            done = subprocess.run([tool, '-gs', str(global_settings), '-B', '-ntp', '-s', str(settings), '-Dmaven.repo.local=' + str(repository), '-DskipTests', 'clean', 'verify'],
                                  cwd=project, env=mirrored, text=True, errors='replace', stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            # A lookup the shelf could not answer must not block a later build.
            for marker in repository.rglob('*.lastUpdated'): marker.unlink()
            return done
        done = build_from_shelf()
        if done.returncode != 0:
            # A bundled client the shelf does not hold: its prepared pack fills the gap.
            added = []
            merge(artifacts / 'repository', repository, added)
            done = build_from_shelf()
            if done.returncode != 0:
                # The pack did not help either; do not leave it behind in the project.
                for path in reversed(added):
                    if path.is_dir(): shutil.rmtree(path, ignore_errors=True)
                    else: path.unlink(missing_ok=True)
        if done.returncode != 0 and offline:
            sys.stderr.write('\n'.join(line for line in done.stdout.splitlines() if line.startswith('[ERROR]')) + '\n')
            raise SystemExit('Maven could not build this project from the package shelf alone. If an artifact is missing, connect and add the packages again.')
elif stack == 'elixir':
    local = managed / 'elixir'
    hex_home = local / 'hex'
    def join_registry(target, base, extra, names=()):
        return subprocess.run([tool, '-e', HEX_REGISTRY_JOIN, str(target), str(base), str(extra), *names], cwd=scratch, env=env).returncode == 0
    if shelf is None:
        known = (hex_home / 'cache.bin').exists()
        merge(artifacts / 'cache', local)
        # merge() keeps a registry cache the project already has; teach it the bundled packages.
        if known: join_registry(hex_home / 'cache.bin', hex_home / 'cache.bin', artifacts / 'cache/hex/cache.bin')
    else:
        # Resolve in a scratch Hex home: links to every shelf tarball, and a registry cache that
        # joins the project's entries with the shelf's. Only what gets locked is kept afterwards.
        merge(shelf / 'cache/tools', local / 'tools')
        hex_home = scratch / 'hex-home'
        (hex_home / 'packages/hexpm').mkdir(parents=True)
        for source in (artifacts / 'cache/hex/packages/hexpm', shelf / 'cache/hex/packages/hexpm', local / 'hex/packages/hexpm'):
            for archive in sorted(source.glob('*.tar')):
                link = hex_home / 'packages/hexpm' / archive.name
                link.unlink(missing_ok=True)
                link.symlink_to(archive.resolve())
        # With an unfamiliar cache format the shelf's whole registry still resolves offline.
        if not join_registry(hex_home / 'cache.bin', local / 'hex/cache.bin', shelf / 'cache/hex/cache.bin'):
            shutil.copyfile(shelf / 'cache/hex/cache.bin', hex_home / 'cache.bin')
        if (artifacts / 'cache/hex/cache.bin').is_file():
            join_registry(hex_home / 'cache.bin', hex_home / 'cache.bin', artifacts / 'cache/hex/cache.bin')
    arguments = [name + '=' + package['version'] for name, package in packages.items()]
    run([tool, str(catalog / 'service-clients/add-mix-clients.exs'), str(project / 'mix.exs'), *arguments])
    env.update(MIX_HOME=str(local / 'mix'), HEX_HOME=str(hex_home), MIX_DEPS_PATH=str(local / 'deps'), MIX_ARCHIVES=str(local / 'tools/archives'), MIX_REBAR3=str(local / 'tools/rebar3'), MIX_OS_CONCURRENCY_LOCK='0', HEX_OFFLINE='1' if offline else '0')
    lock = project / 'mix.lock'
    previous = managed / 'previous-mix-lock'
    if lock.exists(): shutil.copyfile(lock, previous)
    run([tool, '-S', 'mix', 'deps.get'])
    if previous.exists():
        run([tool, '-e', 'read = fn path -> {value, _} = Code.eval_file(path); value end; before = read.(hd(System.argv())); after_lock = read.(List.last(System.argv())); changed = Enum.filter(before, fn {key, value} -> Map.has_key?(after_lock, key) and Map.fetch!(after_lock, key) != value end); if changed != [], do: raise("Adding clients would change existing Mix pins: #{inspect(Enum.map(changed, &elem(&1, 0)))}")', str(previous), str(lock)])
        previous.unlink()
    if shelf is not None:
        locked = re.findall(r'\{:hex, :"?([a-z0-9_]+)"?, "([^"]+)"', lock.read_text())
        (local / 'hex/packages/hexpm').mkdir(parents=True, exist_ok=True)
        for name, version in locked:
            source, target = hex_home / 'packages/hexpm' / f'{name}-{version}.tar', local / 'hex/packages/hexpm' / f'{name}-{version}.tar'
            if source.exists() and not target.exists():
                shutil.copy2(source.resolve(), target)
        if not join_registry(local / 'hex/cache.bin', local / 'hex/cache.bin', hex_home / 'cache.bin', sorted({name + '=' + version for name, version in locked})):
            shutil.copyfile(hex_home / 'cache.bin', local / 'hex/cache.bin')
        shutil.rmtree(hex_home)
elif stack == 'swift':
    cache = managed / 'swift/cache'
    repositories = artifacts / 'cache/repositories'
    destination = cache / 'repositories'
    destination.mkdir(parents=True, exist_ok=True)
    def adopt(source, into, clone=False):
        target = into / source.name
        if target.exists():
            # Preserve previously prepared versions while adding the selected pack.
            run(['git', '-c', 'protocol.file.allow=always', '--git-dir', str(target), 'fetch', '--quiet', '--update-shallow', str(source), 'refs/tags/*:refs/tags/*'])
        elif not clone or subprocess.run(['/bin/cp', '-cR', str(source), str(target)], stderr=subprocess.DEVNULL).returncode != 0:
            shutil.rmtree(target, ignore_errors=True)
            shutil.copytree(source, target)
    for source in repositories.iterdir() if shelf is None else ():
        if source.is_dir(): adopt(source, destination)
    archive = artifacts / 'cache.tar.gz'
    if shelf is None and archive.is_file():
        with tarfile.open(archive) as source:
            source.extractall(managed / 'swift', filter='data')
    options = [tool, 'package', '--disable-sandbox', '--cache-path', str(cache), '--scratch-path', str(managed / 'swift/workspace'), '--skip-update']
    description = json.loads(subprocess.check_output(options + ['dump-package'], cwd=project, env=env))
    targets = [target for target in description['targets'] if target['type'] == 'executable'] or [target for target in description['targets'] if target['type'] == 'regular']
    if not targets: raise SystemExit('Choose a Swift package with an app or library target for the selected clients.')
    target = targets[0]['name']
    declared = {source['identity'].lower() for dependency in description['dependencies'] for source in dependency.get('sourceControl', [])}
    linked = {tuple(dependency['product'][:2]) for dependency in targets[0]['dependencies'] if 'product' in dependency}
    by_name = {dependency['byName'][0] for dependency in targets[0]['dependencies'] if 'byName' in dependency}
    for package in packages.values():
        # SwiftPM names a package after its repository, which a shelf entry's display name may not match.
        owner_name = package['name'] if shelf is None else package['url'].rstrip('/').rsplit('/', 1)[-1].removesuffix('.git')
        identity = owner_name.lower()
        if identity not in declared:
            run(options + ['add-dependency', package['url'], '--exact', package['version']])
        for product in package['products']:
            if product not in by_name and not any(name == product and owner.lower() == identity for name, owner in linked):
                run(options + ['add-target-dependency', product, target, '--package', owner_name])
    if shelf is not None:
        # Resolve against a private copy of the shelf's repository cache joined with the bundled
        # pack's and the project's own, then keep only the repositories the resolved graph names.
        pool = scratch / 'swift-shelf'
        pooled = pool / 'cache/repositories'
        pooled.mkdir(parents=True)
        if archive.is_file():
            with tarfile.open(archive) as source:
                source.extractall(pool, filter='data')
        for store in (shelf / 'cache/repositories', repositories, destination):
            for repository in sorted(store.iterdir()) if store.is_dir() else ():
                if repository.is_dir(): adopt(repository, pooled, clone=True)
        local = dict(env, CLANG_MODULE_CACHE_PATH=str(pool / 'clang-cache'), SWIFTPM_MODULECACHE_OVERRIDE=str(pool / 'manifest-modules'))
        resolve = [tool, 'package', '--disable-sandbox', '--manifest-cache', 'local', '--cache-path', str(pool / 'cache'), '--scratch-path', str(pool / 'build'),
                   '--config-path', str(pool / 'config'), '--security-path', str(pool / 'security')]
        if offline:
            run(resolve + ['--skip-update', 'resolve'], env=local)
        elif subprocess.run(resolve + ['--skip-update', 'resolve'], cwd=project, env=local, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode != 0:
            # Connected: the network fills in whatever the shelf lacks.
            run(resolve + ['resolve'], env=local)
        def canonical(url):
            url = url.strip().lower().rstrip('/')
            return url[:-4] if url.endswith('.git') else url
        pinned = {canonical(pin['location']) for pin in json.loads((project / 'Package.resolved').read_text()).get('pins', [])} if (project / 'Package.resolved').exists() else set()
        for repository in sorted(pooled.iterdir()):
            found = re.search(r'^\s*url\s*=\s*(\S+)', (repository / 'config').read_text(), re.MULTILINE) if (repository / 'config').is_file() else None
            if found and canonical(found.group(1)) in pinned: adopt(repository, destination)
        shutil.rmtree(pool, ignore_errors=True)
else:
    raise SystemExit('Unsupported client adapter: ' + stack)
run(['/bin/sh', str(catalog / 'project-adapters' / (stack + '.sh')), 'prepare', str(project), str(managed), tool])
if finish: finish()
