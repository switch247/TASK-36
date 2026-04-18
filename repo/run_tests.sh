#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

if ! command -v docker >/dev/null 2>&1; then
  echo "[run_tests] Docker is required in PATH" >&2
  exit 1
fi

cleanup() {
  docker compose --profile test down -v --remove-orphans >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "[run_tests] Cleaning previous test stack state"
cleanup

export API_BASE="http://app:8000/api/v1"
export ROCKET_CORS_ORIGINS="http://frontend:8080,http://localhost:8080,http://127.0.0.1:8080"

echo "[run_tests] Building and starting app stack"
docker compose up -d --build --remove-orphans db seed app frontend

MYSQL_USER="${MYSQL_USER:-eagle}"
MYSQL_PASSWORD="${MYSQL_PASSWORD:-change_this_user_password}"
MYSQL_ROOT_PASSWORD="${MYSQL_ROOT_PASSWORD:-change_this_root_password}"

echo "[run_tests] Waiting for MySQL to accept connections"
ready=0
for attempt in $(seq 1 120); do
  if docker compose exec -T db mysqladmin ping -h 127.0.0.1 -uroot -p"$MYSQL_ROOT_PASSWORD" --silent >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 2
done
if [ "$ready" -ne 1 ]; then
  echo "[run_tests] MySQL did not become ready in time" >&2
  docker compose logs db --tail 80 >&2 || true
  exit 1
fi
echo "[run_tests] MySQL is accepting connections"

echo "[run_tests] Running backend tests"
docker compose --profile test run --build --rm --no-deps backend-test

echo "[run_tests] Running frontend tests"
docker compose --profile test run --build --rm --no-deps frontend-test

echo "[run_tests] Running browser E2E tests"
docker compose --profile test run --build --rm --no-deps e2e-test
