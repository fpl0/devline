#!/bin/sh
set -eu
. "$(dirname "$0")/common.sh"
cd "$project"
cache="$project/.devline-offline/go/module-cache"
merge "$artifacts/module-cache" "$cache"
chmod -R u+w "$cache"
export GOMODCACHE="$cache" GOCACHE="$TMPDIR/go-build" GOTOOLCHAIN=local GOWORK=off GOFLAGS=-mod=mod
if [ "$offline" = true ]; then export GOPROXY=off GOSUMDB=off; fi
imports="$TMPDIR/client-imports"
if [ -f devline_clients/dependencies.go ]; then
    head -1 devline_clients/dependencies.go | /usr/bin/grep -q '^// Devline selected service clients' || { echo 'devline_clients/dependencies.go already exists and is not managed by Devline.' >&2; exit 1; }
    /usr/bin/awk '/^_ / { print }' devline_clients/dependencies.go > "$imports"
else : > "$imports"; fi
add_package() {
    "$tool" get "$package_name@$package_version"
    n=$(count "$package_prefix.imports")
    if [ "$n" -eq 0 ]; then printf '_ "%s"\n' "$package_name" >> "$imports"
    else k=0; while [ "$k" -lt "$n" ]; do printf '_ "%s"\n' "$(field "$package_prefix.imports.$k")" >> "$imports"; k=$((k+1)); done; fi
}
packages
mkdir -p devline_clients
{ printf '%s\n' '// Devline selected service clients: keep their APIs available in the offline vendor tree.' 'package devlineclients' 'import ('; LC_ALL=C sort -u "$imports"; printf ')\n'; } > devline_clients/dependencies.go
"$tool" mod tidy
"$tool" mod vendor
# The generic adapter verifies directly against vendor, without shared caches.
