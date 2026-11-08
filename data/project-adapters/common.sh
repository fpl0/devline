# Shared conventions for project adapter entry points.

export_starter_toolchain() {
    export DEVLINE_TOOLCHAIN="$1"
    # Published starter scripts still read this alias.
    export DEVKIT_TOOLCHAIN="$DEVLINE_TOOLCHAIN"
}

offline_requested() {
    case "${DEVLINE_OFFLINE:-}" in
        true|1) return 0 ;;
        *) return 1 ;;
    esac
}

find_starter_test() {
    [ -f .devline/origin.json ] || [ -f .devkit-origin.json ] || return 1

    script=./.devline/scripts/test-offline.sh
    [ -f "$script" ] || script=./test-offline.sh
    [ -x "$script" ] || return 1
    printf '%s\n' "$script"
}
