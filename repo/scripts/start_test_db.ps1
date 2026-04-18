$ErrorActionPreference = "Stop"

$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $root

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Error "[start_test_db] Docker is required in PATH"
}

$mysqlUser = if ($env:MYSQL_USER) { $env:MYSQL_USER } else { "eagle" }
$mysqlPassword = if ($env:MYSQL_PASSWORD) { $env:MYSQL_PASSWORD } else { "change_this_user_password" }
$mysqlDatabase = if ($env:MYSQL_DATABASE) { $env:MYSQL_DATABASE } else { "eagle_exam" }
$mysqlRootPassword = if ($env:MYSQL_ROOT_PASSWORD) { $env:MYSQL_ROOT_PASSWORD } else { "change_this_root_password" }

$printEnv = ($args.Count -ge 1 -and $args[0] -eq "--print-env")

function Log($msg) {
    if (-not $printEnv) { Write-Host $msg }
}

Log "[start_test_db] Starting db service via docker compose"
docker compose up -d --remove-orphans db | Out-Null

Log "[start_test_db] Waiting for MySQL to accept connections"
$ready = $false
for ($i = 1; $i -le 120; $i++) {
    docker compose exec -T db mysqladmin ping -h 127.0.0.1 -uroot -p"$mysqlRootPassword" --silent 2>$null | Out-Null
    if ($LASTEXITCODE -eq 0) {
        $ready = $true
        break
    }
    Start-Sleep -Seconds 2
}
if (-not $ready) {
    docker compose logs db --tail 80
    Write-Error "[start_test_db] MySQL did not become ready in time"
}
Log "[start_test_db] MySQL is ready on localhost:3306"

$testDbUrl = "mysql://${mysqlUser}:${mysqlPassword}@127.0.0.1:3306/${mysqlDatabase}"
$testAdminUrl = "mysql://root:${mysqlRootPassword}@127.0.0.1:3306/mysql"

if ($printEnv) {
    Write-Output "`$env:TEST_DATABASE_URL = '$testDbUrl'"
    Write-Output "`$env:TEST_ADMIN_DATABASE_URL = '$testAdminUrl'"
    Write-Output "`$env:TEST_JWT_SECRET = 'test-jwt-secret-change-me'"
} else {
    Write-Host "[start_test_db] Export these env vars before running cargo test:"
    Write-Host "    `$env:TEST_DATABASE_URL = '$testDbUrl'"
    Write-Host "    `$env:TEST_ADMIN_DATABASE_URL = '$testAdminUrl'"
    Write-Host "    `$env:TEST_JWT_SECRET = 'test-jwt-secret-change-me'"
    Write-Host ""
    Write-Host "Then run, e.g.:"
    Write-Host "    cargo test -p backend --test api_assets_tests"
    Write-Host "    cargo test -p backend --tests"
}
