#!/bin/sh
set -eu
cd "$(dirname "$0")"
tool=${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-dotnet}}
export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE=true DOTNET_GENERATE_ASPNET_CERTIFICATE=false
export DOTNET_EnableDiagnostics=0 DOTNET_CLI_USE_MSBUILD_SERVER=0 MSBUILDDISABLENODEREUSE=1
"$tool" restore OfflineApi.csproj --locked-mode --configfile NuGet.Config -p:NuGetAudit=false
"$tool" build OfflineApi.csproj --no-restore -p:UseSharedCompilation=false

scratch=$(mktemp -d "$PWD/obj/devline-http.XXXXXX")
pid=
stop_server() {
    if [ -n "$pid" ]; then
        kill "$pid" 2>/dev/null || true
        # A host can block SIGTERM before starting the offline test. Bound
        # shutdown so the smoke test can still check a SQLite restart.
        attempts=0
        while [ "$attempts" -lt 20 ] && kill -0 "$pid" 2>/dev/null; do
            attempts=$((attempts + 1))
            sleep 0.1
        done
        if [ "$attempts" -eq 20 ]; then kill -KILL "$pid" 2>/dev/null || true; fi
        wait "$pid" 2>/dev/null || true
        pid=
    fi
}
cleanup() {
    stop_server
    /bin/rm -r "$scratch"
}
trap cleanup EXIT
trap 'exit 130' HUP INT TERM

log="$scratch/server.log"
body="$scratch/response.json"
headers="$scratch/headers"
start_server() {
    ASPNETCORE_URLS=http://127.0.0.1:0 \
        ConnectionStrings__Tasks="Data Source=$scratch/tasks.db" \
        Logging__LogLevel__Microsoft__Hosting__Lifetime=Information \
        "$tool" bin/Debug/net10.0/OfflineApi.dll >"$log" 2>&1 &
    pid=$!
    attempts=0
    while [ "$attempts" -lt 200 ] && kill -0 "$pid" 2>/dev/null; do
        base=$(awk '/Now listening on: http:\/\/127\.0\.0\.1:/ { print $NF; exit }' "$log")
        if [ -n "$base" ]; then return; fi
        attempts=$((attempts + 1))
        sleep 0.1
    done
    printf '%s\n' 'The REST API did not start.' >&2
    cat "$log" >&2
    exit 1
}

request() {
    method=$1
    path=$2
    expected=$3
    shift 3
    status=$(/usr/bin/curl --silent --show-error --noproxy '*' --max-time 5 \
        --request "$method" --header 'Content-Type: application/json' \
        --dump-header "$headers" --output "$body" --write-out '%{http_code}' "$@" "$base$path")
    if [ "$status" != "$expected" ]; then
        printf '%s %s returned %s (expected %s)\n' "$method" "$path" "$status" "$expected" >&2
        cat "$body" >&2
        exit 1
    fi
}

field() {
    /usr/bin/plutil -extract "$1" raw -o - "$body"
}

start_server
request GET /health/ready 200
test "$(field status)" = ready
test "$(field database)" = sqlite
request GET /api/tasks 200
test "$(cat "$body")" = '[]'
request POST /api/tasks 400 --data '{"title":"   "}'
request POST /api/tasks 201 --data '{"title":" Read docs "}'
id=$(field id)
resource="/api/tasks/$id"
location=$(awk 'tolower($1) == "location:" { sub(/\r$/, "", $2); print $2 }' "$headers")
test "$location" = "$base$resource"
test "$(field title)" = 'Read docs'
test "$(field completed)" = false
request PUT "$resource" 204 --data '{"title":"Read offline docs","completed":true}'
stop_server

start_server
request GET "$resource" 200
test "$(field id)" = "$id"
test "$(field title)" = 'Read offline docs'
test "$(field completed)" = true
request GET /api/tasks 200
test "$(field 0.id)" = "$id"
request DELETE "$resource" 204
request GET "$resource" 404
request DELETE "$resource" 404
printf '%s\n' 'REST API CRUD, validation and restart persistence passed.'
