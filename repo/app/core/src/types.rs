use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    Coordinator,
    Proctor,
    Auditor,
}

impl UserRole {
    pub fn can_manage_users(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    pub fn can_manage_inventory(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Coordinator)
    }

    pub fn can_view_reporting(&self) -> bool {
        matches!(
            self,
            UserRole::Admin | UserRole::Coordinator | UserRole::Auditor
        )
    }

    pub fn can_run_prints(&self) -> bool {
        matches!(
            self,
            UserRole::Admin | UserRole::Coordinator | UserRole::Proctor
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: UserRole,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub last_activity: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipCity {
    pub zip_code: String,
    pub city: String,
    pub state: Option<String>,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomCapacityStats {
    pub average_capacity: f64,
    pub room_capacity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiActor {
    pub user_id: Option<String>,
    pub role: UserRole,
    pub username: Option<String>,
}
