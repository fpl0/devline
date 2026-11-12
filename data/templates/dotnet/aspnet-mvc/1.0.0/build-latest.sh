#!/bin/sh
set -eu
project=${1:?project folder}
tool=${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-dotnet}}
export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_GENERATE_ASPNET_CERTIFICATE=false DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE=true
export DOTNET_EnableDiagnostics=0 DOTNET_CLI_USE_MSBUILD_SERVER=0 MSBUILDDISABLENODEREUSE=1

python3 - "$project" "$tool" <<'PY'
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

project = Path(sys.argv[1]).resolve()
tool = sys.argv[2]
manifest_name = "TaskBoard.csproj"
with tempfile.TemporaryDirectory(prefix="devline-api-refresh-") as temporary:
    stage = Path(temporary)
    for name in (manifest_name, "NuGet.Config", "global.json"):
        shutil.copyfile(project / name, stage / name)
    manifest = stage / manifest_name
    floated = re.sub(r'Version="\[(\d+)\.(\d+)(?:\.\d+)+\]"',
                     lambda match: f'Version="{match[1]}.{match[2]}.*"', manifest.read_text())
    manifest.write_text(floated)
    cache = stage / "packages"
    env = dict(os.environ, DOTNET_CLI_HOME=str(stage / "home"), NUGET_PACKAGES=str(cache),
               NUGET_HTTP_CACHE_PATH=str(stage / "http"))

    def restore(*options):
        subprocess.run([tool, "restore", manifest_name, "--use-lock-file", "--force-evaluate",
                        "-p:NuGetAudit=false", *options], cwd=stage, env=env, check=True)

    restore("--source", "https://api.nuget.org/v3/index.json")
    lock = stage / "packages.lock.json"
    packages = json.loads(lock.read_text())["dependencies"]["net10.0"]
    manifest.write_text(re.sub(
        r'(PackageReference Include="([^"]+)" Version=")[^"]+"',
        lambda match: f'{match[1]}[{packages[match[2]]["resolved"]}]"', floated,
    ))
    feed = stage / "nuget-packages"
    feed.mkdir()
    for name, entry in packages.items():
        package = name.lower()
        version = entry["resolved"]
        archive = cache / package / version / f"{package}.{version}.nupkg"
        shutil.copyfile(archive, feed / archive.name)

    env["NUGET_PACKAGES"] = str(stage / "locked-packages")
    restore("--configfile", "NuGet.Config")
    shutil.rmtree(project / "nuget-packages")
    shutil.copytree(feed, project / "nuget-packages")
    for name in (manifest_name, "packages.lock.json"):
        shutil.copyfile(stage / name, project / name)
PY
