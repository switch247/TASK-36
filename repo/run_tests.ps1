$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Error "[run_tests] Docker is required in PATH"
}

function Cleanup-TestStack {
    try {
        docker compose --profile test down -v --remove-orphans | Out-Null
    } catch {
    }
}

try {
    Write-Host "[run_tests] Cleaning previous test stack state"
    Cleanup-TestStack

    $env:API_BASE = "http://app:8000/api/v1"
    $env:ROCKET_CORS_ORIGINS = "http://frontend:8080,http://localhost:8080,http://127.0.0.1:8080"

    Write-Host "[run_tests] Building and starting app stack"
    docker compose up -d --build --remove-orphans db seed app frontend

    Write-Host "[run_tests] Running backend tests"
    docker compose --profile test run --build --rm --no-deps backend-test

    Write-Host "[run_tests] Running frontend tests"
    docker compose --profile test run --build --rm --no-deps frontend-test

    Write-Host "[run_tests] Running browser E2E tests"
    docker compose --profile test run --build --rm --no-deps e2e-test
} finally {
    Cleanup-TestStack
}
