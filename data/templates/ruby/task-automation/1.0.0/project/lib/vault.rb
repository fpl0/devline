require "digest"
require "fileutils"
require "find"
require "json"
require "pathname"
require "tempfile"
require "time"

class SnapshotVault
  class Error < StandardError; end

  NAME = /\A[A-Za-z0-9][A-Za-z0-9_-]{0,63}\z/

  def initialize(directory)
    raise Error, "Vault cannot be a symbolic link" if File.symlink?(directory)
    FileUtils.mkdir_p(directory)
    @root = File.realpath(directory)
    %w[objects snapshots].each { |name| safe_directory(File.join(@root, name)) }
  end

  def snapshots
    Dir.glob(File.join(@root, "snapshots", "*.json")).sort.map do |path|
      manifest = read_manifest(File.basename(path, ".json"))
      { name: manifest.fetch("name"), created_at: manifest.fetch("created_at"), files: manifest.fetch("files").size,
        bytes: manifest.fetch("files").sum { |file| file.fetch("bytes") } }
    end
  end

  def capture(source, name)
    validate_name(name)
    raise Error, "Source cannot be a symbolic link" if File.symlink?(source)
    source = File.realpath(source)
    raise Error, "Source must be a directory" unless File.directory?(source)
    raise Error, "Keep the vault outside the source tree" if @root == source || @root.start_with?(source + File::SEPARATOR)
    locked do
      path = manifest_path(name)
      raise Error, "Snapshot already exists: #{name}" if File.exist?(path) || File.symlink?(path)
      files = []
      Find.find(source) do |file|
        next if file == source
        raise Error, "Symbolic links are not supported: #{file}" if File.symlink?(file)
        next if File.directory?(file)
        raise Error, "Only regular files are supported: #{file}" unless File.file?(file)
        before = File.stat(file)
        digest = Digest::SHA256.file(file).hexdigest
        object = object_path(digest)
        if File.exist?(object)
          verify_object(digest, before.size)
        else
          atomic_write(object) { |output| File.open(file, "rb") { |input| IO.copy_stream(input, output) } }
          verify_object(digest, before.size)
        end
        after = File.stat(file)
        raise Error, "File changed while taking snapshot: #{file}" unless [before.ino, before.size, before.mtime] == [after.ino, after.size, after.mtime]
        files << { path: Pathname(file).relative_path_from(Pathname(source)).to_s, bytes: before.size, sha256: digest, mode: before.mode & 0o777 }
      end
      manifest = { version: 1, name: name, created_at: Time.now.utc.iso8601, source: source, files: files.sort_by { |item| item[:path] } }
      atomic_write(path) { |output| output.write(JSON.pretty_generate(manifest) + "\n") }
      manifest
    end
  end

  def verify(name)
    manifest = read_manifest(name)
    manifest.fetch("files").each { |file| verify_object(file.fetch("sha256"), file.fetch("bytes")) }
    { name: name, verified_files: manifest.fetch("files").size, bytes: manifest.fetch("files").sum { |file| file.fetch("bytes") } }
  end

  def restore(name, destination, overwrite: false)
    manifest = read_manifest(name)
    verify(name)
    raise Error, "Destination cannot be a symbolic link" if File.symlink?(destination)
    destination = File.expand_path(destination)
    Pathname(destination).ascend do |parent|
      raise Error, "Refusing symbolic link ancestor: #{parent}" if File.symlink?(parent)
    end
    raise Error, "Cannot restore into the vault" if destination == @root || destination.start_with?(@root + File::SEPARATOR)
    # Preflight every path before writing any file.
    targets = manifest.fetch("files").map do |file|
      target = File.join(destination, file.fetch("path"))
      check_target(destination, target, overwrite)
      [file, target]
    end
    safe_directory(destination)
    targets.each do |file, target|
      relative = Pathname(target).relative_path_from(Pathname(destination)).each_filename.to_a
      relative[0...-1].reduce(destination) { |parent, part| safe_directory(File.join(parent, part)) }
      check_target(destination, target, overwrite)
      atomic_write(target) { |output| File.open(object_path(file.fetch("sha256")), "rb") { |input| IO.copy_stream(input, output) } }
      File.chmod(file.fetch("mode"), target)
    end
    { name: name, restored_files: targets.size, destination: destination }
  end

  private

  def validate_name(name)
    raise Error, "Snapshot name must use 1–64 letters, numbers, underscores or hyphens" unless name.is_a?(String) && NAME.match?(name)
  end

  def manifest_path(name)
    validate_name(name)
    File.join(@root, "snapshots", "#{name}.json")
  end

  def object_path(digest)
    raise Error, "Invalid object digest" unless digest.is_a?(String) && digest.match?(/\A[0-9a-f]{64}\z/)
    File.join(@root, "objects", digest)
  end

  def read_manifest(name)
    path = manifest_path(name)
    raise Error, "Manifest cannot be a symbolic link" if File.symlink?(path)
    data = JSON.parse(File.read(path))
    raise Error, "Unsupported manifest" unless data.is_a?(Hash) && data["version"] == 1 && data["name"] == name && data["files"].is_a?(Array)
    paths = []
    data.fetch("files").each do |file|
      raise Error, "Manifest entries must be objects" unless file.is_a?(Hash)
      path = file.fetch("path")
      raise Error, "Unsafe manifest path" unless path.is_a?(String) && !path.include?("\0") && !path.empty? && !path.start_with?("/") && !path.include?("\\") && path.split("/", -1).none? { |part| ["", ".", ".."].include?(part) }
      raise Error, "Duplicate manifest path" if paths.include?(path)
      paths << path
      raise Error, "Invalid file size or mode" unless file["bytes"].is_a?(Integer) && file["bytes"] >= 0 && file["mode"].is_a?(Integer) && file["mode"].between?(0, 0o777)
      object_path(file.fetch("sha256"))
    end
    paths.sort.each_cons(2) do |parent, child|
      raise Error, "Manifest contains conflicting file paths" if child.start_with?(parent + File::SEPARATOR)
    end
    data
  rescue JSON::ParserError, KeyError => error
    raise Error, "Invalid manifest: #{error.message}"
  end

  def verify_object(digest, bytes)
    object = object_path(digest)
    raise Error, "Corrupt or missing object #{digest}" if File.symlink?(object) || !File.file?(object) || File.size(object) != bytes || Digest::SHA256.file(object).hexdigest != digest
  end

  def safe_directory(path)
    raise Error, "Refusing symbolic link: #{path}" if File.symlink?(path)
    FileUtils.mkdir_p(path)
    raise Error, "Not a directory: #{path}" unless File.directory?(path)
    path
  end

  def check_target(root, target, overwrite)
    current = root
    raise Error, "Refusing symbolic link: #{root}" if File.symlink?(root)
    relative = Pathname(target).relative_path_from(Pathname(root)).each_filename.to_a
    relative.each_with_index do |part, index|
      current = File.join(current, part)
      raise Error, "Refusing symbolic link: #{current}" if File.symlink?(current)
      next unless File.exist?(current)
      if index == relative.length - 1
        raise Error, "Refusing to overwrite #{current}; pass --overwrite explicitly" unless overwrite && File.file?(current)
      else
        raise Error, "Parent is not a directory: #{current}" unless File.directory?(current)
      end
    end
  end

  def atomic_write(path)
    raise Error, "Refusing symbolic link: #{path}" if File.symlink?(path)
    Tempfile.create([".snapshot-", ".tmp"], File.dirname(path)) do |output|
      yield output
      output.flush
      output.fsync
      File.rename(output.path, path)
    end
  end

  def locked
    lock = File.join(@root, ".lock")
    raise Error, "Lock cannot be a symbolic link" if File.symlink?(lock)
    File.open(lock, File::RDWR | File::CREAT, 0o600) do |file|
      file.flock(File::LOCK_EX)
      yield
    ensure
      file.flock(File::LOCK_UN)
    end
  end
end
