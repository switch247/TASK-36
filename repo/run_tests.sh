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

echo "[run_tests] Running backend tests"
docker compose --profile test run --build --rm --no-deps backend-test

echo "[run_tests] Running frontend tests"
docker compose --profile test run --build --rm --no-deps frontend-test

echo "[run_tests] Running browser E2E tests"
docker compose --profile test run --build --rm --no-deps e2e-test
