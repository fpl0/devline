ENV["RACK_ENV"] = "test"
require "minitest/autorun"
require "rack/test"
require "tmpdir"
require_relative "app"

class InboxTest < Minitest::Test
  include Rack::Test::Methods
  SECRET = "a" * 64

  def app
    DispatchInbox
  end

  def setup
    @directory = Dir.mktmpdir
    @now = Time.now.to_i
    @inbox = Inbox.new(File.join(@directory, "inbox.db"), output: File.join(@directory, "slips"), clock: -> { @now })
    DispatchInbox.set :inbox, @inbox
    DispatchInbox.set :signing_secret, SECRET
    header "Host", "localhost"
    @body = JSON.generate(id: "evt-1", type: "order.paid", data: { order_id: "ORDER-1", customer: "Alex", total_cents: 1299 })
  end

  def teardown
    @inbox.close
    FileUtils.remove_entry(@directory)
  end

  def deliver(body = @body, timestamp = Time.now.to_i.to_s)
    header "Content-Type", "application/json"
    header "X-Event-Timestamp", timestamp
    header "X-Event-Signature", Signature.sign(SECRET, timestamp, body)
    post "/webhooks/orders", body
  end

  def test_signature_replay_and_idempotency
    deliver
    assert_equal 202, last_response.status
    deliver
    assert_equal 200, last_response.status
    deliver(@body.sub("1299", "1399"))
    assert_equal 409, last_response.status
    deliver(@body, (Time.now.to_i - 301).to_s)
    assert_equal 401, last_response.status
    header "X-Event-Signature", "0" * 64
    post "/webhooks/orders", @body
    assert_equal 401, last_response.status
    assert_equal 1, @inbox.list.size
  end

  def test_processing_failure_backoff_retry_and_restart
    @inbox.accept(@body)
    failure = ->(_) { raise IOError, "disk unavailable" }
    3.times do
      @inbox.process_one(deliver: failure)
      @now += 10
    end
    assert_equal "failed", @inbox.list.first[:status]
    @inbox.retry_event("evt-1")
    result = @inbox.process_one
    assert_equal "processed", result[:status]
    slip = JSON.parse(File.read(File.join(@directory, "slips", "evt-1.json")))
    assert_equal "ORDER-1", slip.fetch("order").fetch("order_id")
    @inbox.close
    @inbox = Inbox.new(File.join(@directory, "inbox.db"), output: File.join(@directory, "slips"))
    assert_equal "processed", @inbox.list.first[:status]
    assert_nil @inbox.process_one
  end

  def test_failed_attempt_is_not_immediately_reclaimed
    @inbox.accept(@body)
    @inbox.process_one(deliver: ->(_) { raise IOError, "temporary" })
    assert_nil @inbox.process_one
    @now += 3
    assert_equal "processed", @inbox.process_one[:status]
  end

  def test_expired_worker_cannot_overwrite_a_newer_success
    @inbox.accept(@body)
    stale_delivery = lambda do |_|
      @now += 61
      newer = Inbox.new(File.join(@directory, "inbox.db"), output: File.join(@directory, "slips"), clock: -> { @now })
      begin
        assert_equal "processed", newer.process_one[:status]
      ensure
        newer.close
      end
      raise IOError, "old attempt failed late"
    end
    assert_equal "processed", @inbox.process_one(deliver: stale_delivery)[:status]
    assert_equal 2, @inbox.list.first[:attempts]
    assert_nil @inbox.list.first[:error]
  end

  def test_validation_and_dashboard
    deliver("{}"); assert_equal 422, last_response.status
    deliver(@body.sub("evt-1", "../escape")); assert_equal 422, last_response.status
    get "/"; assert_equal 200, last_response.status
    assert_includes last_response.body, "Dispatch Inbox"
    header "Content-Type", "text/plain"
    post "/process", "{}"; assert_equal 415, last_response.status
  end
end
