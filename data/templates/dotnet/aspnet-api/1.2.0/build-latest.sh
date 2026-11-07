#!/bin/sh
set -eu
project=${1:?project folder}
tool=${DEVKIT_TOOLCHAIN:-dotnet}
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
trap 'exit 130' HUP INT TERM
export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_GENERATE_ASPNET_CERTIFICATE=false DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE=true
export DOTNET_EnableDiagnostics=0 DOTNET_CLI_USE_MSBUILD_SERVER=0 MSBUILDDISABLENODEREUSE=1
export DOTNET_CLI_HOME="$scratch/home" NUGET_PACKAGES="$scratch/packages" NUGET_HTTP_CACHE_PATH="$scratch/http-cache"
mkdir "$scratch/tests"
cp "$project/OfflineApi.csproj" "$project/global.json" "$project/NuGet.Config" "$scratch/"
cp "$project/tests/OfflineApi.Tests.csproj" "$scratch/tests/"
cd "$scratch"
python3 - <<'PYTHON'
from pathlib import Path
import re

for manifest in Path(".").rglob("*.csproj"):
    contents = re.sub(
        r'Version="\[(\d+)\.(\d+)(?:\.\d+)+\]"',
        lambda match: f'Version="{match[1]}.{match[2]}.*"',
        manifest.read_text(),
    )
    manifest.write_text(contents)
PYTHON
"$tool" restore tests/OfflineApi.Tests.csproj --use-lock-file --force-evaluate --source https://api.nuget.org/v3/index.json
python3 - <<'PYTHON'
from pathlib import Path
import json
import re
import shutil

feed = Path("nuget-packages")
feed.mkdir()
manifests = [Path("OfflineApi.csproj"), Path("tests/OfflineApi.Tests.csproj")]
for manifest in manifests:
    packages = json.loads(manifest.with_name("packages.lock.json").read_text())["dependencies"]["net10.0"]
    contents = re.sub(
        r'(PackageReference Include="([^"]+)" Version=")[^"]+"',
        lambda match: f'{match[1]}[{packages[match[2]]["resolved"]}]"',
        manifest.read_text(),
    )
    manifest.write_text(contents)
    for package, metadata in packages.items():
        if "resolved" not in metadata:
            continue
        package = package.lower()
        resolved = metadata["resolved"]
        archive = Path("packages") / package / resolved / f"{package}.{resolved}.nupkg"
        shutil.copyfile(archive, feed / archive.name)
PYTHON
export NUGET_PACKAGES="$scratch/locked-packages"
"$tool" restore tests/OfflineApi.Tests.csproj --use-lock-file --force-evaluate --configfile NuGet.Config -p:NuGetAudit=false -p:UseSharedCompilation=false
rm -rf "$project/nuget-packages"
mv nuget-packages "$project/"
cp OfflineApi.csproj packages.lock.json "$project/"
cp tests/OfflineApi.Tests.csproj tests/packages.lock.json "$project/tests/"
