require "sequel"
require "sqlite3"
require "json"
require "digest"
require "fileutils"
require "tempfile"
require "securerandom"

class Inbox
  class InvalidEvent < StandardError; end
  class Conflict < StandardError; end

  MAX_ATTEMPTS = 3

  def initialize(path, output:, clock: -> { Time.now.to_i })
    FileUtils.mkdir_p(File.dirname(path))
    @database = Sequel.sqlite(path, max_connections: 1, timeout: 5000)
    @database.run("PRAGMA journal_mode=WAL")
    @output, @clock = output, clock
    @database.create_table?(:events) do
      String :id, primary_key: true
      String :body, text: true, null: false
      String :digest, null: false
      String :status, null: false, default: "pending"
      Integer :attempts, null: false, default: 0
      Integer :available_at, null: false
      Integer :received_at, null: false
      String :error, text: true
      String :lease_token
    end
    unless @database.schema(:events).any? { |column, _| column == :lease_token }
      @database.alter_table(:events) { add_column :lease_token, String }
    end
    @events = @database[:events]
  end

  def close
    @database.disconnect
  end

  def accept(raw)
    event = JSON.parse(raw)
    raise InvalidEvent, "Expected an object with id, type, and data" unless event.is_a?(Hash) && event.keys.sort == %w[data id type]
    raise InvalidEvent, "Event id must use letters, numbers, underscores or hyphens (1–80 characters)" unless event["id"].is_a?(String) && event["id"].match?(/\A[A-Za-z0-9_-]{1,80}\z/)
    data = event["data"]
    valid = event["type"] == "order.paid" && data.is_a?(Hash) && data.keys.sort == %w[customer order_id total_cents] &&
            data["order_id"].is_a?(String) && data["order_id"].match?(/\A[A-Za-z0-9_-]{1,80}\z/) &&
            data["customer"].is_a?(String) && data["customer"].strip.length.between?(1, 120) &&
            data["total_cents"].is_a?(Integer) && data["total_cents"].between?(1, 100_000_000)
    raise InvalidEvent, "Expected order.paid with order_id, customer and positive integer total_cents" unless valid
    digest = Digest::SHA256.hexdigest(raw)
    @database.transaction(mode: :immediate) do
      if (previous = @events[id: event["id"]])
        raise Conflict, "Event id already belongs to a different payload" unless previous[:digest] == digest
        return [public_event(previous), false]
      end
      @events.insert(id: event["id"], body: raw, digest: digest, available_at: @clock.call, received_at: @clock.call)
    end
    [public_event(@events[id: event["id"]]), true]
  rescue JSON::ParserError => error
    raise InvalidEvent, error.message
  end

  def list
    @events.order(Sequel.desc(:received_at), :id).limit(100).all.map { |row| public_event(row) }
  end

  def retry_event(id)
    count = @events.where(id: id, status: "failed").update(status: "pending", attempts: 0, available_at: @clock.call, error: nil)
    raise Conflict, "Only failed events can be retried" unless count == 1
    public_event(@events[id: id])
  end

  def process_one(deliver: method(:write_slip))
    # A lease allows recovery after an interrupted worker; file delivery is idempotent by event id.
    token = SecureRandom.hex(16)
    event = @database.transaction(mode: :immediate) do
      now = @clock.call
      due = @events.where(status: ["pending", "processing"]).where { available_at <= now }.order(:received_at, :id).first
      if due
        @events.where(id: due[:id]).update(status: "processing", attempts: due[:attempts] + 1, available_at: @clock.call + 60, lease_token: token)
      end
      due
    end
    return nil unless event
    attempts = event[:attempts] + 1
    begin
      deliver.call(JSON.parse(event[:body]))
      @events.where(id: event[:id], status: "processing", lease_token: token).update(status: "processed", error: nil, lease_token: nil)
    rescue StandardError => error
      @events.where(id: event[:id], status: "processing", lease_token: token).update(status: attempts >= MAX_ATTEMPTS ? "failed" : "pending", lease_token: nil,
                                                                                     available_at: @clock.call + 2 ** attempts, error: error.message[0, 500])
    end
    public_event(@events[id: event[:id]])
  end

  def write_slip(event)
    FileUtils.mkdir_p(@output)
    content = JSON.pretty_generate({ event_id: event.fetch("id"), type: "packing_slip", order: event.fetch("data") }) + "\n"
    Tempfile.create([".slip-", ".json"], @output) do |file|
      file.write(content)
      file.flush
      file.fsync
      File.rename(file.path, File.join(@output, "#{event.fetch("id")}.json"))
    end
  end

  private

  def public_event(row)
    row.reject { |key, _| %i[body digest lease_token].include?(key) }.merge(data: JSON.parse(row[:body]).fetch("data"))
  end
end
