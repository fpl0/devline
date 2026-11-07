# __PROJECT_NAME__

A small Phoenix and Bandit API with a `GET /healthz` endpoint. Requires Elixir 1.20.5 and Erlang/OTP 29.

Run `./test-offline.sh` to compile and test using the included source dependencies and Rebar.
After compiling, run `PHX_SERVER=1 MIX_REBAR3="$PWD/vendor-tools/rebar3" MIX_ARCHIVES="$PWD/vendor-tools/archives" HEX_OFFLINE=1 mix run --no-halt` to serve the API at http://127.0.0.1:4000.

## Included local packages

The vendored Hex sources include Phoenix and Bandit, Ecto SQL with Postgrex,
MyXQL and SQLite, Redix, AMQP, ExAws S3 with Req HTTP, and Joken/JOSE for JWTs.
Erlang/OTP Logger and Elixir's ExUnit supply logging and testing. Exact
versions and Hex checksums are in `vendor/upstream-lock.exs`.

`./test-offline.sh` builds the vendored SQLite C code with the local
compiler. On macOS install the Apple Command Line Tools while online
(`xcode-select --install`). The tests exercise SQLite, local signing, and
client configuration. A live database, broker, cache or object store
requires its corresponding endpoint and credentials.

Devline compiles the vendored Ranch Erlang source with Rebar and uses
relative SQLite build paths, including when the project folder contains spaces.
