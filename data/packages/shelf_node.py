"""npm packages kept as an npm cache that `npm install --offline` resolves from."""
import json
import pathlib
import shutil
import subprocess


def build(shelf):
    cache = shelf.out / 'npm-cache'
    cache.mkdir(exist_ok=True)
    beside = pathlib.Path(shelf.tool).with_name('npm')
    npm = [str(beside) if beside.exists() else 'npm']
    flags = ['--ignore-scripts', '--no-audit', '--no-fund', '--no-update-notifier', '--cache', str(cache)]

    def install(args, cwd):
        """npm ends a failure with the path of its log, so report the lines that say why."""
        done = subprocess.run(npm + args + flags, cwd=cwd, env=shelf.env, text=True, errors='replace',
                              stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        if done.returncode == 0:
            return
        lines = [line[len('npm error'):].strip() for line in done.stdout.splitlines() if line.startswith('npm error')]
        lines = [line for line in lines if line and 'complete log of this run' not in line]
        code = next((line[len('code '):] for line in lines if line.startswith('code ')), '')
        detail = next((line for line in lines if not line.startswith(('code ', 'syscall ', 'errno ', 'path '))), '')
        raise RuntimeError(' '.join(part for part in (code, detail) if part) or f'npm exited with {done.returncode}')

    def download(package):
        # One package per install: the cache keeps every archive and registry document it
        # touched, and unrelated packages never have to agree on shared dependencies.
        name = package['name']
        work = shelf.scratch('package')
        (work / 'package.json').write_text(json.dumps({'name': 'devline-shelf', 'version': '1.0.0', 'private': True}))
        install(['install', '--save-exact', name], work)
        lock = json.loads((work / 'package-lock.json').read_text())
        version = lock['packages']['node_modules/' + name]['version']
        shutil.rmtree(work, ignore_errors=True)
        return version

    shelf.each(download)
    # A starter's own pinned archives must be at hand when a package joins its lock.
    for lock in shelf.starter_files('*/*/project/package-lock.json'):
        seed = shelf.scratch('seed')
        shutil.copy(lock, seed / 'package-lock.json')
        shutil.copy(lock.with_name('package.json'), seed / 'package.json')
        try:
            install(['ci'], seed)
        except Exception as error:
            print('starter packages skipped:', lock.parent.parent.parent.name, error, flush=True)
        shutil.rmtree(seed, ignore_errors=True)
    # Keep the archives and index, not npm's run logs.
    shutil.rmtree(cache / '_logs', ignore_errors=True)
    (cache / '_update-notifier-last-checked').unlink(missing_ok=True)
