#!/usr/bin/env bash
# Spin up *only* the MySQL service used by the integration test suite and wait
# for it to accept connections. Prints the env vars the caller should export
# before running `cargo test` locally.
#
# Usage:
#   ./scripts/start_test_db.sh
#   eval "$(./scripts/start_test_db.sh --print-env)"
#   cargo test -p backend --tests
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v docker >/dev/null 2>&1; then
  echo "[start_test_db] Docker is required in PATH" >&2
  exit 1
fi

MYSQL_USER="${MYSQL_USER:-eagle}"
MYSQL_PASSWORD="${MYSQL_PASSWORD:-change_this_user_password}"
MYSQL_DATABASE="${MYSQL_DATABASE:-eagle_exam}"
MYSQL_ROOT_PASSWORD="${MYSQL_ROOT_PASSWORD:-change_this_root_password}"

PRINT_ENV=0
if [ "${1:-}" = "--print-env" ]; then
  PRINT_ENV=1
fi

log() {
  if [ "$PRINT_ENV" -eq 0 ]; then
    echo "$@"
  fi
}

log "[start_test_db] Starting db service via docker compose"
docker compose up -d --remove-orphans db >/dev/null

log "[start_test_db] Waiting for MySQL to accept connections"
ready=0
for attempt in $(seq 1 120); do
  if docker compose exec -T db mysqladmin ping -h 127.0.0.1 -uroot -p"$MYSQL_ROOT_PASSWORD" --silent >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 2
done
if [ "$ready" -ne 1 ]; then
  echo "[start_test_db] MySQL did not become ready in time" >&2
  docker compose logs db --tail 80 >&2 || true
  exit 1
fi
log "[start_test_db] MySQL is ready on localhost:3306"

TEST_DATABASE_URL="mysql://${MYSQL_USER}:${MYSQL_PASSWORD}@127.0.0.1:3306/${MYSQL_DATABASE}"
TEST_ADMIN_DATABASE_URL="mysql://root:${MYSQL_ROOT_PASSWORD}@127.0.0.1:3306/mysql"

if [ "$PRINT_ENV" -eq 1 ]; then
  echo "export TEST_DATABASE_URL='${TEST_DATABASE_URL}'"
  echo "export TEST_ADMIN_DATABASE_URL='${TEST_ADMIN_DATABASE_URL}'"
  echo "export TEST_JWT_SECRET='test-jwt-secret-change-me'"
else
  cat <<EOF
[start_test_db] Export these env vars before running cargo test:
    export TEST_DATABASE_URL='${TEST_DATABASE_URL}'
    export TEST_ADMIN_DATABASE_URL='${TEST_ADMIN_DATABASE_URL}'
    export TEST_JWT_SECRET='test-jwt-secret-change-me'

Then run, e.g.:
    cargo test -p backend --test api_assets_tests
    cargo test -p backend --tests
EOF
fi
