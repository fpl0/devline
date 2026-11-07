#!/bin/sh
set -eu
project=${1:?project folder}
tool=${DEVKIT_TOOLCHAIN:-mvn}
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
trap 'exit 130' HUP INT TERM
python3 - "$project/pom.xml" <<'PYTHON'
from pathlib import Path
from urllib.request import urlopen
from xml.etree import ElementTree as ET
import re
import sys

manifest = Path(sys.argv[1])
contents = manifest.read_text()
for name, coordinate in {
    "picocli.version": "info/picocli/picocli",
    "jackson.version": "tools/jackson/core/jackson-databind",
    "junit.version": "org/junit/jupiter/junit-jupiter",
}.items():
    pattern = rf"(<{re.escape(name)}>)([0-9.]+)(</{re.escape(name)}>)"
    current = re.search(pattern, contents)[2]
    series = ".".join(current.split(".")[:2])
    with urlopen(f"https://repo.maven.apache.org/maven2/{coordinate}/maven-metadata.xml", timeout=25) as source:
        candidates = [tag.text for tag in ET.parse(source).findall(".//version")]
    stable = [version for version in candidates if version and re.fullmatch(re.escape(series) + r"\.\d+", version)]
    latest = max([current, *stable], key=lambda version: tuple(map(int, version.split("."))))
    contents = re.sub(pattern, lambda match: match[1] + latest + match[3], contents, count=1)
manifest.write_text(contents)
PYTHON
cp -R "$project/src" "$project/pom.xml" "$project/settings-offline.xml" "$scratch/"
cd "$scratch"
"$tool" -B -ntp -s settings-offline.xml "-Dmaven.repo.local=$scratch/repository" verify
find "$scratch/repository" -type f \( -name '*.lastUpdated' -o -name '_remote.repositories' \) -delete
rm -rf "$project/maven-repository"
mv "$scratch/repository" "$project/maven-repository"
