use bcrypt::{hash, DEFAULT_COST};
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::env;

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let database_url = env::var("DATABASE_URL")?;
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    ensure_roles_table(&pool).await?;

    let admin_id = "11111111-1111-1111-1111-111111111111";
    seed_roles(&pool).await?;
    seed_users(&pool, admin_id).await?;
    seed_templates(&pool, admin_id).await?;
    seed_rooms(&pool, admin_id).await?;
    seed_candidates(&pool, admin_id).await?;
    seed_sessions(&pool, admin_id).await?;
    seed_assets(&pool, admin_id).await?;

    println!("Seed completed successfully.");
    Ok(())
}

async fn ensure_roles_table(pool: &MySqlPool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS roles (
            id CHAR(36) PRIMARY KEY,
            name VARCHAR(64) NOT NULL UNIQUE,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn seed_roles(pool: &MySqlPool) -> anyhow::Result<()> {
    let roles = [
        ("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa", "Admin"),
        ("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb", "Coordinator"),
        ("cccccccc-cccc-cccc-cccc-cccccccccccc", "Proctor"),
        ("dddddddd-dddd-dddd-dddd-dddddddddddd", "Auditor"),
    ];

    for (id, name) in roles {
        sqlx::query("INSERT INTO roles (id, name) VALUES (?, ?) ON DUPLICATE KEY UPDATE name = VALUES(name)")
            .bind(id)
            .bind(name)
            .execute(pool)
            .await?;
    }
    Ok(())
}

async fn seed_users(pool: &MySqlPool, admin_id: &str) -> anyhow::Result<()> {
    let users = [
        (
            admin_id.to_string(),
            "admin_local".to_string(),
            env::var("SEED_ADMIN_PASSWORD").unwrap_or_else(|_| "AdminPass#2026!".to_string()),
            "Admin".to_string(),
        ),
        (
            "22222222-2222-2222-2222-222222222222".to_string(),
            "coord_local".to_string(),
            env::var("SEED_COORD_PASSWORD").unwrap_or_else(|_| "CoordPass#2026!".to_string()),
            "Coordinator".to_string(),
        ),
        (
            "33333333-3333-3333-3333-333333333333".to_string(),
            "proctor_local".to_string(),
            env::var("SEED_PROCTOR_PASSWORD")
                .unwrap_or_else(|_| "ProctorPass#2026!".to_string()),
            "Proctor".to_string(),
        ),
        (
            "44444444-4444-4444-4444-444444444444".to_string(),
            "auditor_local".to_string(),
            env::var("SEED_AUDITOR_PASSWORD")
                .unwrap_or_else(|_| "AuditorPass#2026!".to_string()),
            "Auditor".to_string(),
        ),
    ];

    for (id, username, password, role) in users {
        let password_hash = hash(password, DEFAULT_COST)?;
        sqlx::query(
            "INSERT INTO users (id, username, password_hash, role, failed_login_attempts)
             VALUES (?, ?, ?, ?, 0)
             ON DUPLICATE KEY UPDATE
                password_hash = VALUES(password_hash),
                role = VALUES(role),
                failed_login_attempts = 0,
                lockout_until = NULL",
        )
        .bind(id)
        .bind(username)
        .bind(password_hash)
        .bind(role)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_rooms(pool: &MySqlPool, admin_id: &str) -> anyhow::Result<()> {
    let rooms = [
        ("room-seed-001", 40, "Main Hall A"),
        ("room-seed-002", 30, "Lab Wing B"),
        ("room-seed-003", 25, "Annex C"),
    ];

    for (id, capacity, location) in rooms {
        sqlx::query(
            "INSERT INTO rooms (id, capacity, location, created_by)
             VALUES (?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE capacity = VALUES(capacity), location = VALUES(location)",
        )
        .bind(id)
        .bind(capacity)
        .bind(location)
        .bind(admin_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn seed_templates(pool: &MySqlPool, admin_id: &str) -> anyhow::Result<()> {
    let templates = [
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

    for (template_id, version_no, snapshot) in templates {
        sqlx::query(
            "INSERT INTO template_versions (id, template_id, version_no, snapshot, locked_for_final_print, created_by)
             VALUES (?, ?, ?, CAST(? AS JSON), FALSE, ?)
             ON DUPLICATE KEY UPDATE snapshot = VALUES(snapshot)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(template_id)
        .bind(version_no)
        .bind(snapshot.to_string())
        .bind(admin_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn seed_candidates(pool: &MySqlPool, admin_id: &str) -> anyhow::Result<()> {
    let candidates = [
        ("cand-seed-001", "ENC_DOB_001", "NAT001122", "BARCODE-001", r#"{"room_id":"room-seed-001","name":"Alice Demo"}"#),
        ("cand-seed-002", "ENC_DOB_002", "NAT001123", "BARCODE-002", r#"{"room_id":"room-seed-002","name":"Brian Demo"}"#),
        ("cand-seed-003", "ENC_DOB_003", "NAT001124", "BARCODE-003", r#"{"room_id":"room-seed-003","name":"Carla Demo"}"#),
    ];

    for (id, encrypted_dob, national_id, scanned_barcode, metadata_json) in candidates {
        sqlx::query(
            "INSERT INTO candidates (id, encrypted_dob, national_id, scanned_barcode, metadata, created_by)
             VALUES (?, ?, ?, ?, CAST(? AS JSON), ?)
             ON DUPLICATE KEY UPDATE national_id = VALUES(national_id), scanned_barcode = VALUES(scanned_barcode), metadata = VALUES(metadata)",
        )
        .bind(id)
        .bind(encrypted_dob)
        .bind(national_id)
        .bind(scanned_barcode)
        .bind(metadata_json)
        .bind(admin_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn seed_sessions(pool: &MySqlPool, admin_id: &str) -> anyhow::Result<()> {
    let sessions = [
        ("session-seed-001", "candidate-registration", 90, "Scheduled", "03/28/2026 09:00 AM", "03/28/2026 10:30 AM"),
        ("session-seed-002", "candidate-registration", 60, "Scheduled", "03/28/2026 11:00 AM", "03/28/2026 12:00 PM"),
        ("session-seed-003", "candidate-registration", 120, "Draft", "03/29/2026 08:00 AM", "03/29/2026 10:00 AM"),
    ];

    for (id, template_name, duration_minutes, status, starts_at, ends_at) in sessions {
        sqlx::query(
            "INSERT INTO exam_sessions (id, template_name, duration_minutes, status, starts_at, ends_at, created_by)
             VALUES (?, ?, ?, ?, STR_TO_DATE(?, '%m/%d/%Y %h:%i %p'), STR_TO_DATE(?, '%m/%d/%Y %h:%i %p'), ?)
             ON DUPLICATE KEY UPDATE template_name = VALUES(template_name), duration_minutes = VALUES(duration_minutes), status = VALUES(status), starts_at = VALUES(starts_at), ends_at = VALUES(ends_at)",
        )
        .bind(id)
        .bind(template_name)
        .bind(duration_minutes)
        .bind(status)
        .bind(starts_at)
        .bind(ends_at)
        .bind(admin_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn seed_assets(pool: &MySqlPool, admin_id: &str) -> anyhow::Result<()> {
    let assets = [
        ("asset-seed-001", "BOOKLET-A01", "Prepared", "session-seed-001", "2026-05-01", 0),
        ("asset-seed-002", "BOOKLET-B14", "InTransit", "session-seed-002", "2026-04-15", 1),
        ("asset-seed-003", "BOOKLET-C07", "Delivered", "session-seed-003", "2026-04-30", 0),
    ];

    for (id, booklet_code, tracking_status, session_id, expires_on, incident_count) in assets {
        sqlx::query(
            "INSERT INTO assets (id, booklet_code, tracking_status, session_id, expires_on, incident_count, created_by)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE tracking_status = VALUES(tracking_status), session_id = VALUES(session_id), expires_on = VALUES(expires_on), incident_count = VALUES(incident_count)",
        )
        .bind(id)
        .bind(booklet_code)
        .bind(tracking_status)
        .bind(session_id)
        .bind(expires_on)
        .bind(incident_count)
        .bind(admin_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}
