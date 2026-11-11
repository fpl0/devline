"""Go modules kept as a download cache that works as an offline GOPROXY."""
import json
import os
import shutil


def build(shelf):
    cache = shelf.out / 'module-cache'
    cache.mkdir(exist_ok=True)
    env = {'GOMODCACHE': cache, 'GOCACHE': shelf.scratch('go-build'), 'GOPATH': shelf.scratch('gopath'),
           'GOTOOLCHAIN': 'local', 'GOWORK': 'off', 'GOFLAGS': '-mod=mod -modcacherw'}

    def download(package):
        work = shelf.scratch('module')
        name = package['name']
        shelf.run([shelf.tool, 'mod', 'init', 'devline.local/shelf'], cwd=work, env=env)
        shelf.run([shelf.tool, 'get', name + '@latest'], cwd=work, env=env)
        imports = package.get('imports') or [name]
        (work / 'shelf.go').write_text('package shelf\n\nimport (\n' + ''.join(f'\t_ {json.dumps(path)}\n' for path in imports) + ')\n')
        # Tidy fails on an import path that does not exist, so nothing needs compiling here.
        shelf.run([shelf.tool, 'mod', 'tidy'], cwd=work, env=env)
        shelf.run([shelf.tool, 'mod', 'download', 'all'], cwd=work, env=env)
        version = shelf.run([shelf.tool, 'list', '-m', '-f', '{{.Version}}', name], cwd=work, env=env, capture=True).strip()
        shutil.rmtree(work, ignore_errors=True)
        return version

    shelf.each(download)
    # A starter's own pinned modules must be resolvable when a package joins its graph.
    for manifest in shelf.starter_files('*/*/project/go.mod'):
        seed = shelf.scratch('seed')
        shutil.copy(manifest, seed / 'go.mod')
        if manifest.with_name('go.sum').exists():
            shutil.copy(manifest.with_name('go.sum'), seed / 'go.sum')
        try:
            shelf.run([shelf.tool, 'mod', 'download', 'all'], cwd=seed, env=env)
        except Exception as error:
            print('starter modules skipped:', manifest.parent.parent.name, str(error).splitlines()[-1], flush=True)
    # Keep the module archives and index; the extracted source trees are rebuilt on demand.
    for item in cache.iterdir():
        if item.name == 'cache':
            continue
        if item.is_dir():
            for folder, names, files in os.walk(item):
                for entry in names + files:
                    try:
                        os.chmod(os.path.join(folder, entry), 0o700)
                    except OSError:
                        pass
            shutil.rmtree(item, ignore_errors=True)
        else:
            item.unlink()
    shutil.rmtree(cache / 'cache' / 'vcs', ignore_errors=True)
