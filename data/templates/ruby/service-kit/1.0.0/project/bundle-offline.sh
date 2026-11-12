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
    developer=$(/usr/bin/xcode-select -p 2>/dev/null) || { echo 'Install Xcode Command Line Tools before going offline (xcode-select --install).' >&2; exit 1; }
    # Invoke the selected tools directly so Xcode shims cannot write outside the test sandbox.
    sdk="$developer/SDKs/MacOSX.sdk"
    tools="$developer/usr/bin"
    if [ -d "$developer/Toolchains/XcodeDefault.xctoolchain/usr/bin" ]; then
      sdk="$developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk"
      tools="$developer/Toolchains/XcodeDefault.xctoolchain/usr/bin:$tools"
    fi
    export SDKROOT="${SDKROOT:-$sdk}" PATH="$tools:$PATH"
    trilogy=$(awk '/^    trilogy \(/{gsub(/[()]/,"",$2); print $2; exit}' Gemfile.lock)
    if [ -n "$trilogy" ]; then
      openssl=${DEVLINE_OPENSSL_DIR:-/opt/homebrew/opt/openssl@3}
      if ! grep -Eq '^#[[:space:]]*define[[:space:]]+OPENSSL_VERSION_MAJOR[[:space:]]+3' "$openssl/include/openssl/opensslv.h" 2>/dev/null ||
        [ ! -f "$openssl/lib/libssl.dylib" ]; then
        echo 'Install OpenSSL 3 development files before going offline. Use brew install openssl@3 or set DEVLINE_OPENSSL_DIR to its prefix.' >&2
        exit 1
      fi
      export BUNDLE_BUILD__OPENSSL="--with-openssl-dir=$openssl" BUNDLE_BUILD__TRILOGY="--with-openssl-dir=$openssl"
      # Trilogy's extconf passes an unquoted -I flag. Clang's include path preserves spaces.
      ruby_abi=$("$ruby" -rrbconfig -e 'print RbConfig::CONFIG.fetch("ruby_version")')
      export C_INCLUDE_PATH="$BUNDLE_PATH/ruby/$ruby_abi/gems/trilogy-$trilogy/ext/trilogy-ruby/inc${C_INCLUDE_PATH:+:$C_INCLUDE_PATH}"
    fi
    exec "$ruby" -S bundle "_${bundler}_" install --local --jobs 2
    ;;
  exec) exec "$ruby" -S bundle "_${bundler}_" exec "$@" ;;
  *) echo 'Use bundle-offline.sh install or bundle-offline.sh exec COMMAND.' >&2; exit 2 ;;
esac
