require "thor"
require_relative "lib/vault"

class Automation < Thor
  class_option :vault, type: :string, default: "data/vault", desc: "Directory containing snapshots and content-addressed objects"

  def self.exit_on_failure?
    true
  end

  desc "capture SOURCE", "Create an immutable snapshot of a directory"
  option :name, type: :string, required: true

  def capture(source)
    execute { |vault| vault.capture(source, options.fetch("name")) }
  end

  desc "list", "List snapshots, file counts and sizes"

  def list
    execute(&:snapshots)
  end

  desc "verify NAME", "Verify every stored byte against its SHA-256 digest"

  def verify(name)
    execute { |vault| vault.verify(name) }
  end

  desc "restore NAME DESTINATION", "Restore files; existing files are protected by default"
  option :overwrite, type: :boolean, default: false

  def restore(name, destination)
    execute { |vault| vault.restore(name, destination, overwrite: options["overwrite"]) }
  end

  no_commands do
    def execute
      result = yield SnapshotVault.new(options.fetch("vault"))
      puts JSON.pretty_generate(result)
    rescue SnapshotVault::Error, SystemCallError => error
      raise Thor::Error, error.message
    end
  end
end

Automation.start(ARGV) if $PROGRAM_NAME == __FILE__
