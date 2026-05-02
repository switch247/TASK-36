#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

if ! command -v docker >/dev/null 2>&1; then
  echo "[run_tests] Docker is required in PATH" >&2
  exit 1
fi

TEST_RUNNER_IMAGE="${TEST_RUNNER_IMAGE:-eagle_test_runner:local}"
COMPOSE_PROJECT="${COMPOSE_PROJECT_NAME:-$(basename "$ROOT" | tr '[:upper:]' '[:lower:]')}"
COMPOSE_NETWORK="${COMPOSE_PROJECT}_default"
REBUILD_STACK="${REBUILD_STACK:-1}"

cleanup() {
  docker compose down -v --remove-orphans >/dev/null 2>&1 || true
}
trap cleanup EXIT

run_in_test_runner() {
  docker run --rm \
    --network "$COMPOSE_NETWORK" \
    -v "$ROOT:/workspace" \
    -w /workspace \
    -e TEST_DATABASE_URL="mysql://${MYSQL_USER:-eagle}:${MYSQL_PASSWORD:-change_this_user_password}@db:3306/${MYSQL_DATABASE:-eagle_exam}" \
    -e TEST_ADMIN_DATABASE_URL="mysql://root:${MYSQL_ROOT_PASSWORD:-change_this_root_password}@db:3306/mysql" \
    -e TEST_JWT_SECRET="${JWT_SECRET:-replace_with_long_random_secret}" \
    -e JWT_SECRET="${JWT_SECRET:-replace_with_long_random_secret}" \
    -e DATABASE_URL="mysql://${MYSQL_USER:-eagle}:${MYSQL_PASSWORD:-change_this_user_password}@db:3306/${MYSQL_DATABASE:-eagle_exam}" \
    -e FRONTEND_URL="http://frontend" \
    -e BACKEND_URL="http://app:8001/api/v1" \
    -e ADMIN_USERNAME="${BOOTSTRAP_ADMIN_USERNAME:-admin_local}" \
    -e ADMIN_PASSWORD="${BOOTSTRAP_ADMIN_PASSWORD:-AdminPass#2026!}" \
    -e COORD_USERNAME="${BOOTSTRAP_COORDINATOR_USERNAME:-coord_local}" \
    -e COORD_PASSWORD="${BOOTSTRAP_COORDINATOR_PASSWORD:-CoordPass#2026!}" \
    -e PROCTOR_USERNAME="${BOOTSTRAP_PROCTOR_USERNAME:-proctor_local}" \
    -e PROCTOR_PASSWORD="${BOOTSTRAP_PROCTOR_PASSWORD:-ProctorPass#2026!}" \
    -e AUDITOR_USERNAME="${BOOTSTRAP_AUDITOR_USERNAME:-auditor_local}" \
    -e AUDITOR_PASSWORD="${BOOTSTRAP_AUDITOR_PASSWORD:-AuditorPass#2026!}" \
    "$TEST_RUNNER_IMAGE" "$@"
}

ensure_test_runner_image() {
  if docker image inspect "$TEST_RUNNER_IMAGE" >/dev/null 2>&1 && [ "${REBUILD_TEST_RUNNER:-0}" != "1" ]; then
    echo "[run_tests] Reusing cached test runner image $TEST_RUNNER_IMAGE"
    return
  fi

  echo "[run_tests] Building reusable test runner image $TEST_RUNNER_IMAGE"
  docker build --target test-runner -t "$TEST_RUNNER_IMAGE" .
}

wait_for_http() {
  local url="$1"
  local name="$2"
  local ready=0
  for attempt in $(seq 1 120); do
    if run_in_test_runner node -e "
      const url = process.argv[1];
      fetch(url, { redirect: 'manual' })
        .then(() => process.exit(0))
        .catch(() => process.exit(1));
    " "$url" >/dev/null 2>&1; then
      ready=1
      break
    fi
    sleep 2
  done

  if [ "$ready" -ne 1 ]; then
    echo "[run_tests] $name did not become ready in time" >&2
    return 1
  fi
}

echo "[run_tests] Cleaning previous test stack state"
cleanup

export API_BASE="http://app:8001/api/v1"
export ROCKET_CORS_ORIGINS="http://frontend,http://frontend:8080,http://localhost:8080,http://127.0.0.1:8080"

if [ "$REBUILD_STACK" = "1" ]; then
  echo "[run_tests] Rebuilding and starting app stack"
  docker compose up -d --build --remove-orphans db seed app frontend
else
  echo "[run_tests] Starting app stack without rebuild (set REBUILD_STACK=1 to rebuild)"
  docker compose up -d --remove-orphans db seed app frontend
fi

MYSQL_USER="${MYSQL_USER:-eagle}"
MYSQL_PASSWORD="${MYSQL_PASSWORD:-change_this_user_password}"
MYSQL_ROOT_PASSWORD="${MYSQL_ROOT_PASSWORD:-change_this_root_password}"
MYSQL_DATABASE="${MYSQL_DATABASE:-eagle_exam}"

ensure_test_runner_image

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

echo "[run_tests] Waiting for backend HTTP"
wait_for_http "http://app:8001/api/v1/health" "Backend"

echo "[run_tests] Waiting for frontend HTTP"
wait_for_http "http://frontend" "Frontend"

echo "[run_tests] Running backend tests"
run_in_test_runner cargo test -p backend --tests

echo "[run_tests] Running frontend tests"
run_in_test_runner cargo test -p frontend --lib --tests --bin frontend

echo "[run_tests] Running browser E2E tests"
run_in_test_runner node /workspace/e2e/run_all_e2e.js
