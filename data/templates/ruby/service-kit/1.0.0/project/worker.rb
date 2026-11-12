require_relative "lib/inbox"
root = ENV.fetch("DISPATCH_DATA", "data")
inbox = Inbox.new(File.join(root, "inbox.sqlite3"), output: File.join(root, "packing-slips"))
begin
  loop do
    event = inbox.process_one
    puts JSON.generate(event) if event
    break unless ARGV.include?("--watch")
    sleep 1 unless event
  end
ensure
  inbox.close
end
