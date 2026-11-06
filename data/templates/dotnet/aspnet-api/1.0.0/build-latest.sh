#!/bin/sh
set -eu
project=${1:?project folder}
tool=${DEVKIT_TOOLCHAIN:-dotnet}
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_GENERATE_ASPNET_CERTIFICATE=false DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE=true
export DOTNET_CLI_HOME="$scratch/home" NUGET_PACKAGES="$scratch/packages" NUGET_HTTP_CACHE_PATH="$scratch/http-cache"
cp "$project/OfflineApi.csproj" "$project/global.json" "$project/NuGet.Config" "$scratch/"
cd "$scratch"
sed -i '' 's/Version="\[[0-9.]*\]"/Version="10.0.*"/' OfflineApi.csproj
"$tool" restore --use-lock-file --force-evaluate --source https://api.nuget.org/v3/index.json
python3 - "$project" <<'PYTHON'
from pathlib import Path
import json
import re
import shutil
import sys

project = Path(sys.argv[1])
packages = json.loads(Path("packages.lock.json").read_text())["dependencies"]["net10.0"]
version = packages["Microsoft.EntityFrameworkCore.Sqlite"]["resolved"]
manifest = Path("OfflineApi.csproj")
manifest.write_text(re.sub(r'Version="10\.0\.\*"', f'Version="[{version}]"', manifest.read_text()))
feed = project / "nuget-packages"
shutil.rmtree(feed)
feed.mkdir()
for package, metadata in packages.items():
    package = package.lower()
    resolved = metadata["resolved"]
    archive = Path("packages") / package / resolved / f"{package}.{resolved}.nupkg"
    shutil.copyfile(archive, feed / archive.name)
(project / "OfflineApi.csproj").write_bytes(manifest.read_bytes())
Path("nuget-packages").symlink_to(feed, target_is_directory=True)
PYTHON
export NUGET_PACKAGES="$scratch/locked-packages"
"$tool" restore --use-lock-file --force-evaluate --configfile NuGet.Config -p:NuGetAudit=false
cp packages.lock.json "$project/"
