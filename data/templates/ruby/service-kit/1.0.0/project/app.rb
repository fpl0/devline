require "dotenv"
require "logger"
require "sinatra/base"
require_relative "support"

class ServiceKit < Sinatra::Base
  set :logger, Logger.new($stdout)
  set :show_exceptions, false
  set :raise_errors, false

  before { content_type :json }

  error Support::ServiceUnavailable, Sequel::Error, Bunny::Exception,
    Aws::S3::Errors::ServiceError, Seahorse::Client::NetworkingError, NATS::Error,
    Net::SMTPError, SystemCallError, IOError, Timeout::Error do
    # Service URLs and exception messages can contain generated credentials.
    settings.logger.warn("support_service_unavailable type=#{env.fetch('sinatra.error').class}")
    status 503
    JSON.generate(detail: "Local service unavailable. Check Devline services.")
  end

  not_found { JSON.generate(detail: "Not found") }

  helpers do
    def fail_request(code, message)
      halt code, JSON.generate(detail: message)
    end

    def ticket_id
      id = params.fetch("id")
      fail_request(422, "Expected a ticket ID") unless id.match?(/\A[a-f0-9]{32}\z/)
      id
    end

    def ticket
      Support.read_ticket(ticket_id) || fail_request(404, "Ticket not found")
    end

    def ticket_input
      body = request.body.read(64_001)
      fail_request(413, "Ticket body is too large") if body.bytesize > 64_000
      input = JSON.parse(body)
      unless input.is_a?(Hash) && input.keys.sort == %w[details title]
        fail_request(422, "Send a title and details")
      end
      title = input.fetch("title")
      details = input.fetch("details")
      unless title.is_a?(String) && details.is_a?(String)
        fail_request(422, "Title and details must be strings")
      end
      title = title.strip
      details = details.strip
      unless title.length.between?(1, 120) && details.length.between?(1, 4000) &&
          !title.match?(/[\r\n\x00]/) && !details.include?("\0")
        fail_request(422, "Use a single-line title and nonempty details within the length limits")
      end
      [title, details]
    rescue JSON::ParserError
      fail_request(422, "Expected a JSON object")
    end
  end

  get "/" do
    JSON.generate(app: "Support Desk", create: "POST /tickets with title and details", list: "GET /tickets")
  end

  get "/health" do
    environment = ENV.fetch("APP_ENVIRONMENT", "local")
    settings.logger.info("health_checked environment=#{environment}")
    JSON.generate(status: "ok", environment:)
  end

  post "/tickets" do
    record = Support.create_ticket(*ticket_input)
    status 201
    JSON.generate(record)
  end

  get "/tickets" do
    JSON.generate(Support.list_tickets)
  end

  get "/tickets/:id" do
    JSON.generate(ticket)
  end

  post "/tickets/:id/notify" do
    Support.enqueue_notification(ticket)
    status 202
    JSON.generate(id: ticket_id, status: "queued")
  end

  post "/notifications/next" do
    record = Support.deliver_notification
    JSON.generate(record ? {id: record.fetch(:id), status: "sent"} : {status: "empty"})
  end

  post "/tickets/:id/index" do
    task = Support.index_ticket(ticket)
    status 202
    JSON.generate(task:)
  end

  get "/search/tasks/:task" do
    task = params.fetch("task")
    fail_request(422, "Expected a task number") unless task.match?(/\A[0-9]+\z/)
    JSON.generate(status: Support.index_status(task))
  end

  get "/search" do
    query = params.fetch("q", "")
    fail_request(422, "Provide a search query of up to 120 characters") unless query.length.between?(1, 120)
    JSON.generate(Support.search_tickets(query))
  end

  post "/tickets/:id/export" do
    Support.export_ticket(ticket)
    status 201
    JSON.generate(id: ticket_id, status: "exported")
  end

  get "/tickets/:id/export" do
    record = Support.read_export(ticket_id) || fail_request(404, "Export not found")
    JSON.generate(record)
  end

  post "/tickets/:id/activity" do
    event = Support.activity(ticket_id, ticket)
    status 201
    JSON.generate(event)
  end

  get "/tickets/:id/activity" do
    event = Support.activity(ticket_id) || fail_request(404, "Activity not found")
    JSON.generate(event)
  end
end
