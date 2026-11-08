#!/bin/sh
set -eu
action=${1:?action}; project=${2:?project folder}; cache=${3:?managed cache}; tool=${4:?Cargo executable}
shift 4
cd "$project"
[ -f Cargo.lock ] || { echo 'Generate and review Cargo.lock before preparing a reproducible Rust project.' >&2; exit 1; }
export PATH="$(dirname "$tool"):$PATH"
vendor_path="${cache#"$project"/}/rust/vendor"

cargo_with_sources() {
    # Cargo resolves paths in a config file two levels above that file. The
    # nested cache needs a command-line path resolved from the project root.
    if [ "$cache" = "$project/.devline/cache" ]; then
        set -- --config "source.vendored-sources.directory=\"$vendor_path\"" "$@"
    fi
    exec "$tool" --config "$cache/rust-sources.toml" "$@"
}

case "$action" in
    prepare)
        mkdir -p "$cache/rust"
        export CARGO_HOME="$HOME/cargo" CARGO_TARGET_DIR="$HOME/target"
        "$tool" vendor --locked --versioned-dirs "$vendor_path" > "$cache/rust-sources.toml"
        ;;
    verify)
        export CARGO_HOME="$HOME/cargo" CARGO_TARGET_DIR="$HOME/target" CARGO_NET_OFFLINE=true
        if [ -s "$cache/rust-sources.toml" ]; then
            cargo_with_sources test --frozen
        fi
        exec "$tool" test --frozen
        ;;
    run)
        export CARGO_NET_OFFLINE=true
        if [ "${1:-}" = cargo ] && [ -s "$cache/rust-sources.toml" ]; then
            shift
            cargo_with_sources "$@"
        fi
        exec "$@"
        ;;
    *) exit 2 ;;
esac
