#!/bin/sh
# Values come only from Devline's validated recommendation catalog.
project=${1:?project}; selection=${2:?selection}; offline=${3:?offline}; tool=${4:?tool}
field() { /usr/bin/plutil -extract "$1" raw -o - "$selection" 2>/dev/null || :; }
count() { /usr/bin/plutil -extract "$1" raw -expect array -o - "$selection" 2>/dev/null || printf '0'; }
catalog=$(field catalog_dir)
stack=$(field stack)
artifacts="$catalog/service-clients/artifacts/$stack"
# An optional package shelf in the library: one more local source beside the bundled packs.
shelf=$(field shelf_dir)
managed_relative=${DEVLINE_PROJECT_CACHE:-.devline-offline}
managed="$project/$managed_relative"
mkdir -p "$managed"
export DEVLINE_OFFLINE="$offline" PATH="$(dirname "$tool"):$PATH"
merge() {
    [ -d "$1" ] || return 0
    mkdir -p "$2"
    /bin/cp -cR "$1/." "$2/" 2>/dev/null || /bin/cp -R "$1/." "$2/"
}
packages() {
    integration_count=$(count integrations)
    i=0
    while [ "$i" -lt "$integration_count" ]; do
        package_count=$(count "integrations.$i.packages")
        j=0
        while [ "$j" -lt "$package_count" ]; do
            package_prefix="integrations.$i.packages.$j"
            package_name=$(field "$package_prefix.name")
            package_version=$(field "$package_prefix.version")
            add_package
            j=$((j+1))
        done
        i=$((i+1))
    done
}
