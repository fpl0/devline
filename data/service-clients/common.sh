#!/bin/sh
# Values come only from Devline's validated recommendation catalog.
project=${1:?project}; selection=${2:?selection}; offline=${3:?offline}; tool=${4:?tool}
base=$(cd "$(dirname "$0")" && pwd)
field() { /usr/bin/plutil -extract "$1" raw -o - "$selection" 2>/dev/null || :; }
count() { /usr/bin/plutil -extract "$1" raw -expect array -o - "$selection" 2>/dev/null || printf '0'; }
catalog=$(field catalog_dir)
stack=$(field stack)
artifacts="$catalog/service-clients/artifacts/$stack"
mkdir -p "$project/.devline-offline"
export DEVLINE_OFFLINE="$offline" PATH="$(dirname "$tool"):$PATH"
merge() {
    [ -d "$1" ] || return 0
    mkdir -p "$2"
    /bin/cp -cR "$1/." "$2/" 2>/dev/null || /bin/cp -R "$1/." "$2/"
}
packages() {
    i=0
    while [ "$i" -lt "$(count integrations)" ]; do
        j=0
        while [ "$j" -lt "$(count "integrations.$i.packages")" ]; do
            package_prefix="integrations.$i.packages.$j"
            package_name=$(field "$package_prefix.name")
            package_version=$(field "$package_prefix.version")
            add_package
            j=$((j+1))
        done
        i=$((i+1))
    done
}
