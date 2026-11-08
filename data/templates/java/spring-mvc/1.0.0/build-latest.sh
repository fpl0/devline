#!/bin/sh
set -eu
project=${1:?project folder}
tool=${DEVKIT_TOOLCHAIN:-mvn}
scratch=$(mktemp -d)
trap 'rm -r "$scratch"' EXIT
trap 'exit 130' HUP INT TERM
/usr/bin/curl -fLsS --max-time 25 https://repo.maven.apache.org/maven2/org/springframework/boot/spring-boot-starter-parent/maven-metadata.xml -o "$scratch/metadata.xml"
python3 - "$project/pom.xml" "$scratch/metadata.xml" <<'PYTHON'
from pathlib import Path
from xml.etree import ElementTree as ET
import re
import sys

manifest = Path(sys.argv[1])
contents = manifest.read_text()
pattern = r"(<artifactId>spring-boot-starter-parent</artifactId>\s*<version>)([0-9.]+)"
current = re.search(pattern, contents)[2]
series = ".".join(current.split(".")[:2])
versions = [node.text for node in ET.parse(sys.argv[2]).findall(".//version")]
stable = [version for version in versions if version and re.fullmatch(re.escape(series) + r"\.\d+", version)]
latest = max([current, *stable], key=lambda version: tuple(map(int, version.split("."))))
manifest.write_text(re.sub(pattern, lambda match: match[1] + latest, contents, count=1))
PYTHON
mkdir "$scratch/source"
cp -R "$project/src" "$project/pom.xml" "$project/settings-offline.xml" "$scratch/source/"
cd "$scratch/source"
"$tool" -B -ntp -s settings-offline.xml "-Dmaven.repo.local=$scratch/repository" clean verify
find "$scratch/repository" -type f \( -name '*.lastUpdated' -o -name '_remote.repositories' -o -name 'resolver-status.properties' \) -delete
if [ -d "$project/maven-repository" ]; then mv "$project/maven-repository" "$scratch/previous"; fi
mv "$scratch/repository" "$project/maven-repository"
