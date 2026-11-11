"""PyPI wheels for the local interpreter, kept as a pip --find-links directory."""
import re
import shutil


def normal(name):
    return re.sub(r'[-_.]+', '-', name.split('[')[0]).lower()


def build(shelf):
    wheels = shelf.out / 'wheelhouse'
    wheels.mkdir(exist_ok=True)
    venv = shelf.scratch('venv')
    shelf.run([shelf.tool, '-I', '-m', 'venv', venv])
    pip = [venv / 'bin/python', '-I', '-m', 'pip', '--isolated', '--disable-pip-version-check']

    def download(package):
        # Resolve each package on its own: a hundred unrelated packages rarely share one
        # consistent set of pins, and a project only ever adds a few of them.
        fresh = shelf.scratch('wheels')
        shelf.run(pip + ['download', '--only-binary=:all:', '--dest', fresh, '--find-links', wheels, package['name']])
        version = None
        for wheel in fresh.iterdir():
            project, release = wheel.name.split('-')[:2]
            if normal(project) == normal(package['name']):
                version = release
            target = wheels / wheel.name
            if not target.exists():
                shutil.move(str(wheel), target)
        shutil.rmtree(fresh, ignore_errors=True)
        return version

    # Editing pyproject.toml offline needs this helper beside the packages.
    try:
        download({'name': 'tomlkit'})
    except Exception as error:
        print('tomlkit helper unavailable:', error, flush=True)
    shelf.each(download)
