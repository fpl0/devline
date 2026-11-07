#!/bin/sh
# Online preparation. Offline verification is required before the release is installed.
set -eu
project=${1:?project folder}
scratch=$(mktemp -d)
cleanup() {
    status=$?
    chmod -R u+w "$scratch" 2>/dev/null || :
    rm -rf "$scratch" || :
    exit "$status"
}
trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM

tool=${DEVKIT_TOOLCHAIN:-go}
cd "$project"
export GOWORK=off GOTOOLCHAIN=local CGO_ENABLED=0
export GOMODCACHE="$scratch/resolve-cache" GOCACHE="$scratch/build-cache"
"$tool" get \
    github.com/aws/aws-sdk-go-v2@latest \
    github.com/aws/aws-sdk-go-v2/credentials@latest \
    github.com/aws/aws-sdk-go-v2/service/s3@latest \
    github.com/go-chi/chi/v5@latest \
    github.com/go-sql-driver/mysql@latest \
    github.com/golang-jwt/jwt/v5@latest \
    github.com/jackc/pgx/v5@latest \
    github.com/nats-io/nats.go@latest \
    github.com/ncruces/go-sqlite3@latest \
    github.com/rabbitmq/amqp091-go@latest \
    github.com/redis/go-redis/v9@latest
"$tool" mod tidy
"$tool" mod vendor
