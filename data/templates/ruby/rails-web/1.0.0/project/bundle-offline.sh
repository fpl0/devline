#!/bin/sh
set -eu
project=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
cd "$project"
ruby=${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-ruby}}
action=${1:?use install or exec}
shift

case "$ruby" in
  /*) export PATH="$(dirname "$ruby"):$PATH" ;;
esac
"$ruby" -e 'abort "Requires Ruby 4.0.9 or a newer 4.0 patch on Apple Silicon macOS" unless Gem::Requirement.new("~> 4.0.0", ">= 4.0.9").satisfied_by?(Gem::Version.new(RUBY_VERSION)) && RUBY_PLATFORM.start_with?("arm64-darwin")'
bundler=$(awk '/^BUNDLED WITH$/{getline; print $1; exit}' Gemfile.lock)
"$ruby" -rrubygems -e 'abort "Install Bundler #{ARGV[0]} before going offline" if Gem::Specification.find_all_by_name("bundler", ARGV[0]).empty?' "$bundler"
export BUNDLE_GEMFILE="$project/Gemfile" BUNDLE_IGNORE_CONFIG=1 BUNDLE_FROZEN=1 BUNDLE_DISABLE_SHARED_GEMS=1 BUNDLE_NO_CACHE=1
export BUNDLE_PATH=${DEVLINE_GEM_PATH:-"$project/.bundle-gems"}
export BUNDLE_USER_HOME=${DEVLINE_BUNDLER_HOME:-"$project/.bundler"}
export BUNDLE_APP_CONFIG="$BUNDLE_USER_HOME/config"

case "$action" in
  install)
    developer=$(/usr/bin/xcode-select -p 2>/dev/null) || { echo 'Install Xcode Command Line Tools before going offline (xcode-select --install).' >&2; exit 1; }
    sdk="$developer/SDKs/MacOSX.sdk"
    tools="$developer/usr/bin"
    if [ -d "$developer/Toolchains/XcodeDefault.xctoolchain/usr/bin" ]; then
      sdk="$developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk"
      tools="$developer/Toolchains/XcodeDefault.xctoolchain/usr/bin:$tools"
    fi
    export SDKROOT="${SDKROOT:-$sdk}" PATH="$tools:$PATH"
    exec "$ruby" -S bundle "_${bundler}_" install --local --jobs 2
    ;;
  exec) exec "$ruby" -S bundle "_${bundler}_" exec "$@" ;;
  *) echo 'Use bundle-offline.sh install or bundle-offline.sh exec COMMAND.' >&2; exit 2 ;;
esac
