"""Crates kept as a Cargo home: the registry index and .crate archives for every platform."""
import re
import shutil
import subprocess


def build(shelf):
    home = shelf.out / 'cargo-home'
    home.mkdir(exist_ok=True)
    env = {'CARGO_HOME': home, 'CARGO_TARGET_DIR': shelf.scratch('target'), 'CARGO_TERM_COLOR': 'never'}
    cargo = shelf.tool

    def run(args, cwd):
        """Cargo states the reason first and details after it, so report its `error:` line."""
        done = subprocess.run([cargo] + args, cwd=cwd, env=dict(shelf.env, **{key: str(value) for key, value in env.items()}),
                              text=True, errors='replace', stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        if done.returncode != 0:
            reason = next((line[len('error:'):].strip() for line in done.stdout.splitlines() if line.startswith('error:')), '')
            raise RuntimeError(reason or f'cargo exited with {done.returncode}')
        return done.stdout

    # Declaring the local compiler makes Cargo choose releases it can actually build.
    release = re.search(r'\d+\.\d+', shelf.run([cargo, '-V'], env=env, capture=True))
    rust_version = f'rust-version = "{release.group(0)}"\n' if release else ''

    def download(package):
        # One crate per resolution: unrelated crates never have to agree on shared versions.
        work = shelf.scratch('crate')
        (work / 'src').mkdir()
        (work / 'src/lib.rs').write_text('')
        (work / 'Cargo.toml').write_text(
            '[package]\nname = "devline-shelf"\nversion = "0.1.0"\nedition = "2024"\n' + rust_version + '\n[dependencies]\n')
        add = ['add', package['name']]
        if package.get('default_features') is False:
            add.append('--no-default-features')
        for feature in package.get('features') or []:
            add += ['--features', feature]
        run(add, work)
        # Without --target this fetches every platform's crates, which `cargo vendor` needs later.
        run(['fetch'], work)
        tree = run(['tree', '--offline', '--quiet', '--depth', '1', '--prefix', 'none', '--edges', 'normal'], work)
        wanted = package['name'].replace('_', '-').lower()
        version = None
        for line in tree.splitlines()[1:]:
            parts = line.split()
            if len(parts) >= 2 and parts[0].replace('_', '-').lower() == wanted and parts[1].startswith('v'):
                version = parts[1][1:]
        shutil.rmtree(work, ignore_errors=True)
        return version

    shelf.each(download)
    # A starter's own pinned crates must be at hand when a package joins its lock.
    for lock in shelf.starter_files('*/*/project/Cargo.lock'):
        seed = shelf.scratch('seed')
        (seed / 'src').mkdir()
        (seed / 'src/lib.rs').write_text('')
        shutil.copy(lock, seed / 'Cargo.lock')
        shutil.copy(lock.with_name('Cargo.toml'), seed / 'Cargo.toml')
        if lock.with_name('build.rs').exists():
            shutil.copy(lock.with_name('build.rs'), seed / 'build.rs')
        try:
            run(['fetch', '--locked'], seed)
        except Exception as error:
            print('starter crates skipped:', lock.parent.parent.parent.name, error, flush=True)
        shutil.rmtree(seed, ignore_errors=True)
    # Keep the archives and index; Cargo unpacks the sources again on demand.
    shutil.rmtree(home / 'registry' / 'src', ignore_errors=True)
    for stale in home.glob('.package-cache*'):
        stale.unlink()
