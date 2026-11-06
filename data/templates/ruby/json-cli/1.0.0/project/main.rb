require "json"
require "optparse"

options = { pretty: false }
parser = OptionParser.new do |args|
  args.banner = "Usage: ruby main.rb [--pretty] < data.json"
  args.on("--pretty", "Indent JSON output") { options[:pretty] = true }
end

begin
  parser.parse!
  data = JSON.parse($stdin.read)
  puts options[:pretty] ? JSON.pretty_generate(data) : JSON.generate(data)
rescue JSON::ParserError, OptionParser::ParseError => error
  warn error.message
  exit 1
end
