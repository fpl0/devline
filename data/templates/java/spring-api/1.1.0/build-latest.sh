#!/bin/sh
set -eu
project=${1:?project folder}
tool=${DEVKIT_TOOLCHAIN:-mvn}
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
/usr/bin/curl -fLsS --max-time 25 https://repo.maven.apache.org/maven2/org/springframework/boot/spring-boot-starter-parent/maven-metadata.xml -o "$scratch/metadata.xml"
/usr/bin/curl -fLsS --max-time 25 https://repo.maven.apache.org/maven2/software/amazon/awssdk/s3/maven-metadata.xml -o "$scratch/s3.xml"
python3 - "$project/pom.xml" "$scratch/metadata.xml" "$scratch/s3.xml" <<'PYTHON'
from pathlib import Path
from xml.etree import ElementTree as ET
import re
import sys

manifest = Path(sys.argv[1])
versions = [n.text for n in ET.parse(sys.argv[2]).findall(".//version")]
compatible = [v for v in versions if v and re.fullmatch(r"4\.1\.\d+", v)]
latest = max(compatible, key=lambda version: tuple(map(int, version.split("."))))
contents = manifest.read_text()
contents = re.sub(r"(<artifactId>spring-boot-starter-parent</artifactId>\s*<version>)[^<]+", r"\g<1>" + latest, contents, count=1)
current = re.search(r"<aws.sdk.version>([0-9.]+)</aws.sdk.version>", contents)[1]
series = ".".join(current.split(".")[:2])
s3_versions = [n.text for n in ET.parse(sys.argv[3]).findall(".//version")]
s3_stable = [v for v in s3_versions if v and re.fullmatch(re.escape(series) + r"\.\d+", v)]
s3_latest = max([current, *s3_stable], key=lambda version: tuple(map(int, version.split("."))))
contents = contents.replace(f"<aws.sdk.version>{current}</aws.sdk.version>", f"<aws.sdk.version>{s3_latest}</aws.sdk.version>")
manifest.write_text(contents)
PYTHON
mkdir "$scratch/source"
cp -R "$project/src" "$project/pom.xml" "$project/settings-offline.xml" "$scratch/source/"
cd "$scratch/source"
"$tool" -B -ntp -s settings-offline.xml "-Dmaven.repo.local=$scratch/repository" verify
find "$scratch/repository" -type f \( -name '*.lastUpdated' -o -name '_remote.repositories' \) -delete
rm -rf "$project/maven-repository"
mv "$scratch/repository" "$project/maven-repository"
