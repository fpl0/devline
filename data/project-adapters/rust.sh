#!/bin/sh
set -eu
action=${1:?action}; project=${2:?project folder}; cache=${3:?managed cache}; tool=${4:?Cargo executable}
shift 4
cd "$project"
[ -f Cargo.lock ] || { echo 'Generate and review Cargo.lock before preparing a reproducible Rust project.' >&2; exit 1; }
export PATH="$(dirname "$tool"):$PATH"
case "$action" in
    prepare)
        mkdir -p "$cache/rust"
        export CARGO_HOME="$HOME/cargo" CARGO_TARGET_DIR="$HOME/target"
        "$tool" vendor --locked --versioned-dirs .devline-offline/rust/vendor > "$cache/rust-sources.toml"
        ;;
    verify)
        export CARGO_HOME="$HOME/cargo" CARGO_TARGET_DIR="$HOME/target" CARGO_NET_OFFLINE=true
        if [ -s "$cache/rust-sources.toml" ]; then
            exec "$tool" --config "$cache/rust-sources.toml" test --frozen
        fi
        exec "$tool" test --frozen
        ;;
    run)
        export CARGO_NET_OFFLINE=true
        if [ "${1:-}" = cargo ] && [ -s "$cache/rust-sources.toml" ]; then
            shift
            exec "$tool" --config "$cache/rust-sources.toml" "$@"
        fi
        exec "$@"
        ;;
    *) exit 2 ;;
esac
