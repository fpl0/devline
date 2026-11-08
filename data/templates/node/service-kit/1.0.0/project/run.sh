#!/bin/sh
set -eu
cd "$(dirname "$0")"
tool=${DEVKIT_TOOLCHAIN:-node}
if [ -n "${DEVLINE_POSTGRES_URL:-}" ] && [ -n "${DEVLINE_MYSQL_URL:-}" ] &&
   [ -n "${DEVLINE_RABBITMQ_URL:-}" ] && [ -n "${DEVLINE_REDIS_URL:-}" ] &&
   [ -n "${DEVLINE_NATS_URL:-}" ] && [ -n "${DEVLINE_SEAWEEDFS_URL:-}" ] &&
   [ -n "${DEVLINE_MEILISEARCH_URL:-}" ] && [ -n "${DEVLINE_MAILPIT_WEB_PORT:-}" ]; then
    exec "$tool" src/release-server.mjs
fi
exec "$tool" src/server.mjs
