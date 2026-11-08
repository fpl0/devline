#!/bin/sh
set -eu
mode=${1:?prepare, verify, or run}
project=${2:?staged project}
cache=${3:?staged cache}
ruby=${4:?Ruby executable}
shift 4
adapters=$(cd "$(dirname "$0")" && pwd)
. "$adapters/common.sh"
cd "$project"
export PATH="$(dirname "$ruby"):$PATH"
if [ "$mode" = run ] && [ ! -f "$cache/Gemfile.lock" ] && { [ -f "$project/.devline/origin.json" ] || [ -f "$project/.devkit-origin.json" ]; }; then
  if [ ! -f "$project/Gemfile" ]; then
    [ "$#" -gt 0 ] || { echo 'Choose a command to run.' >&2; exit 2; }
    exec "$@"
  fi
  cache="$project"
fi
scratch=$(mktemp -d "${TMPDIR:-/tmp}/devline-ruby.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
export BUNDLE_GEMFILE="$cache/Gemfile" BUNDLE_IGNORE_CONFIG=1
export BUNDLE_USER_HOME="$scratch/bundler" BUNDLE_PATH="$scratch/gems" BUNDLE_DISABLE_SHARED_GEMS=1
export BUNDLE_DISABLE_VERSION_CHECK=1 BUNDLE_APP_CONFIG="$scratch/app-config"

bundler_version() {
  version=
  if [ -f "$cache/Gemfile.lock" ]; then
    version=$(awk '/^BUNDLED WITH$/{getline; print $1; exit}' "$cache/Gemfile.lock")
  fi
  if [ -z "$version" ]; then version=$("$ruby" -rbundler -e 'puts Bundler::VERSION'); fi
  "$ruby" -rrubygems -e 'abort "Install Bundler #{ARGV[0]} for this Ruby before preparing the project" if Gem::Specification.find_all_by_name("bundler", ARGV[0]).empty?' "$version"
  export BUNDLER_VERSION="$version"
}

bundle() {
  "$ruby" -S bundle "_${BUNDLER_VERSION}_" "$@"
}

registry_lock() {
  if grep -Eq '^(GIT|PATH)$|^[[:space:]]+remote:[[:space:]]+https?://[^/[:space:]]*@' "$cache/Gemfile.lock"; then
    echo 'Prepare local path, Git, and credential-bearing Gemfile sources separately. Devline caches registry gems.' >&2
    exit 1
  fi
}

check_syntax() {
  "$ruby" -e '
    files = Dir["*.rb", "app/**/*.rb", "lib/**/*.rb", "test/**/*.rb"].uniq.sort
    files.each { |file| RubyVM::InstructionSequence.compile_file(file) }
    puts "Offline gem installation verified. Checked syntax for #{files.length} Ruby files."
  '
}

install_gems() {
  bundle install --local --jobs 4
  bundle check
}

case "$mode" in
  prepare)
    if [ ! -f Gemfile ]; then
      echo 'Add or export a Gemfile for this project before preparing its gem cache.' >&2
      exit 1
    fi
    if grep -Eq '^[[:space:]]*(gemspec|eval_gemfile)([[:space:](]|$)|\b(git|github|path):' Gemfile; then
      echo 'This Gemfile uses local or Git dependencies. Prepare an equivalent registry Gemfile or a project-specific local bundle.' >&2
      exit 1
    fi
    mkdir -p "$cache"
    cp Gemfile "$cache/Gemfile"
    if [ -d "$project/vendor/cache" ]; then
      mkdir -p "$cache/vendor/cache"
      cp "$project"/vendor/cache/*.gem "$cache/vendor/cache/"
    fi
    if [ -f .ruby-version ]; then cp .ruby-version "$cache/.ruby-version"; fi
    if [ -f Gemfile.lock ]; then
      cp Gemfile.lock "$cache/Gemfile.lock"
      registry_lock
    fi
    bundler_version
    platform=$("$ruby" -rrubygems -e 'puts Gem::Platform.local')
    if offline_requested; then
      "$ruby" "$adapters/../service-clients/lock-ruby-offline.rb"
    else
      bundle lock --add-platform "$platform" --add-checksums
    fi
    registry_lock
    if [ -f Gemfile.lock ]; then
      "$ruby" -rbundler - "$project/Gemfile.lock" "$cache/Gemfile.lock" <<'RUBY'
before, after = ARGV.map do |path|
  Bundler::LockfileParser.new(File.read(path)).specs.group_by(&:name)
end
changed = after.keys.intersection(before.keys).reject do |name|
  after.fetch(name).all? { |spec| before.fetch(name).any? { |old| old.version == spec.version } }
end
abort "Pinned gems changed while adding the current platform: #{changed.join(', ')}" unless changed.empty?
RUBY
    fi
    if ! offline_requested; then bundle cache --no-install; fi
    ;;
  verify|run)
    if [ ! -f "$cache/Gemfile" ] || [ ! -f "$cache/Gemfile.lock" ] || [ ! -d "$cache/vendor/cache" ]; then
      echo 'Prepare this project’s Ruby dependencies before running offline.' >&2
      exit 1
    fi
    registry_lock
    bundler_version
    export BUNDLE_FROZEN=1 BUNDLE_NO_CACHE=1
    if "$ruby" -rrubygems/package -e 'exit(!Dir.glob(ARGV[0]).any? { |file| spec = Gem::Package.new(file).spec; spec.platform == "ruby" && !spec.extensions.empty? })' "$cache/vendor/cache/*.gem"; then
      developer=$(/usr/bin/xcode-select -p 2>/dev/null) || { echo 'Install Xcode Command Line Tools before preparing these native gems (xcode-select --install).' >&2; exit 1; }
      # Invoke the selected tools directly so Xcode shims cannot write outside the test sandbox.
      sdk="$developer/SDKs/MacOSX.sdk"
      tools="$developer/usr/bin"
      if [ -d "$developer/Toolchains/XcodeDefault.xctoolchain/usr/bin" ]; then
        sdk="$developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk"
        tools="$developer/Toolchains/XcodeDefault.xctoolchain/usr/bin:$tools"
      fi
      export SDKROOT="${SDKROOT:-$sdk}" PATH="$tools:$PATH"
    fi
    if grep -Eq '^    (openssl|trilogy) \(' "$cache/Gemfile.lock"; then
      openssl=${DEVLINE_OPENSSL_DIR:-/opt/homebrew/opt/openssl@3}
      if ! grep -Eq '^#[[:space:]]*define[[:space:]]+OPENSSL_VERSION_MAJOR[[:space:]]+3' "$openssl/include/openssl/opensslv.h" 2>/dev/null ||
        [ ! -f "$openssl/lib/libssl.dylib" ]; then
        echo 'Install OpenSSL 3 development files before building these gems. Use brew install openssl@3 or set DEVLINE_OPENSSL_DIR.' >&2
        exit 1
      fi
      export BUNDLE_BUILD__OPENSSL="--with-openssl-dir=$openssl" BUNDLE_BUILD__TRILOGY="--with-openssl-dir=$openssl"
      trilogy=$(awk '/^    trilogy \(/{gsub(/[()]/,"",$2); print $2; exit}' "$cache/Gemfile.lock")
      if [ -n "$trilogy" ]; then
        # Trilogy's extconf passes an unquoted -I flag. Clang's include path preserves spaces.
        ruby_abi=$("$ruby" -rrbconfig -e 'print RbConfig::CONFIG.fetch("ruby_version")')
        export C_INCLUDE_PATH="$BUNDLE_PATH/ruby/$ruby_abi/gems/trilogy-$trilogy/ext/trilogy-ruby/inc${C_INCLUDE_PATH:+:$C_INCLUDE_PATH}"
      fi
    fi
    if [ "$mode" = run ]; then
      [ "$#" -gt 0 ] || { echo 'Choose a command to run.' >&2; exit 2; }
      "$ruby" "$adapters/ruby-runtime-cache.rb" "$cache/ruby-runtime" "$cache" "$ruby" "$BUNDLER_VERSION" "$@"
      exit
    fi
    install_gems
    if [ -f bin/rails ] && grep -q '^    rails (' "$cache/Gemfile.lock" && [ -d test ]; then
      bundle exec "$ruby" bin/rails test
    elif [ -f Rakefile ] && grep -q '^    rake (' "$cache/Gemfile.lock"; then
      bundle exec rake -P > "$scratch/tasks"
      if grep -qx 'rake test' "$scratch/tasks"; then
        bundle exec rake test
      else
        check_syntax
      fi
    elif grep -q '^    minitest (' "$cache/Gemfile.lock"; then
      if "$ruby" -e 'exit Dir["test_*.rb", "test/**/*_test.rb", "test/**/test_*.rb"].empty?'; then
        bundle exec "$ruby" -e 'Dir["test_*.rb", "test/**/*_test.rb", "test/**/test_*.rb"].uniq.sort.each { |file| require File.expand_path(file) }'
      else
        check_syntax
      fi
    else
      check_syntax
    fi
    ;;
  *) echo 'Expected prepare, verify, or run.' >&2; exit 2 ;;
esac
