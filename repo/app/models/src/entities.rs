use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use app_core::types::UserRole;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub failed_login_attempts: i32,
    pub lockout_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Candidate {
    pub id: String,
    pub encrypted_dob: String,
    pub national_id: String,
    pub scanned_barcode: String,
    pub metadata: serde_json::Value,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Room {
    pub id: String,
    pub capacity: i32,
    pub location: String,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExamSession {
    pub id: String,
    pub template_name: String,
    pub duration_minutes: i32,
    pub status: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub locked_for_final_print: bool,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Asset {
    pub id: String,
    pub booklet_code: String,
    pub tracking_status: String,
    pub session_id: String,
    pub expires_on: Option<chrono::NaiveDate>,
    pub incident_count: i32,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: String,
    pub actor_user_id: Option<String>,
    pub action: String,
    pub resource: String,
    pub ip_address: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ZipCityRow {
    pub zip_code: String,
    pub city: String,
    pub state: Option<String>,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportSeatUtilization {
    pub room_id: String,
    pub location: String,
    pub capacity: i32,
    pub allocated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportNearExpiryAsset {
    pub id: String,
    pub booklet_code: String,
    pub expires_on: chrono::NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportIncidentRate {
    pub session_id: String,
    pub avg_incidents: f64,
}

impl User {
    pub fn parsed_role(&self) -> UserRole {
        match self.role.as_str() {
            "Admin" | "Administrator" => UserRole::Admin,
            "Coordinator" | "Exam Coordinator" => UserRole::Coordinator,
            "Proctor" => UserRole::Proctor,
            _ => UserRole::Auditor,
        }
    }
}
