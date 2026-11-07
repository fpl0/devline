# Resolve through Bundler using the local gem cache, including its checksum store.
# `bundle lock --local` in Bundler 4 omits cached specifications for new manifests;
# `--add-checksums` contacts the registry. Both are inappropriate offline.
require "bundler"
require "rubygems/package"
definition = Bundler.definition
definition.add_platform(Gem::Platform.local)
definition.resolve_with_cache!.each do |spec|
  next unless spec.source.respond_to?(:checksum_store)
  archive = Bundler.app_cache.join("#{spec.full_name}.gem")
  next unless archive.file?
  package = Gem::Package.new(archive.to_s)
  checksum = Bundler::Checksum.from_gem_package(package)
  spec.source.checksum_store.register(spec, checksum)
end
definition.lock
