require "aws-sdk-s3"
require "base64"
require "bunny"
require "fileutils"
require "json"
require "nats/io/client"
require "net/http"
require "net/smtp"
require "redis"
require "securerandom"
require "sequel"

module Support
  class ServiceUnavailable < StandardError; end

  QUEUE = "support-notifications"
  STREAM = "SUPPORT_ACTIVITY"
  INDEX = "support-tickets"
  BUCKET = "devline"
  SCHEMA_LOCK = Mutex.new

  module_function

  def setting(name)
    value = ENV["DEVLINE_#{name}"]
    raise ServiceUnavailable if value.nil? || value.empty?
    value
  end

  def with_tickets
    url = ENV["DEVLINE_POSTGRES_URL"]
    if url && !url.empty?
      db = Sequel.connect(url, connect_timeout: 3)
    else
      path = ENV.fetch("SUPPORT_SQLITE_PATH", "storage/support.sqlite3")
      FileUtils.mkdir_p(File.dirname(path))
      db = Sequel.sqlite(path, timeout: 3000)
    end
    SCHEMA_LOCK.synchronize do
      db.create_table?(:support_tickets) do
        String :id, primary_key: true
        String :title, null: false, text: true
        String :details, null: false, text: true
      end
    end
    yield db[:support_tickets]
  ensure
    db&.disconnect
  end

  def create_ticket(title, details)
    ticket = {id: SecureRandom.hex(16), title:, details:}
    with_tickets { |tickets| tickets.insert(ticket) }
    ticket
  end

  def list_tickets
    with_tickets { |tickets| tickets.order(:id).limit(100).all }
  end

  def read_ticket(id)
    url = ENV["DEVLINE_REDIS_URL"]
    return with_tickets { |tickets| tickets[id:] } if url.nil? || url.empty?

    key = "support:ticket:#{id}"
    cache = Redis.new(url:, timeout: 2, reconnect_attempts: 0)
    begin
      hit = cache.get(key)
      return JSON.parse(hit, symbolize_names: true) if hit
    rescue Redis::BaseError, JSON::ParserError
      # The authoritative record lives in PostgreSQL.
    end
    ticket = with_tickets { |tickets| tickets[id:] }
    begin
      cache.setex(key, 600, JSON.generate(ticket)) if ticket
    rescue Redis::BaseError
      # A cache outage leaves the database response available.
    end
    ticket
  ensure
    cache&.close
  end

  def with_queue
    broker = Bunny.new(
      host: "127.0.0.1", port: Integer(setting("RABBITMQ_AMQP_PORT")),
      username: "devline", password: setting("RABBITMQ_PASSWORD"), vhost: "devline",
      connection_timeout: 3, continuation_timeout: 3000,
      automatically_recover: false, recover_from_connection_close: false
    )
    broker.start
    channel = broker.create_channel
    yield channel, channel.queue(QUEUE, durable: true)
  ensure
    broker&.close if broker&.open?
  end

  def enqueue_notification(ticket)
    with_queue do |channel, queue|
      channel.confirm_select
      queue.publish(JSON.generate(ticket), persistent: true, mandatory: true)
      raise ServiceUnavailable unless channel.wait_for_confirms
    end
  end

  def deliver_notification
    port = Integer(setting("MAILPIT_SMTP_PORT"))
    with_queue do |channel, queue|
      delivery, _, payload = queue.pop(manual_ack: true)
      next nil unless delivery
      ticket = JSON.parse(payload, symbolize_names: true)
      from = "support@example.test"
      recipient = "triage@example.test"
      body = Base64.encode64("#{ticket.fetch(:title)}\n\n#{ticket.fetch(:details)}")
      message = <<~EMAIL
        From: #{from}
        To: #{recipient}
        Subject: Support ticket #{ticket.fetch(:id)}
        MIME-Version: 1.0
        Content-Type: text/plain; charset=UTF-8
        Content-Transfer-Encoding: base64

        #{body}
      EMAIL
      smtp = Net::SMTP.new("127.0.0.1", port, open_timeout: 4, read_timeout: 4, starttls: false)
      # Closing without an acknowledgement requeues the job when SMTP fails.
      smtp.start("localhost") { |connection| connection.send_message(message, from, recipient) }
      channel.ack(delivery.delivery_tag)
      ticket
    end
  end

  def search_request(method, path, body: nil, missing: nil)
    url = URI("#{setting('MEILISEARCH_URL')}#{path}")
    request = method.new(url)
    request["Authorization"] = "Bearer #{setting('MEILISEARCH_PASSWORD')}"
    if body
      request["Content-Type"] = "application/json"
      request.body = JSON.generate(body)
    end
    response = Net::HTTP.start(url.host, url.port, nil, open_timeout: 4, read_timeout: 4) do |http|
      http.request(request)
    end
    return missing if response.code == "404" && !missing.nil?
    raise ServiceUnavailable unless response.is_a?(Net::HTTPSuccess)
    JSON.parse(response.body)
  end

  def index_ticket(ticket)
    search_request(Net::HTTP::Post, "/indexes/#{INDEX}/documents?primaryKey=id", body: [ticket]).fetch("taskUid")
  end

  def index_status(task_id)
    search_request(Net::HTTP::Get, "/tasks/#{task_id}").fetch("status")
  end

  def search_tickets(query)
    search_request(Net::HTTP::Post, "/indexes/#{INDEX}/search",
      body: {q: query, limit: 20}, missing: {"hits" => []}).fetch("hits")
  end

  def object_store
    Aws::S3::Client.new(
      endpoint: setting("SEAWEEDFS_URL"), region: "us-east-1",
      access_key_id: "devline", secret_access_key: setting("SEAWEEDFS_PASSWORD"),
      force_path_style: true, retry_limit: 0, http_open_timeout: 3, http_read_timeout: 3
    )
  end

  def export_ticket(ticket)
    object_store.put_object(bucket: BUCKET, key: "support/#{ticket.fetch(:id)}.json",
      body: JSON.generate(ticket), content_type: "application/json")
  end

  def read_export(id)
    result = object_store.get_object(bucket: BUCKET, key: "support/#{id}.json")
    JSON.parse(result.body.read)
  rescue Aws::S3::Errors::NoSuchKey
    nil
  end

  def activity(id, ticket = nil)
    connection = NATS.connect(setting("NATS_URL"), connect_timeout: 3, reconnect: false)
    stream = connection.jetstream(timeout: 3)
    subject = "support.ticket.#{id}"
    if ticket
      stream.add_stream(name: STREAM, subjects: ["support.ticket.*"], max_msgs: 1000)
      event = {id:, event: "reviewed", title: ticket.fetch(:title)}
      stream.publish(subject, JSON.generate(event))
      return event
    end
    JSON.parse(stream.get_last_msg(STREAM, subject).data)
  rescue NATS::JetStream::Error::NotFound
    raise if ticket
    nil
  ensure
    connection&.close
  end
end
