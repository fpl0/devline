require "net/http"
require "json"
require "uri"
require_relative "lib/signature"

path = ARGV.fetch(0, "examples/order-paid.json")
url = URI(ENV.fetch("DISPATCH_URL", "http://127.0.0.1:7461/webhooks/orders"))
abort "DISPATCH_URL must be HTTP(S)" unless %w[http https].include?(url.scheme)
body, timestamp = File.read(path), Time.now.to_i.to_s
request = Net::HTTP::Post.new(url, { "Content-Type" => "application/json", "X-Event-Timestamp" => timestamp,
                                     "X-Event-Signature" => Signature.sign(Signature.secret, timestamp, body) })
request.body = body
response = Net::HTTP.start(url.host, url.port, nil, use_ssl: url.scheme == "https", open_timeout: 5, read_timeout: 5) { |http| http.request(request) }
puts response.body
exit(response.is_a?(Net::HTTPSuccess) ? 0 : 1)
