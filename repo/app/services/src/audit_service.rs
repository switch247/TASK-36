use anyhow::Result;
use chrono::Utc;
use sqlx::MySqlPool;
use tracing::info;

#[derive(Clone)]
pub struct AuditService {
    pool: MySqlPool,
}

impl AuditService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn record_api_call(
        &self,
        user_id: Option<&str>,
        action: &str,
        resource: &str,
        ip_address: &str,
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO audit_logs (id, actor_user_id, action, resource, ip_address, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(user_id)
        .bind(action)
        .bind(resource)
        .bind(ip_address)
        .bind(Utc::now().naive_utc())
        .execute(&self.pool)
        .await?;

        info!(action = %action, resource = %resource, user_id = ?user_id, ip = %ip_address, "audit event");
        Ok(())
    }

    pub async fn record_change(
        &self,
        entity_name: &str,
        entity_id: &str,
        action: &str,
        previous_state: Option<serde_json::Value>,
        new_state: Option<serde_json::Value>,
        changed_by: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO entity_change_history (id, entity_name, entity_id, action, previous_state, new_state, changed_by, changed_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(entity_name)
        .bind(entity_id)
        .bind(action)
        .bind(previous_state)
        .bind(new_state)
        .bind(changed_by)
        .bind(Utc::now().naive_utc())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
