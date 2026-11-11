"""RubyGems archives kept as one Bundler vendor/cache directory."""
import json
import re
import shutil


def build(shelf):
    store = shelf.out / 'vendor/cache'
    store.mkdir(parents=True, exist_ok=True)
    home = shelf.scratch('bundler')
    # One download area for every package, so a shared dependency is fetched once.
    env = {'BUNDLE_IGNORE_CONFIG': '1', 'BUNDLE_USER_HOME': home / 'user', 'BUNDLE_APP_CONFIG': home / 'app',
           'BUNDLE_PATH': home / 'gems', 'GEM_HOME': home / 'gem-home', 'GEM_SPEC_CACHE': home / 'specs',
           'BUNDLE_DISABLE_SHARED_GEMS': '1', 'BUNDLE_DISABLE_VERSION_CHECK': '1', 'BUNDLE_SILENCE_ROOT_WARNING': '1',
           'BUNDLE_RETRY': '2'}
    bundle = [shelf.tool, '-S', 'bundle']

    def download(package):
        # Resolve each gem on its own: the newest releases of a hundred unrelated gems rarely
        # share one consistent set of pins, and a project only ever adds a few of them.
        name = package['name']
        if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._-]*', name):
            raise RuntimeError('Invalid gem name')
        work = shelf.scratch('gem')
        (work / 'Gemfile').write_text('source "https://rubygems.org"\ngem ' + json.dumps(name) + '\n')
        local = dict(env, BUNDLE_GEMFILE=work / 'Gemfile')
        # Bundler resolves for this Ruby and this Mac, and skips prereleases unless nothing else exists.
        try:
            shelf.run(bundle + ['cache', '--no-install'], cwd=work, env=local)
        except RuntimeError as error:
            # Bundler wraps its messages; keep the whole sentence on the one line that is reported.
            raise RuntimeError(' '.join(str(error).split())) from None
        lock = (work / 'Gemfile.lock').read_text()
        found = re.search(r'^    ' + re.escape(name) + r' \(([^)\s-]+)', lock, re.MULTILINE | re.IGNORECASE)
        for gem in sorted((work / 'vendor/cache').glob('*.gem')):
            target = store / gem.name
            if not target.exists():
                shutil.move(str(gem), target)
        shutil.rmtree(work, ignore_errors=True)
        return found.group(1) if found else None

    shelf.each(download)
