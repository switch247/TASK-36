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

    $env:API_BASE = "http://app:8001/api/v1"
    $env:ROCKET_CORS_ORIGINS = "http://frontend:8080,http://localhost:8080,http://127.0.0.1:8080"

    Write-Host "[run_tests] Building and starting app stack"
    docker compose up -d --build --remove-orphans db seed app frontend

    $rootPassword = if ($env:MYSQL_ROOT_PASSWORD) { $env:MYSQL_ROOT_PASSWORD } else { "change_this_root_password" }

    Write-Host "[run_tests] Waiting for MySQL to accept connections"
    $ready = $false
    for ($i = 1; $i -le 120; $i++) {
        docker compose exec -T db mysqladmin ping -h 127.0.0.1 -uroot -p"$rootPassword" --silent 2>$null | Out-Null
        if ($LASTEXITCODE -eq 0) {
            $ready = $true
            break
        }
        Start-Sleep -Seconds 2
    }
    if (-not $ready) {
        docker compose logs db --tail 80
        Write-Error "[run_tests] MySQL did not become ready in time"
    }
    Write-Host "[run_tests] MySQL is accepting connections"

    Write-Host "[run_tests] Running backend tests"
    docker compose --profile test run --build --rm --no-deps backend-test

    Write-Host "[run_tests] Running frontend tests"
    docker compose --profile test run --build --rm --no-deps frontend-test

    Write-Host "[run_tests] Running browser E2E tests"
    docker compose --profile test run --build --rm --no-deps e2e-test
} finally {
    Cleanup-TestStack
}
