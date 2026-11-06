# __PROJECT_NAME__

A small Phoenix and Bandit API with a `GET /healthz` endpoint. Requires Elixir 1.20.5 and Erlang/OTP 29.

Run `./test-offline.sh` to compile and test using the included source dependencies and Rebar.
After compiling, run `PHX_SERVER=1 MIX_REBAR3="$PWD/vendor-tools/rebar3" MIX_ARCHIVES="$PWD/vendor-tools/archives" HEX_OFFLINE=1 mix run --no-halt` to serve the API at http://127.0.0.1:4000.
