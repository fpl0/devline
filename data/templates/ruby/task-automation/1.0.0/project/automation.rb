require "csv"
require "json"
require "net/http"
require "thor"
require "uri"

class Automation < Thor
  def self.exit_on_failure?
    true
  end

  desc "summarize SOURCE", "Aggregate a CSV file or HTTP fixture by category"
  option :output, type: :string, required: true
  def summarize(source)
    raw = if source.start_with?("https://", "http://")
      uri = URI(source)
      response = Net::HTTP.start(uri.host, uri.port, nil, use_ssl: uri.scheme == "https", open_timeout: 5, read_timeout: 5) do |http|
        http.request(Net::HTTP::Get.new(uri))
      end
      raise Thor::Error, "HTTP request failed with #{response.code}" unless response.is_a?(Net::HTTPSuccess)
      response.body
    else
      File.read(source)
    end

    rows = CSV.parse(raw, headers: true).group_by { |row| row.fetch("category") }
    summary = rows.sort.map do |category, items|
      {category: category, rows: items.size, total_cents: items.sum { |row| Integer(row.fetch("amount_cents"), 10) }}
    end
    File.write(options.fetch("output"), JSON.pretty_generate(summary) + "\n")
  rescue CSV::MalformedCSVError, KeyError, ArgumentError, SystemCallError => error
    raise Thor::Error, error.message
  end
end

Automation.start(ARGV) if $PROGRAM_NAME == __FILE__
