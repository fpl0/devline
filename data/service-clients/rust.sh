#!/bin/sh
set -eu
. "$(dirname "$0")/common.sh"
cd "$project"
merge "$artifacts/cargo-home" "$HOME/cargo"
# The package shelf is one more registry cache. It is cloned beside the bundled one in the
# stage's disposable home; the project keeps only the crates `cargo vendor` copies for its lock.
if [ -n "$shelf" ]; then merge "$shelf/cargo-home" "$HOME/cargo"; fi
export CARGO_HOME="$HOME/cargo" CARGO_TARGET_DIR="$TMPDIR/cargo-target"
if [ "$offline" = true ]; then export CARGO_NET_OFFLINE=true; fi
mkdir -p .cargo
if [ -f .cargo/config.toml ]; then
    # Replace only Devline's registry-vendor source entries, keeping build and
    # target options. Other named/private sources remain package-manager-owned.
    /usr/bin/awk '/^\[/ { skip=($0 == "[source.crates-io]" || $0 == "[source.vendored-sources]") } !skip {print}' .cargo/config.toml > "$TMPDIR/cargo-config"
    /bin/cp "$TMPDIR/cargo-config" .cargo/config.toml
fi
add_package() {
    requirement=$(field "$package_prefix.existing_requirement")
    set -- "$package_name@${requirement:-$package_version}"
    alias=$(field "$package_prefix.existing_name")
    if [ -n "$alias" ] && [ "$alias" != "$package_name" ]; then set -- "$@" --rename "$alias"; fi
    if [ "$(field "$package_prefix.default_features")" = false ]; then set -- "$@" --no-default-features; fi
    n=$(count "$package_prefix.features"); k=0
    while [ "$k" -lt "$n" ]; do set -- "$@" --features "$(field "$package_prefix.features.$k")"; k=$((k+1)); done
    "$tool" add "$@"
}
packages
mkdir -p "$managed/rust"
"$tool" vendor --locked --versioned-dirs "$managed_relative/rust/vendor" > "$managed/rust-sources.toml"
cat "$managed/rust-sources.toml" >> .cargo/config.toml
