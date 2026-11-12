require "openssl"
require "rack/utils"
require "securerandom"
require "fileutils"

module Signature
  module_function

  def secret
    return ENV.fetch("DISPATCH_SECRET") if ENV["DISPATCH_SECRET"] && ENV["DISPATCH_SECRET"].bytesize >= 32
    raise "DISPATCH_SECRET must contain at least 32 bytes" if ENV["DISPATCH_SECRET"]
    path = File.join(ENV.fetch("DISPATCH_DATA", "data"), "signing-key")
    FileUtils.mkdir_p(File.dirname(path))
    begin
      File.open(path, File::WRONLY | File::CREAT | File::EXCL, 0o600) { |file| file.write(SecureRandom.hex(32)) }
    rescue Errno::EEXIST
      # Reuse the private local key across restarts.
    end
    File.read(path)
  end

  def sign(secret, timestamp, body)
    OpenSSL::HMAC.hexdigest("SHA256", secret, "#{timestamp}.#{body}")
  end

  def valid?(secret, timestamp, body, signature, now: Time.now.to_i)
    timestamp.to_s.match?(/\A\d{1,12}\z/) && (now - timestamp.to_i).abs <= 300 &&
      signature.to_s.match?(/\A[0-9a-f]{64}\z/) && Rack::Utils.secure_compare(sign(secret, timestamp, body), signature)
  end
end
