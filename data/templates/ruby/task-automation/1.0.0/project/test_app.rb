require "minitest/autorun"
require "rbconfig"
require "tmpdir"
require "webrick"
require_relative "automation"

class AutomationTest < Minitest::Test
  FIXTURE = File.expand_path("fixtures/sales.csv", __dir__)
  EXPECTED = [
    {"category" => "books", "rows" => 2, "total_cents" => 2000},
    {"category" => "coffee", "rows" => 1, "total_cents" => 450}
  ].freeze

  def test_file_cli
    Dir.mktmpdir do |temp|
      output = File.join(temp, "report.json")
      assert system(RbConfig.ruby, "automation.rb", "summarize", FIXTURE, "--output", output)
      assert_equal EXPECTED, JSON.parse(File.read(output))
    end
  end

  def test_loopback_http_fixture
    server = WEBrick::HTTPServer.new(BindAddress: "127.0.0.1", Port: 0, Logger: WEBrick::Log.new(File::NULL), AccessLog: [])
    server.mount_proc("/sales.csv") { |_request, response| response.body = File.read(FIXTURE) }
    thread = Thread.new { server.start }
    Dir.mktmpdir do |temp|
      output = File.join(temp, "report.json")
      url = "http://127.0.0.1:#{server.listeners.first.addr[1]}/sales.csv"
      Automation.start(["summarize", url, "--output", output])
      assert_equal EXPECTED, JSON.parse(File.read(output))
    end
  ensure
    server&.shutdown
    thread&.join
  end
end
