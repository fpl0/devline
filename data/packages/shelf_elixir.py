"""Hex packages kept as a Hex home: the registry cache plus package tarballs, with Hex and rebar3."""
import re
import shutil


def build(shelf):
    cache = shelf.out / 'cache'
    cache.mkdir(exist_ok=True)
    # Start from the bundled client pack. Its Hex and rebar3 run this download, and its registry
    # entries make the shelf's registry cache a superset of what bundled clients were locked with.
    sources = [shelf.catalog / 'service-clients/artifacts/elixir/cache']
    sources += [tools.parent for tools in sorted((shelf.catalog / 'templates/elixir').glob('*/*/project/vendor-tools'))]
    for part, names in (('tools', ('tools', 'vendor-tools')), ('hex', ('hex',))):
        found = next((source / name for source in sources for name in names if (source / name).is_dir()), None)
        if not (cache / part).exists() and found:
            shutil.copytree(found, cache / part)
    tools = cache / 'tools'
    if not (tools / 'rebar3').is_file() or not (tools / 'archives').is_dir():
        raise RuntimeError('The bundled Hex and rebar3 tools are missing from the catalog')
    (cache / 'hex').mkdir(exist_ok=True)
    env = {'MIX_HOME': shelf.scratch('mix'), 'HEX_HOME': cache / 'hex', 'MIX_ARCHIVES': tools / 'archives',
           'MIX_REBAR3': tools / 'rebar3', 'MIX_OS_CONCURRENCY_LOCK': '0', 'HEX_OFFLINE': '0', 'MIX_ENV': 'prod'}

    def download(package):
        name = package['name']
        if not re.fullmatch(r'[a-z][a-z0-9_]*', name):
            raise RuntimeError('Invalid Hex package name')

        def resolve(requirement):
            # Resolve each package on its own: Hex picks the newest release and its dependencies,
            # and keeps every tarball and registry entry it used in the shelf's Hex home.
            work = shelf.scratch('package')
            (work / 'mix.exs').write_text(
                'defmodule Shelf.MixProject do\n  use Mix.Project\n'
                f'  def project, do: [app: :shelf, version: "1.0.0", deps: [{{:{name}, "{requirement}"}}]]\nend\n')
            local = dict(env, MIX_DEPS_PATH=work / 'deps', MIX_BUILD_PATH=work / '_build')
            try:
                shelf.run([shelf.tool, '-S', 'mix', 'deps.get'], cwd=work, env=local)
                lock = (work / 'mix.lock').read_text()
            finally:
                shutil.rmtree(work, ignore_errors=True)
            found = re.search(r'\{:hex, :' + re.escape(name) + r', "([^"]+)"', lock)
            return found.group(1) if found else None

        try:
            return resolve('>= 0.0.0')
        except RuntimeError as error:
            reason = ' '.join(str(error).split())
            if 'No package with name' in reason:
                raise RuntimeError('Not found on Hex') from None
            try:
                # Only prereleases exist: a requirement that names one lets Hex choose them.
                return resolve('>= 0.0.0-0')
            except RuntimeError:
                raise RuntimeError(reason) from None

    shelf.each(download)
