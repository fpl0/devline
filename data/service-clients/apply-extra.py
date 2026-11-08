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

stack, destination, selection_path, offline_value, tool = sys.argv[1:]
project = Path(destination).resolve()
selection = json.loads(Path(selection_path).read_text())
catalog = Path(selection['catalog_dir']).resolve()
artifacts = catalog / 'service-clients/artifacts' / stack
managed_relative = os.environ.get('DEVLINE_PROJECT_CACHE', '.devline-offline')
managed = project / managed_relative
bundled = project / '.devline/packages' if managed_relative == '.devline/cache' else project
offline = offline_value == 'true'
packages = {package['name']: package for item in selection['integrations'] if not item.get('builtin') for package in item['packages']}
env = os.environ.copy()
env['DEVLINE_OFFLINE'] = '1' if offline else '0'
env['PATH'] = str(Path(tool).parent) + os.pathsep + env.get('PATH', '')

def run(command, **kwargs):
    subprocess.run(command, cwd=project, env=env, check=True, **kwargs)

def merge(source, destination):
    if not source.is_dir():
        return
    destination.mkdir(parents=True, exist_ok=True)
    for item in source.iterdir():
        target = destination / item.name
        if item.is_symlink():
            raise SystemExit('Dependency cache contains an unsupported symbolic link: ' + str(item))
        if item.is_dir():
            merge(item, target)
        elif not target.exists():
            shutil.copy2(item, target)
        elif item.name.endswith(('.gem', '.jar', '.pom', '.tar')) and target.read_bytes() != item.read_bytes():
            raise SystemExit('Prepared artifact conflicts with an existing cached file: ' + str(target))

if stack == 'ruby':
    manifest = project / 'Gemfile'
    text = manifest.read_text()
    for name, package in packages.items():
        if not re.fullmatch(r'[A-Za-z0-9_-]+', name): raise SystemExit('Invalid gem name')
        if not re.search(r'\bgem\s*[ (]\s*[\'"]' + re.escape(name) + r'[\'"]', text):
            text += '\ngem ' + json.dumps(name) + ', ' + json.dumps('= ' + package['version']) + '\n'
    manifest.write_text(text)
    merge(managed / 'vendor/cache', project / 'vendor/cache')
    merge(artifacts / 'vendor/cache', project / 'vendor/cache')
    env.update(BUNDLE_IGNORE_CONFIG='1', BUNDLE_GEMFILE=str(manifest), BUNDLE_USER_HOME=str(managed / 'ruby-resolver'), BUNDLE_DISABLE_VERSION_CHECK='1')
    before = project / 'Gemfile.lock'
    old = before.read_text() if before.exists() else None
    command = [tool, str(catalog / 'service-clients/lock-ruby-offline.rb')] if offline else [tool, '-S', 'bundle', 'lock']
    run(command)
    if old is not None:
        previous = managed / 'previous-gem-lock'
        previous.parent.mkdir(parents=True, exist_ok=True)
        previous.write_text(old)
        run([tool, '-rbundler', '-e', 'before,after=ARGV.map{|p| Bundler::LockfileParser.new(File.read(p)).specs.group_by(&:name)}; changed=before.keys.intersection(after.keys).reject{|n|after[n].all?{|s|before[n].any?{|o|o.version==s.version}}}; abort "Adding clients would change pinned gems: #{changed.join(", ")}" unless changed.empty?', str(previous), str(before)])
        previous.unlink()
elif stack == 'java':
    add_maven_dependencies(project / 'pom.xml', packages)
    merge(bundled / 'maven-repository', managed / 'java/repository')
    merge(artifacts / 'repository', managed / 'java/repository')
elif stack == 'elixir':
    merge(artifacts / 'cache', managed / 'elixir')
    arguments = [name + '=' + package['version'] for name, package in packages.items()]
    run([tool, str(catalog / 'service-clients/add-mix-clients.exs'), str(project / 'mix.exs'), *arguments])
    local = managed / 'elixir'
    env.update(MIX_HOME=str(local / 'mix'), HEX_HOME=str(local / 'hex'), MIX_DEPS_PATH=str(local / 'deps'), MIX_ARCHIVES=str(local / 'tools/archives'), MIX_REBAR3=str(local / 'tools/rebar3'), MIX_OS_CONCURRENCY_LOCK='0', HEX_OFFLINE='1' if offline else '0')
    lock = project / 'mix.lock'
    previous = managed / 'previous-mix-lock'
    if lock.exists(): shutil.copyfile(lock, previous)
    run([tool, '-S', 'mix', 'deps.get'])
    if previous.exists():
        run([tool, '-e', 'read = fn path -> {value, _} = Code.eval_file(path); value end; before = read.(hd(System.argv())); after_lock = read.(List.last(System.argv())); changed = Enum.filter(before, fn {key, value} -> Map.has_key?(after_lock, key) and Map.fetch!(after_lock, key) != value end); if changed != [], do: raise("Adding clients would change existing Mix pins: #{inspect(Enum.map(changed, &elem(&1, 0)))}")', str(previous), str(lock)])
        previous.unlink()
elif stack == 'swift':
    cache = managed / 'swift/cache'
    repositories = artifacts / 'cache/repositories'
    destination = cache / 'repositories'
    destination.mkdir(parents=True, exist_ok=True)
    for source in repositories.iterdir():
        if not source.is_dir(): continue
        target = destination / source.name
        if target.exists():
            # Preserve previously prepared versions while adding the selected pack.
            run(['git', '-c', 'protocol.file.allow=always', '--git-dir', str(target), 'fetch', '--quiet', '--update-shallow', str(source), 'refs/tags/*:refs/tags/*'])
        else:
            shutil.copytree(source, target)
    archive = artifacts / 'cache.tar.gz'
    if archive.is_file():
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
        identity = package['name'].lower()
        if identity not in declared:
            run(options + ['add-dependency', package['url'], '--exact', package['version']])
        for product in package['products']:
            if product not in by_name and not any(name == product and owner.lower() == identity for name, owner in linked):
                run(options + ['add-target-dependency', product, target, '--package', package['name']])
else:
    raise SystemExit('Unsupported client adapter: ' + stack)
run(['/bin/sh', str(catalog / 'project-adapters' / (stack + '.sh')), 'prepare', str(project), str(managed), tool])
