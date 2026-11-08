require "aws-sdk-s3"
require "bunny"
require "faraday"
require "jwt"
require "minitest/autorun"
require "nats/io/client"
require "pg"
require "rack/test"
require "redis"
require "sequel"
require "sqlite3"
require "trilogy"
require "tmpdir"
require_relative "app"

class ServiceKitTest < Minitest::Test
  include Rack::Test::Methods

  def app
    ServiceKit
  end

  def test_rack_route_and_config
    header "Host", "localhost"
    get "/health"
    assert_equal 200, last_response.status
    assert_equal "ok", JSON.parse(last_response.body).fetch("status")
    assert_equal "local", Dotenv.parse(".env.example").fetch("APP_ENVIRONMENT")
  end

  def test_database_adapters_and_sqlite_query
    assert_operator PG.library_version, :>, 0
    assert_includes Trilogy.public_instance_methods, :query
    database = Sequel.sqlite
    database.create_table(:items) { String :name }
    database[:items].insert(name: "offline")
    assert_equal "offline", database[:items].first.fetch(:name)
  ensure
    database&.disconnect
  end

  def test_messaging_and_cache_clients
    broker = Bunny.new(host: "127.0.0.1", vhost: "devline")
    refute broker.open?
    assert_instance_of NATS::Client, NATS::Client.new
    cache = Redis.new(host: "127.0.0.1", port: 6379)
    assert_instance_of Redis, cache
  ensure
    cache&.close
  end

  def test_s3_http_and_jwt
    s3 = Aws::S3::Client.new(region: "us-east-1", stub_responses: true)
    s3.stub_responses(:list_buckets, buckets: [{name: "devline"}])
    assert_equal "devline", s3.list_buckets.buckets.first.name

    stubs = Faraday::Adapter::Test::Stubs.new do |stub|
      stub.get("/health") { [200, {"Content-Type" => "application/json"}, '{"status":"ok"}'] }
    end
    http = Faraday.new("http://127.0.0.1") { |builder| builder.adapter :test, stubs }
    assert_equal "ok", JSON.parse(http.get("/health").body).fetch("status")

    key = OpenSSL::PKey::RSA.generate(2048)
    token = JWT.encode({sub: "local-user"}, key, "RS256")
    assert_equal "local-user", JWT.decode(token, key.public_key, true, algorithm: "RS256").first.fetch("sub")
  end

  def test_ticket_workflow_with_sqlite
    keys = %w[DEVLINE_POSTGRES_URL DEVLINE_REDIS_URL DEVLINE_MEILISEARCH_URL SUPPORT_SQLITE_PATH]
    previous = keys.to_h { |key| [key, ENV.delete(key)] }
    Dir.mktmpdir("support-test") do |directory|
      ENV["SUPPORT_SQLITE_PATH"] = File.join(directory, "support.sqlite3")
      header "Host", "localhost"
      header "Content-Type", "application/json"
      post "/tickets", JSON.generate(title: " ", details: "Something broke")
      assert_equal 422, last_response.status
      post "/tickets", JSON.generate(title: " Printer ", details: "Paper jam")
      assert_equal 201, last_response.status
      ticket = JSON.parse(last_response.body)
      assert_equal "Printer", ticket.fetch("title")
      get "/tickets"
      assert_equal [ticket], JSON.parse(last_response.body)
      get "/tickets/#{ticket.fetch('id')}"
      assert_equal ticket, JSON.parse(last_response.body)
      get "/tickets/#{"0" * 32}"
      assert_equal 404, last_response.status
      get "/search?q=Printer"
      assert_equal 503, last_response.status
    end
  ensure
    ENV.update(previous)
  end
end
