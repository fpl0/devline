#!/bin/sh
set -eu
project=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
cd "$project"
ruby=${DEVKIT_TOOLCHAIN:-ruby}
action=${1:?use install or exec}
shift
"$ruby" -e 'abort "Requires Ruby 4.0 on Apple Silicon macOS" unless RUBY_VERSION.start_with?("4.0.") && RUBY_PLATFORM.start_with?("arm64-darwin")'
bundler=$(awk '/^BUNDLED WITH$/{getline; print $1; exit}' Gemfile.lock)
"$ruby" -rrubygems -e 'abort "Install Bundler #{ARGV[0]} before going offline" if Gem::Specification.find_all_by_name("bundler", ARGV[0]).empty?' "$bundler"
export BUNDLE_GEMFILE="$project/Gemfile" BUNDLE_IGNORE_CONFIG=1 BUNDLE_FROZEN=1 BUNDLE_DISABLE_SHARED_GEMS=1
export BUNDLE_PATH=${DEVLINE_GEM_PATH:-"$project/.bundle-gems"}
export BUNDLE_USER_HOME=${DEVLINE_BUNDLER_HOME:-"$project/.bundler"}
case "$action" in
  install)
    xcode-select -p >/dev/null 2>&1 || { echo 'Install Xcode Command Line Tools before going offline (xcode-select --install).' >&2; exit 1; }
    if grep -q '^    trilogy (' Gemfile.lock; then
      openssl=${DEVLINE_OPENSSL_DIR:-/opt/homebrew/opt/openssl@3}
      if ! grep -Eq '^#[[:space:]]*define[[:space:]]+OPENSSL_VERSION_MAJOR[[:space:]]+3' "$openssl/include/openssl/opensslv.h" 2>/dev/null ||
        [ ! -f "$openssl/lib/libssl.dylib" ]; then
        echo 'Install OpenSSL 3 development files before going offline. Use brew install openssl@3 or set DEVLINE_OPENSSL_DIR to its prefix.' >&2
        exit 1
      fi
      export BUNDLE_BUILD__OPENSSL="--with-openssl-dir=$openssl" BUNDLE_BUILD__TRILOGY="--with-openssl-dir=$openssl"
    fi
    exec "$ruby" -S bundle "_${bundler}_" install --local --jobs 4
    ;;
  exec) exec "$ruby" -S bundle "_${bundler}_" exec "$@" ;;
  *) echo 'Use bundle-offline.sh install or bundle-offline.sh exec COMMAND.' >&2; exit 2 ;;
esac
