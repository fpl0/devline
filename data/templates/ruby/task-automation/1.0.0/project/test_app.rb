require "minitest/autorun"
require "tmpdir"
require_relative "lib/vault"

class VaultTest < Minitest::Test
  def setup
    @root = File.realpath(Dir.mktmpdir)
    @source = File.join(@root, "source")
    FileUtils.mkdir_p(File.join(@source, "docs"))
    File.write(File.join(@source, "notes.txt"), "hello\n")
    File.write(File.join(@source, "docs", "readme.md"), "# Notes\n")
    @vault_path = File.join(@root, "vault")
    @vault = SnapshotVault.new(@vault_path)
  end

  def teardown
    FileUtils.remove_entry(@root)
  end

  def test_capture_verify_restore_and_deduplication
    @vault.capture(@source, "first")
    @vault.capture(@source, "second")
    assert_equal 2, Dir.glob(File.join(@vault_path, "objects", "*")).size
    assert_equal 2, @vault.verify("first")[:verified_files]
    target = File.join(@root, "restored")
    @vault.restore("first", target)
    assert_equal "hello\n", File.read(File.join(target, "notes.txt"))
    assert_equal "# Notes\n", File.read(File.join(target, "docs", "readme.md"))
    assert_equal 2, SnapshotVault.new(@vault_path).snapshots.size
  end

  def test_restore_requires_overwrite_and_preflights_before_writing
    @vault.capture(@source, "first")
    target = File.join(@root, "restored")
    FileUtils.mkdir_p(target)
    File.write(File.join(target, "notes.txt"), "important")
    assert_raises(SnapshotVault::Error) { @vault.restore("first", target) }
    refute File.exist?(File.join(target, "docs"))
    assert_equal "important", File.read(File.join(target, "notes.txt"))
    @vault.restore("first", target, overwrite: true)
    assert_equal "hello\n", File.read(File.join(target, "notes.txt"))
  end

  def test_corruption_is_detected_before_restore
    @vault.capture(@source, "first")
    object = Dir.glob(File.join(@vault_path, "objects", "*")).first
    File.write(object, "corrupt")
    assert_raises(SnapshotVault::Error) { @vault.verify("first") }
    assert_raises(SnapshotVault::Error) { @vault.restore("first", File.join(@root, "restore")) }
    refute File.exist?(File.join(@root, "restore"))
  end

  def test_symlinks_and_path_traversal_are_rejected
    @vault.capture(@source, "first")
    target = File.join(@root, "target")
    FileUtils.mkdir_p(target)
    File.symlink(@source, File.join(target, "docs"))
    assert_raises(SnapshotVault::Error) { @vault.restore("first", target) }
    ancestor = File.join(@root, "ancestor")
    File.symlink(@source, ancestor)
    assert_raises(SnapshotVault::Error) { @vault.restore("first", File.join(ancestor, "new-directory")) }
    refute File.exist?(File.join(@source, "new-directory"))
    manifest = File.join(@vault_path, "snapshots", "first.json")
    data = JSON.parse(File.read(manifest))
    data["files"][0]["path"] = "../escape.txt"
    File.write(manifest, JSON.generate(data))
    assert_raises(SnapshotVault::Error) { @vault.restore("first", target) }
    File.symlink(File.join(@source, "notes.txt"), File.join(@source, "link"))
    assert_raises(SnapshotVault::Error) { @vault.capture(@source, "unsafe") }
    assert_raises(SnapshotVault::Error) { @vault.capture(@source, "../unsafe") }
  end
end
