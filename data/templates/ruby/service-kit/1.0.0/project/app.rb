require "sinatra/base"
require "json"
require_relative "lib/inbox"
require_relative "lib/signature"

class DispatchInbox < Sinatra::Base
  set :bind, "127.0.0.1"
  set :show_exceptions, false
  set :raise_errors, false
  set :host_authorization, { permitted_hosts: ["localhost", "127.0.0.1", "::1"] }
  set :public_folder, File.expand_path("public", __dir__)
  set(:inbox) { @inbox ||= Inbox.new(File.join(ENV.fetch("DISPATCH_DATA", "data"), "inbox.sqlite3"), output: File.join(ENV.fetch("DISPATCH_DATA", "data"), "packing-slips")) }
  set(:signing_secret) { Signature.secret }

  helpers do
    def output(value, status_code = 200)
      content_type :json
      status status_code
      JSON.generate(value)
    end

    def json_only!
      halt 415, output({ error: "Use application/json" }, 415) unless request.media_type == "application/json"
    end
  end

  get("/") { send_file File.join(settings.public_folder, "index.html") }
  get("/health") { output({ status: "ok" }) }
  get("/events") { output(settings.inbox.list) }

  post "/webhooks/orders" do
    json_only!
    body = request.body.read(65_537)
    halt 413, output({ error: "Payload exceeds 64 KiB" }, 413) if body.bytesize > 65_536
    unless Signature.valid?(settings.signing_secret, request.env["HTTP_X_EVENT_TIMESTAMP"], body, request.env["HTTP_X_EVENT_SIGNATURE"])
      halt 401, output({ error: "Signature is invalid or older than five minutes" }, 401)
    end
    event, created = settings.inbox.accept(body)
    output(event, created ? 202 : 200)
  end

  post "/process" do
    json_only!
    results = []
    20.times do
      event = settings.inbox.process_one
      break unless event
      results << event
    end
    output(results)
  end

  post "/events/:id/retry" do
    json_only!
    output(settings.inbox.retry_event(params.fetch("id")))
  end

  error Inbox::InvalidEvent do
    output({ error: env["sinatra.error"].message }, 422)
  end

  error Inbox::Conflict do
    output({ error: env["sinatra.error"].message }, 409)
  end
end
