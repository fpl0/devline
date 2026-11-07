# Compiled gems are derived local output. Their key includes the complete source
# gem closure and the selected Ruby ABI, so no global gem installation is reused.
require 'digest'
require 'fileutils'
require 'json'
require 'rbconfig'
require 'securerandom'

root, source, ruby, bundler, *command = ARGV
abort 'Choose a command to run.' if command.empty?
source = File.realpath(source)
ruby = File.realpath(ruby)
FileUtils.mkdir_p(root, mode: 0o700)
abort 'Ruby runtime cache must be a directory, not a symlink.' if File.symlink?(root) || !File.directory?(root)
root = File.realpath(root)
identity = Digest::SHA256.new
identity << 'devline-ruby-runtime-v1' << ruby << RUBY_DESCRIPTION << RUBY_PLATFORM << bundler
%w[ruby_version arch host_os DLEXT CC configure_args].each { |key| identity << key << RbConfig::CONFIG.fetch(key, '') }
ruby_stat = File.stat(ruby)
identity << [ruby_stat.dev, ruby_stat.ino, ruby_stat.size, ruby_stat.mtime.to_f].join(':')
[File.join(source, 'Gemfile'), File.join(source, 'Gemfile.lock'), __FILE__, File.join(__dir__, 'ruby.sh')].each do |file|
  identity << File.basename(file) << Digest::SHA256.file(file).hexdigest
end
Dir.glob(File.join(source, 'vendor/cache/*.gem')).sort.each do |file|
  abort 'Vendored Ruby gems must be regular files.' unless File.file?(file) && !File.symlink?(file)
  identity << File.basename(file) << Digest::SHA256.file(file).hexdigest
end
ENV.keys.grep(/\ABUNDLE_BUILD__/).sort.each { |key| identity << key << ENV.fetch(key) }
identity << ENV.fetch('SDKROOT', '')
sdk_settings = File.join(ENV.fetch('SDKROOT', ''), 'SDKSettings.json')
identity << Digest::SHA256.file(sdk_settings).hexdigest if File.file?(sdk_settings)
openssl = ENV.fetch('DEVLINE_OPENSSL_DIR', '/opt/homebrew/opt/openssl@3')
%w[include/openssl/opensslv.h lib/libssl.dylib lib/libcrypto.dylib].each do |relative|
  file = File.join(openssl, relative)
  identity << Digest::SHA256.file(file).hexdigest if File.file?(file)
end
key = identity.hexdigest
cached = File.join(root, key)
lock_path = File.join(root, "#{key}.lock")
flags = File::RDWR | File::CREAT
flags |= File::NOFOLLOW if defined?(File::NOFOLLOW)
original_include = ENV['C_INCLUDE_PATH']
original_bundle = ENV.fetch('BUNDLE_PATH')
use_bundle = lambda do |path|
  ENV['BUNDLE_PATH'] = path
  # Trilogy's compiler include option cannot quote spaces. Keep its explicit
  # include search path aligned with the currently staged or published bundle.
  ENV['C_INCLUDE_PATH'] = original_include.gsub(original_bundle, path) if original_include
end
bundle = [ruby, '-S', 'bundle', "_#{bundler}_"]
File.open(lock_path, flags, 0o600) do |lock|
  lock.flock(File::LOCK_EX)
  use_bundle.call(cached)
  receipt = begin
    path = File.join(cached, '.devline-ready.json')
    JSON.parse(File.read(path)) if File.file?(path) && !File.symlink?(path)
  rescue JSON::ParserError, Errno::ENOENT
    nil
  end
  valid = File.directory?(cached) && !File.symlink?(cached) && receipt && receipt['key'] == key &&
    system(*bundle, 'check', out: File::NULL, err: File::NULL)
  if valid
    warn 'Reusing prepared Ruby gems.'
  else
    abort 'Ruby runtime cache contains an unexpected symbolic link.' if File.symlink?(cached)
    staging = File.join(root, ".building-#{key}-#{SecureRandom.hex(6)}")
    begin
      Dir.mkdir(staging, 0o700)
      use_bundle.call(staging)
      warn 'Preparing Ruby gems from the project’s offline cache.'
      abort 'Offline Ruby gem preparation failed.' unless system(*bundle, 'install', '--local', '--jobs', '4', out: $stderr, err: $stderr)
      abort 'Prepared Ruby gems did not pass Bundler checks.' unless system(*bundle, 'check', out: $stderr, err: $stderr)
      File.open(File.join(staging, '.devline-ready.json'), File::WRONLY | File::CREAT | File::EXCL, 0o600) do |file|
        file.write(JSON.generate(key: key, ruby: ruby, abi: RbConfig::CONFIG.fetch('ruby_version')))
        file.flush
        file.fsync
      end
      FileUtils.rm_rf(cached) if File.exist?(cached)
      File.rename(staging, cached)
      File.open(root) { |directory| directory.fsync }
    ensure
      FileUtils.rm_rf(staging) if staging && File.exist?(staging)
    end
  end
end
use_bundle.call(cached)
exec(*bundle, 'exec', *command)
