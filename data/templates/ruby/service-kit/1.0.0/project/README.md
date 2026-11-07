# __PROJECT_NAME__

A Sinatra service starter with a complete local gem cache for Ruby 4.0 on Apple Silicon macOS. Prepare Xcode Command Line Tools and OpenSSL 3 development files before disconnecting. The PostgreSQL and SQLite gems include their arm64 native binaries. Bundler builds the source extensions for Trilogy and OpenSSL locally with the prepared SDK and headers.

```sh
./bundle-offline.sh install
./bundle-offline.sh exec rackup -s webrick -o 127.0.0.1 -p 9292
curl -fsS http://127.0.0.1:9292/health
./test-offline.sh
```

Install OpenSSL 3 with `brew install openssl@3` while connected if needed. Set `DEVLINE_OPENSSL_DIR` to its prefix for a different installation. The script checks these prerequisites before resolving from the cache.

| Use | Included |
| --- | --- |
| Web and HTTP | Sinatra, Rackup, WEBrick, Faraday |
| Databases | Sequel, pg, Trilogy for MySQL, SQLite3 |
| Cache and messaging | redis, Bunny for AMQP, nats-pure |
| Object storage | AWS SDK for S3 |
| Configuration, authentication, logs, and tests | dotenv, JWT, OpenSSL, Logger, Minitest, Rack::Test |

The tests run SQLite queries and exercise the HTTP route, signing, and service client libraries. The object and HTTP calls use stubs. Connect your app to actual service URLs from Devline's project configuration. `Gemfile.lock` pins versions and checksums, and `vendor/cache` retains the original gem notices.
