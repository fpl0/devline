#!/bin/sh
set -eu
url=${DEVLINE_APP_URL:-http://127.0.0.1:7461}
exec ruby -S rackup -s webrick -o 127.0.0.1 -p "${url##*:}"
