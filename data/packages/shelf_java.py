"""Maven Central artifacts kept as a repository that Maven reads through a file:// mirror."""
import os
import re
import shutil
import subprocess
import sys
import xml.etree.ElementTree as ET
from xml.sax.saxutils import escape

CENTRAL = 'https://repo.maven.apache.org/maven2/'
# Pinned so every download resolves the same way; it only ever runs here, while connected.
RESOLVE = 'org.apache.maven.plugins:maven-dependency-plugin:3.11.0:resolve'
PRERELEASE = re.compile(r'(?<![a-z])(alpha|beta|milestone|snapshot|preview|pre|dev|(rc|cr|ea)(?![a-z])|m(?=\d)|[ab](?=\d))', re.IGNORECASE)


def order(version):
    """Sort key: numbers compare as numbers and outrank words; guava's -jre beats -android."""
    return [(2, int(part)) if part.isdigit() else (0 if part.lower() == 'android' else 1, part.lower())
            for part in re.findall(r'\d+|[A-Za-z]+', version)]


def newest(versions):
    """The highest stable version. Maven's own <release> is often a release candidate."""
    stable = [version for version in versions if not PRERELEASE.search(version)]
    # Some old artifacts also carry date stamps (20030203.000550) that would outrank real releases.
    dated = [version for version in stable if re.match(r'\d{8}(\D|$)', version)]
    plain = [version for version in stable if version not in dated]
    return max(plain or dated or versions, key=order)


def build(shelf):
    repository = shelf.out / 'repository'
    repository.mkdir(exist_ok=True)
    # Java ignores $HOME; without this Maven would write to the real ~/.m2.
    options = f'-Duser.home="{os.environ["HOME"]}" -Djava.io.tmpdir="{os.environ["TMPDIR"]}"'
    env = dict(shelf.env, JDK_JAVA_OPTIONS=(shelf.env.get('JDK_JAVA_OPTIONS', '') + ' ' + options).strip())

    # Maven's own settings file mentions ${user.home} in a comment and cannot be parsed when that
    # path contains "--", so it is replaced with an empty one.
    global_settings = shelf.out / 'global-settings.xml'
    global_settings.write_text('<settings xmlns="http://maven.apache.org/SETTINGS/1.0.0"/>\n')

    def resolve(folder):
        done = subprocess.run([shelf.tool, '-gs', str(global_settings), '-B', '-ntp', '-Dmaven.repo.local=' + str(repository), RESOLVE],
                              cwd=folder, env=env, text=True, errors='replace',
                              stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        if done.returncode != 0:
            errors = [line[7:].strip() for line in done.stdout.splitlines() if line.startswith('[ERROR]') and line[7:].strip()]
            reason = errors[0] if errors else f'Maven exited with {done.returncode}'
            raise RuntimeError(reason.split(' -> [Help')[0])

    def download(package):
        name = package['name']
        if not re.fullmatch(r'[A-Za-z0-9._-]+:[A-Za-z0-9._-]+', name):
            raise RuntimeError('Java packages are named groupId:artifactId')
        group, artifact = name.split(':')
        url = CENTRAL + group.replace('.', '/') + '/' + artifact + '/maven-metadata.xml'
        try:
            listing = shelf.run(['/usr/bin/curl', '-fsSL', '--retry', '2', '--max-time', '60', url], capture=True)
        except RuntimeError as error:
            raise RuntimeError('Not found on Maven Central' if '404' in str(error) else ' '.join(str(error).split())) from None
        versions = [item.text.strip() for item in ET.fromstring(listing).iter('version') if item.text and item.text.strip()]
        if not versions:
            raise RuntimeError('Maven Central lists no versions')
        version = newest(versions)
        # Resolve each artifact on its own: its own dependency closure, at the versions it asks for.
        work = shelf.scratch('artifact')
        (work / 'pom.xml').write_text(
            '<project xmlns="http://maven.apache.org/POM/4.0.0"><modelVersion>4.0.0</modelVersion>'
            '<groupId>devline.local</groupId><artifactId>shelf</artifactId><version>1</version>'
            f'<dependencies><dependency><groupId>{escape(group)}</groupId><artifactId>{escape(artifact)}</artifactId>'
            f'<version>{escape(version)}</version></dependency></dependencies></project>\n')
        resolve(work)
        shutil.rmtree(work, ignore_errors=True)
        return version

    shelf.each(download)
    # A starter's parent or BOM pins the versions of shared dependencies. Resolve every package
    # inside each bundled starter too, so those pinned versions are on the shelf as well.
    sys.path.insert(0, str(shelf.catalog / 'service-clients'))
    from manifest_edits import add_maven_dependencies
    for manifest in shelf.starter_files('*/*/project/pom.xml'):
        label = manifest.parent.parent.parent.name
        try:
            seed = shelf.scratch('starter')
            shutil.copy(manifest, seed / 'pom.xml')
            add_maven_dependencies(seed / 'pom.xml', {entry['name']: entry for entry in shelf.resolved})
            resolve(seed)
        except Exception as error:
            print(f'starter versions skipped: {label}: {str(error).splitlines()[-1][:200]}', flush=True)
    # Maven treats the shelf as a remote repository. Drop failed-lookup markers and list the
    # versions that are really here, so version ranges resolve to something on the shelf.
    present = {}
    for folder, _names, files in os.walk(repository):
        for name in files:
            path = os.path.join(folder, name)
            if name.endswith('.lastUpdated'):
                os.unlink(path)
            elif name.endswith('.pom'):
                version, artifact = os.path.basename(folder), os.path.basename(os.path.dirname(folder))
                if name == f'{artifact}-{version}.pom':
                    present.setdefault(os.path.dirname(folder), []).append(version)
    for folder, versions in present.items():
        versions.sort(key=order)
        group = os.path.relpath(os.path.dirname(folder), repository).replace(os.sep, '.')
        with open(os.path.join(folder, 'maven-metadata.xml'), 'w') as index:
            index.write(f'<metadata><groupId>{escape(group)}</groupId><artifactId>{escape(os.path.basename(folder))}</artifactId>'
                        f'<versioning><latest>{escape(versions[-1])}</latest><release>{escape(versions[-1])}</release><versions>'
                        + ''.join(f'<version>{escape(version)}</version>' for version in versions)
                        + '</versions></versioning></metadata>\n')
