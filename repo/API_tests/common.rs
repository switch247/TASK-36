use std::env;
use std::sync::OnceLock;
use std::time::Duration;

use anyhow::Context;
use rocket::http::{Header, Status};
use rocket::local::asynchronous::Client;
use rocket::tokio::sync::{Mutex, MutexGuard};
use serde_json::Value;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{Executor, MySqlPool};

use app_api_v1::{catchers_v1, routes_v1};
use app_services::audit_service::AuditService;
use app_services::auth_service::AuthService;
use app_services::candidate_service::CandidateService;
use app_services::cleansing_service::CleansingService;
use app_services::messaging_service::MessagingService;
use app_services::output_service::OutputService;
use app_services::reporting_service::ReportingService;

pub const ADMIN_USERNAME: &str = "admin_local";
pub const ADMIN_PASSWORD: &str = "AdminPass#2026!";
pub const COORD_USERNAME: &str = "coord_local";
pub const COORD_PASSWORD: &str = "CoordPass#2026!";
pub const PROCTOR_USERNAME: &str = "proctor_local";
pub const PROCTOR_PASSWORD: &str = "ProctorPass#2026!";
pub const AUDITOR_USERNAME: &str = "auditor_local";
pub const AUDITOR_PASSWORD: &str = "AuditorPass#2026!";

// common.rs is included via `#[path]` into many test binaries; not every
// binary reads both `client` and `pool`, which makes them appear dead in
// those individual compilation contexts.
#[allow(dead_code)]
pub struct TestApp {
    pub client: Client,
    pub pool: MySqlPool,
    _lock_guard: MutexGuard<'static, ()>,
    admin_database_url: String,
    database_name: String,
}

impl Drop for TestApp {
    fn drop(&mut self) {
        let admin_url = self.admin_database_url.clone();
        let db_name = self.database_name.clone();
        if admin_url.is_empty() || db_name.is_empty() {
            return;
        }

        let drop_sql = format!("DROP DATABASE IF EXISTS `{db_name}`");
        let _ = std::thread::spawn(move || {
            if let Ok(rt) = rocket::tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                rt.block_on(async move {
                    if let Ok(pool) = MySqlPoolOptions::new()
                        .max_connections(1)
                        .connect(&admin_url)
                        .await
                    {
                        let _ = sqlx::query(&drop_sql).execute(&pool).await;
                    }
                });
            }
        })
        .join();
    }
}

pub async fn setup_app() -> anyhow::Result<TestApp> {
    let test_lock = global_test_lock().lock().await;

    let explicit_database_url = env::var("TEST_DATABASE_URL")
        .ok()
        .or_else(|| env::var("DATABASE_URL").ok());
    let database_url = explicit_database_url.clone().unwrap_or_else(|| {
        "mysql://eagle:change_this_user_password@localhost:3306/eagle_exam".to_string()
    });
    if explicit_database_url.is_none() {
        eprintln!(
            "[integration-tests] TEST_DATABASE_URL/DATABASE_URL not set; falling back to docker-compose default {database_url}"
        );
    }

    let jwt_secret = env::var("TEST_JWT_SECRET")
        .ok()
        .or_else(|| env::var("JWT_SECRET").ok())
        .unwrap_or_else(|| "test-jwt-secret-change-me".to_string());

    let (derived_admin_url, test_database_url, test_database_name) =
        derive_test_db_urls(&database_url)?;
    // Per-test databases require a user with CREATE DATABASE privilege. The
    // MYSQL_USER created by the mysql:8.0 image only has privileges on
    // MYSQL_DATABASE, so callers should provide TEST_ADMIN_DATABASE_URL
    // pointing at the root account. We fall back to the derived eagle@/mysql
    // URL so existing setups keep working when the eagle user has been given
    // CREATE privileges out of band.
    let admin_database_url = env::var("TEST_ADMIN_DATABASE_URL")
        .ok()
        .unwrap_or(derived_admin_url);

    let admin_pool = connect_with_retries(&admin_database_url, Duration::from_secs(60))
        .await
        .with_context(|| format!("failed to connect admin DB URL: {admin_database_url}"))?;

    // MySQL 8.0 has binary logging on by default. Without SUPER, the test
    // user (`eagle`) cannot `CREATE TRIGGER` unless the server has
    // `log_bin_trust_function_creators` enabled (error 1419). Flip it here
    // via the admin pool so tests work against any MySQL instance regardless
    // of its startup flags. Best-effort: if the admin user lacks the right
    // (e.g. caller passed a non-root TEST_ADMIN_DATABASE_URL), fall through —
    // the operator may have enabled the flag at the server command line.
    if let Err(err) = sqlx::query("SET GLOBAL log_bin_trust_function_creators = 1")
        .execute(&admin_pool)
        .await
    {
        eprintln!(
            "[integration-tests] could not SET GLOBAL log_bin_trust_function_creators=1 \
             via admin pool ({err}); relying on server-side configuration. \
             If CREATE TRIGGER later fails with 1419, grant SYSTEM_VARIABLES_ADMIN to \
             the admin user or pass --log-bin-trust-function-creators=ON to mysqld."
        );
    }

    sqlx::query(&format!("DROP DATABASE IF EXISTS `{}`", test_database_name))
        .execute(&admin_pool)
        .await
        .with_context(|| {
            format!(
                "failed to drop test database `{test_database_name}` via {admin_database_url}. \
                 The admin user must have CREATE/DROP privileges — set TEST_ADMIN_DATABASE_URL \
                 to a root-level URL (mysql://root:...@host:3306/mysql)."
            )
        })?;
    sqlx::query(&format!("CREATE DATABASE `{}`", test_database_name))
        .execute(&admin_pool)
        .await
        .with_context(|| {
            format!(
                "failed to create test database `{test_database_name}` via {admin_database_url}. \
                 The admin user must have CREATE/DROP privileges — set TEST_ADMIN_DATABASE_URL \
                 to a root-level URL (mysql://root:...@host:3306/mysql)."
            )
        })?;

    // The mysql:8.0 image only grants MYSQL_USER privileges on MYSQL_DATABASE,
    // so `eagle` cannot connect to the per-test DB without an explicit grant.
    // Propagate privileges on the freshly-created schema to every (user, host)
    // pair the test user is registered under.
    let test_user = extract_user_from_url(&database_url).unwrap_or_else(|| "eagle".to_string());
    if test_user != "root" {
        grant_test_user_on_database(&admin_pool, &test_user, &test_database_name)
            .await
            .with_context(|| {
                format!(
                    "failed to grant privileges on `{test_database_name}` to test user `{test_user}`. \
                     Admin user must have GRANT OPTION — use a root-level TEST_ADMIN_DATABASE_URL."
                )
            })?;
    }

    let pool = connect_with_retries(&test_database_url, Duration::from_secs(30))
        .await
        .with_context(|| format!("failed to connect test DB URL: {test_database_url}"))?;

    ensure_schema(&pool).await?;

    let auth_service = AuthService::new(pool.clone(), jwt_secret);
    let admin_user_id = seed_users(&pool, &auth_service).await?;
    seed_templates(&pool, &admin_user_id).await?;
    seed_zip_city_reference(&pool).await?;

    let candidate_service = CandidateService::new(pool.clone(), [7u8; 32]);
    let cleansing_service = CleansingService::new(pool.clone());
    let audit_service = AuditService::new(pool.clone());
    let reporting_service = ReportingService::new(pool.clone());
    let output_service = OutputService::new(pool.clone());
    let messaging_service = MessagingService::new(pool.clone());

    let rocket = rocket::build()
        .manage(pool.clone())
        .manage(auth_service)
        .manage(candidate_service)
        .manage(cleansing_service)
        .manage(audit_service)
        .manage(reporting_service)
        .manage(output_service)
        .manage(messaging_service)
        .mount("/api/v1", routes_v1())
        .register("/", catchers_v1());

    let client = Client::tracked(rocket).await?;
    Ok(TestApp {
        client,
        pool,
        _lock_guard: test_lock,
        admin_database_url,
        database_name: test_database_name,
    })
}

fn global_test_lock() -> &'static Mutex<()> {
    static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_MUTEX.get_or_init(|| Mutex::new(()))
}

/// Parse the username out of a `mysql://user:pw@host:port/db` URL. We only
/// need this to know which account to GRANT test-DB privileges to; other URL
/// components are not required here.
fn extract_user_from_url(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("mysql://")
        .or_else(|| url.strip_prefix("mariadb://"))?;
    let at_idx = rest.find('@')?;
    let userinfo = &rest[..at_idx];
    let user = userinfo.split(':').next()?;
    if user.is_empty() {
        None
    } else {
        Some(user.to_string())
    }
}

/// Issue `GRANT ALL PRIVILEGES ON <db>.* TO '<test_user>'@'<host>'` for every
/// host the test user is defined under in `mysql.user`. The MySQL docker
/// image creates the default user as `user@%`, but a hand-provisioned MySQL
/// may have `user@localhost` instead, so we grant for each registered host.
async fn grant_test_user_on_database(
    admin_pool: &MySqlPool,
    test_user: &str,
    database_name: &str,
) -> anyhow::Result<()> {
    // `mysql.user.host` is stored as CHAR with a binary collation in some
    // server distributions (notably MariaDB 10+/12). sqlx's MySQL driver
    // rejects decoding a BINARY column as Rust `String`, so cast explicitly
    // to CHAR to get a textual column regardless of server flavor.
    let hosts: Vec<String> = sqlx::query_scalar::<_, String>(
        "SELECT CAST(host AS CHAR) FROM mysql.user WHERE user = ?",
    )
    .bind(test_user)
    .fetch_all(admin_pool)
    .await
    .with_context(|| format!("failed to enumerate mysql.user hosts for `{test_user}`"))?;

    if hosts.is_empty() {
        return Err(anyhow::anyhow!(
            "user `{test_user}` does not exist in mysql.user — cannot grant privileges on `{database_name}`"
        ));
    }

    for host in hosts {
        let grant_sql = format!(
            "GRANT ALL PRIVILEGES ON `{}`.* TO '{}'@'{}'",
            database_name.replace('`', "``"),
            test_user.replace('\'', "''"),
            host.replace('\'', "''"),
        );
        sqlx::query(&grant_sql)
            .execute(admin_pool)
            .await
            .with_context(|| format!("failed to execute `{grant_sql}`"))?;
    }
    Ok(())
}

/// Connect to MySQL, retrying for up to `max_wait` to tolerate a DB that is
/// still starting up. Each individual connection attempt has its own short
/// acquire timeout so we poll rather than block.
async fn connect_with_retries(url: &str, max_wait: Duration) -> anyhow::Result<MySqlPool> {
    let deadline = std::time::Instant::now() + max_wait;
    loop {
        match MySqlPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .connect(url)
            .await
        {
            Ok(pool) => return Ok(pool),
            Err(err) => {
                if std::time::Instant::now() >= deadline {
                    return Err(anyhow::Error::new(err));
                }
                rocket::tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}

fn derive_test_db_urls(database_url: &str) -> anyhow::Result<(String, String, String)> {
    let (base, query_suffix) = match database_url.split_once('?') {
        Some((b, q)) => (b, format!("?{q}")),
        None => (database_url, String::new()),
    };
    let slash_idx = base
        .rfind('/')
        .ok_or_else(|| anyhow::anyhow!("invalid TEST_DATABASE_URL"))?;
    let host_prefix = &base[..slash_idx];
    let test_db_name = format!("eagle_exam_test_{}", uuid::Uuid::new_v4().simple());
    let admin_db_url = format!("{host_prefix}/mysql{query_suffix}");
    let test_db_url = format!("{host_prefix}/{test_db_name}{query_suffix}");
    Ok((admin_db_url, test_db_url, test_db_name))
}

pub async fn login(client: &Client, username: &str, password: &str) -> (Status, Option<Value>) {
    let response = client
        .post("/api/v1/auth/login")
        .json(&serde_json::json!({ "username": username, "password": password }))
        .dispatch()
        .await;

    let status = response.status();
    let body = response.into_json::<Value>().await;
    (status, body)
}

pub fn auth_headers(body: &Value) -> Vec<Header<'static>> {
    let jwt = body
        .get("jwt")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let session_id = body
        .get("session_id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    vec![
        Header::new("Authorization", format!("Bearer {jwt}")),
        Header::new("x-session-id", session_id),
    ]
}

pub fn attach_auth<'a>(
    mut req: rocket::local::asynchronous::LocalRequest<'a>,
    headers: &[Header<'static>],
) -> rocket::local::asynchronous::LocalRequest<'a> {
    // Rocket 0.5's local client rejects URIs containing literal spaces (or
    // any non-RFC-3986 path char) with a 400 before dispatch. Several tests
    // write URLs like `/api/v1/templates/Template A/1` with a literal space
    // in the segment. Percent-encode unsafe bytes in the path and install a
    // fresh Origin so routing can succeed — the route's `<template_id>`
    // capture decodes back to the original string.
    let uri_str = req.uri().to_string();
    if uri_str
        .bytes()
        .any(|b| !b.is_ascii() || b == b' ' || b == b'"' || b == b'<' || b == b'>' || b == b'`' || b == b'\\' || b == b'{' || b == b'}' || b == b'|' || b == b'^')
    {
        let encoded: String = uri_str
            .bytes()
            .map(|b| match b {
                b' ' => "%20".to_string(),
                b'"' => "%22".to_string(),
                b'<' => "%3C".to_string(),
                b'>' => "%3E".to_string(),
                b'`' => "%60".to_string(),
                b'\\' => "%5C".to_string(),
                b'{' => "%7B".to_string(),
                b'}' => "%7D".to_string(),
                b'|' => "%7C".to_string(),
                b'^' => "%5E".to_string(),
                _ => (b as char).to_string(),
            })
            .collect();
        if let Ok(new_uri) = rocket::http::uri::Origin::parse_owned(encoded) {
            // LocalRequest derefs to Request, which exposes set_uri.
            req.set_uri(new_uri);
        }
    }
    for h in headers {
        req = req.header(h.clone());
    }
    req
}

/// Log in as one of the seeded roles and return auth headers suitable for `attach_auth`.
#[allow(dead_code)]
pub async fn login_as(client: &Client, role: Role) -> Vec<Header<'static>> {
    let (user, pass) = role.credentials();
    let (status, body) = login(client, user, pass).await;
    assert_eq!(status, Status::Ok, "login failed for {:?}", role);
    auth_headers(&body.expect("login body"))
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Role {
    Admin,
    Coordinator,
    Proctor,
    Auditor,
}

impl Role {
    #[allow(dead_code)]
    pub fn credentials(self) -> (&'static str, &'static str) {
        match self {
            Role::Admin => (ADMIN_USERNAME, ADMIN_PASSWORD),
            Role::Coordinator => (COORD_USERNAME, COORD_PASSWORD),
            Role::Proctor => (PROCTOR_USERNAME, PROCTOR_PASSWORD),
            Role::Auditor => (AUDITOR_USERNAME, AUDITOR_PASSWORD),
        }
    }
}

/// Return the seeded user id for a role.
#[allow(dead_code)]
pub async fn user_id_for(pool: &MySqlPool, username: &str) -> String {
    sqlx::query_scalar::<_, String>("SELECT id FROM users WHERE username = ?")
        .bind(username)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|e| panic!("failed to fetch user id for {username}: {e}"))
}

/// Insert a room directly via SQL owned by the given user (coordinator or admin).
#[allow(dead_code)]
pub async fn factory_room(
    pool: &MySqlPool,
    id: &str,
    capacity: i32,
    location: &str,
    owner_id: &str,
) {
    sqlx::query("INSERT INTO rooms (id, capacity, location, created_by) VALUES (?, ?, ?, ?)")
        .bind(id)
        .bind(capacity)
        .bind(location)
        .bind(owner_id)
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("factory_room failed: {e}"));
}

/// Insert an exam session owned by `owner_id`.
#[allow(dead_code)]
pub async fn factory_session(
    pool: &MySqlPool,
    id: &str,
    template_name: &str,
    duration_minutes: i32,
    owner_id: &str,
) {
    sqlx::query(
        "INSERT INTO exam_sessions (id, template_name, duration_minutes, status, starts_at, ends_at, locked_for_final_print, created_by)
         VALUES (?, ?, ?, 'Scheduled', UTC_TIMESTAMP(), DATE_ADD(UTC_TIMESTAMP(), INTERVAL ? MINUTE), FALSE, ?)",
    )
    .bind(id)
    .bind(template_name)
    .bind(duration_minutes)
    .bind(duration_minutes)
    .bind(owner_id)
    .execute(pool)
    .await
    .unwrap_or_else(|e| panic!("factory_session failed: {e}"));
}

/// Insert an asset owned by `owner_id` attached to a session.
#[allow(dead_code)]
pub async fn factory_asset(
    pool: &MySqlPool,
    id: &str,
    booklet_code: &str,
    session_id: &str,
    owner_id: &str,
) {
    sqlx::query(
        "INSERT INTO assets (id, booklet_code, tracking_status, session_id, incident_count, created_by)
         VALUES (?, ?, 'Prepared', ?, 0, ?)",
    )
    .bind(id)
    .bind(booklet_code)
    .bind(session_id)
    .bind(owner_id)
    .execute(pool)
    .await
    .unwrap_or_else(|e| panic!("factory_asset failed: {e}"));
}

/// Create a candidate via the HTTP API with sensible defaults. Accepts the role's auth headers.
#[allow(dead_code)]
pub async fn factory_candidate_http(
    client: &Client,
    headers: &[Header<'static>],
    id: &str,
    national_id: &str,
    scanned_barcode: &str,
) {
    let payload = serde_json::json!({
        "candidate_id": id,
        "date_of_birth": "03/27/2001",
        "national_id": national_id,
        "scanned_barcode": scanned_barcode,
        "metadata_json": format!("{{\"name\":\"Candidate {id}\",\"room_id\":\"room-a\"}}")
    });
    let resp = attach_auth(client.post("/api/v1/candidates").json(&payload), headers)
        .dispatch()
        .await;
    assert_eq!(
        resp.status(),
        Status::Created,
        "factory_candidate_http failed for {id}"
    );
}

/// Create a second user for cross-ownership tests. Returns (user_id, username).
#[allow(dead_code)]
pub async fn factory_user(
    client: &Client,
    pool: &MySqlPool,
    username: &str,
    password: &str,
    role: &str,
) -> String {
    let auth_service = client
        .rocket()
        .state::<app_services::auth_service::AuthService>()
        .expect("auth service state");
    let hashed = auth_service.hash_password(password).expect("hash");
    let user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, role, failed_login_attempts) VALUES (?, ?, ?, ?, 0)",
    )
    .bind(&user_id)
    .bind(username)
    .bind(hashed)
    .bind(role)
    .execute(pool)
    .await
    .expect("insert factory user");
    user_id
}

/// Assert that a JSON body has a field equal to the given JSON value.
#[allow(dead_code)]
pub fn assert_field_eq(body: &Value, path: &str, expected: Value) {
    let mut cur = body;
    for seg in path.split('.') {
        cur = cur
            .get(seg)
            .unwrap_or_else(|| panic!("missing field path '{path}' in body {body}"));
    }
    assert_eq!(cur, &expected, "path={path} body={body}");
}

/// Count rows in a table with a simple WHERE clause.
#[allow(dead_code)]
pub async fn count_rows(pool: &MySqlPool, sql: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(sql)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|e| panic!("count_rows failed for `{sql}`: {e}"))
}

async fn ensure_schema(pool: &MySqlPool) -> anyhow::Result<()> {
    let scripts = [
        include_str!("../app/models/migrations/001_init.sql"),
        include_str!("../app/models/migrations/002_seed_zip_city.sql"),
        include_str!("../app/models/migrations/003_print_output_template_ref.sql"),
        include_str!("../app/models/migrations/004_candidate_uniqueness.sql"),
        include_str!("../app/models/migrations/005_attachment_blob.sql"),
        include_str!("../app/models/migrations/006_session_assignments.sql"),
        include_str!("../app/models/migrations/007_print_outputs_created_at_precision.sql"),
    ];
    for script in scripts {
        execute_migration_script(pool, script).await?;
    }

    Ok(())
}

async fn execute_migration_script(pool: &MySqlPool, script: &str) -> anyhow::Result<()> {
    // Defensively strip a leading UTF-8 BOM (U+FEFF). Some editors save SQL
    // files with a BOM on Windows, which MySQL rejects as a 1064 syntax error
    // because the invisible bytes appear before `CREATE TABLE`.
    let script = script.strip_prefix('\u{feff}').unwrap_or(script);

    // Acquire one connection for the whole script so user/session variables
    // (e.g. `@sql`, `@template_id_exists`) set by conditional-DDL migrations
    // persist across every statement in the file.
    let mut conn = pool.acquire().await?;

    let mut delimiter = ";".to_string();
    let mut statement = String::new();

    for raw_line in script.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("DELIMITER ") {
            delimiter = rest.trim().to_string();
            continue;
        }

        statement.push_str(raw_line);
        statement.push('\n');

        if statement.trim_end().ends_with(&delimiter) {
            let mut finalized = statement.trim_end().to_string();
            if finalized.ends_with(&delimiter) {
                let new_len = finalized.len() - delimiter.len();
                finalized.truncate(new_len);
            }
            let stmt = finalized.trim();
            if !stmt.is_empty() {
                execute_raw(&mut conn, stmt).await?;
            }
            statement.clear();
        }
    }

    let tail = statement.trim();
    if !tail.is_empty() {
        execute_raw(&mut conn, tail).await?;
    }

    Ok(())
}

/// Execute a migration statement via MySQL's text protocol (`COM_QUERY`).
/// Passing a `&str` directly to `Executor::execute` yields a query with no
/// arguments, which the sqlx MySQL driver sends unprepared. This avoids
/// `ER_UNSUPPORTED_PS` (1295) for statements that cannot go through
/// `COM_STMT_PREPARE` — notably `PREPARE` / `EXECUTE` / `DEALLOCATE PREPARE`,
/// used by conditional-DDL migrations to add columns/indexes idempotently.
async fn execute_raw(conn: &mut sqlx::MySqlConnection, sql: &str) -> anyhow::Result<()> {
    conn.execute(sql)
        .await
        .with_context(|| format!("failed to execute migration SQL: {sql}"))?;
    Ok(())
}

// Available as test-infrastructure for per-test data cleanup; per-database
// isolation makes it unused today, keep for ad-hoc test rewrites.
#[allow(dead_code)]
async fn reset_data(pool: &MySqlPool) -> anyhow::Result<()> {
    let resets = [
        // audit_logs is immutable via DELETE trigger; TRUNCATE bypasses row triggers in MySQL.
        "TRUNCATE TABLE audit_logs",
        "TRUNCATE TABLE entity_change_history",
        "TRUNCATE TABLE template_versions",
        "TRUNCATE TABLE merge_candidates",
        "TRUNCATE TABLE attachments",
        "TRUNCATE TABLE print_outputs",
        "TRUNCATE TABLE exam_session_assignments",
        "TRUNCATE TABLE assets",
        "TRUNCATE TABLE exam_sessions",
        "TRUNCATE TABLE rooms",
        "TRUNCATE TABLE candidates",
        "TRUNCATE TABLE zip_city_reference",
        "TRUNCATE TABLE message_drafts",
        "TRUNCATE TABLE user_sessions",
        "TRUNCATE TABLE users",
    ];

    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(pool)
        .await?;
    for stmt in resets {
        sqlx::query(stmt).execute(pool).await?;
    }
    sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(pool)
        .await?;

    Ok(())
}

async fn seed_users(pool: &MySqlPool, auth: &AuthService) -> anyhow::Result<String> {
    let mut admin_user_id = String::new();
    let users = [
        (ADMIN_USERNAME, ADMIN_PASSWORD, "Admin"),
        (COORD_USERNAME, COORD_PASSWORD, "Coordinator"),
        (PROCTOR_USERNAME, PROCTOR_PASSWORD, "Proctor"),
        (AUDITOR_USERNAME, AUDITOR_PASSWORD, "Auditor"),
    ];

    for (username, password, role) in users {
        let hashed = auth.hash_password(password)?;
        let user_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO users (id, username, password_hash, role, failed_login_attempts) VALUES (?, ?, ?, ?, 0)",
        )
        .bind(&user_id)
        .bind(username)
        .bind(hashed)
        .bind(role)
        .execute(pool)
        .await?;
        if username == ADMIN_USERNAME {
            admin_user_id = user_id;
        }
    }

    Ok(admin_user_id)
}

async fn seed_templates(pool: &MySqlPool, created_by: &str) -> anyhow::Result<()> {
    let snapshots = [
        (
            "base-template",
            1_i32,
            serde_json::json!({
                "rules": {
                    "id": ["Required"],
                    "duration_minutes": ["Required", {"Range":{"min":15.0,"max":360.0}}],
                    "status": ["Required"],
                    "starts_at": ["Required"],
                    "ends_at": ["Required"]
                },
                "admit_card": {"title": "Base Admit Card"},
                "seating_chart": {"title": "Base Seating Chart"},
                "door_sign": {"title": "Base Door Sign"},
                "proctor_packet": {"checklist": ["Check IDs", "Verify assets"]},
                "summary_report": {"title": "Base Summary Report"}
            }),
        ),
        (
            "Template A",
            1_i32,
            serde_json::json!({
                "rules": {
                    "id": ["Required"],
                    "duration_minutes": ["Required", {"Range":{"min":15.0,"max":360.0}}],
                    "status": ["Required"],
                    "starts_at": ["Required"],
                    "ends_at": ["Required"]
                },
                "admit_card": {"title": "Template A Admit Card"},
                "seating_chart": {"title": "Template A Seating Chart"},
                "door_sign": {"title": "Template A Door Sign"},
                "proctor_packet": {"checklist": ["Room prep", "Attendance", "Incident tracking"]},
                "summary_report": {"title": "Template A Summary Report"}
            }),
        ),
        (
            "candidate-registration",
            1_i32,
            serde_json::json!({
                "rules": {
                    "date_of_birth": ["Required"],
                    "national_id": ["Required"],
                    "scanned_barcode": ["Required"],
                    "name": ["Required"]
                },
                "admit_card": {"title": "Candidate Registration Admit Card"},
                "summary_report": {"title": "Candidate Registration Summary"}
            }),
        ),
        (
            "room-config",
            1_i32,
            serde_json::json!({
                "rules": {
                    "id": ["Required"],
                    "capacity": ["Required", {"Range":{"min":1.0,"max":500.0}}],
                    "location": ["Required"]
                }
            }),
        ),
        (
            "proctor-profile",
            1_i32,
            serde_json::json!({
                "rules": {
                    "username": ["Required"],
                    "role": ["Required"]
                }
            }),
        ),
    ];

    for (template_id, version_no, snapshot) in snapshots {
        // MySQL 8 and MariaDB both auto-validate a JSON-shaped string bound
        // into a JSON column, so avoid `CAST(? AS JSON)` — MariaDB rejects
        // that construct when the argument arrives via the binary prepared-
        // statement protocol.
        sqlx::query(
            "INSERT INTO template_versions (id, template_id, version_no, snapshot, locked_for_final_print, created_by)
             VALUES (?, ?, ?, ?, FALSE, ?)
             ON DUPLICATE KEY UPDATE snapshot = VALUES(snapshot)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(template_id)
        .bind(version_no)
        .bind(snapshot.to_string())
        .bind(created_by)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_zip_city_reference(pool: &MySqlPool) -> anyhow::Result<()> {
    let rows = [
        ("00100", "Nairobi", Some("Nairobi County"), "KE"),
        ("20100", "Nakuru", Some("Nakuru County"), "KE"),
        ("40100", "Kisumu", Some("Kisumu County"), "KE"),
    ];

    for (zip_code, city, state, country) in rows {
        sqlx::query(
            "INSERT INTO zip_city_reference (zip_code, city, state, country)
             VALUES (?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE city = VALUES(city), state = VALUES(state), country = VALUES(country)",
        )
        .bind(zip_code)
        .bind(city)
        .bind(state)
        .bind(country)
        .execute(pool)
        .await?;
    }

    Ok(())
}
