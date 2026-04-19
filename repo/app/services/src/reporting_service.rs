use app_models::entities::ReportSeatUtilization;
use chrono::{Duration, Utc};
use sqlx::MySqlPool;

#[derive(Clone)]
pub struct ReportingService {
    pool: MySqlPool,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct ReportNearExpiryAsset {
    pub id: String,
    pub booklet_code: String,
    pub expires_on: chrono::NaiveDate,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct ReportIncidentRate {
    pub session_id: String,
    pub avg_incidents: f64,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct ReportReturnRate {
    pub session_id: String,
    pub total_assets: i64,
    pub returned_assets: i64,
    pub return_rate_pct: f64,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct ReportMaterialInventory {
    pub asset_id: String,
    pub booklet_code: String,
    pub tracking_status: String,
    pub session_id: String,
    pub expires_on: Option<chrono::NaiveDate>,
    pub incident_count: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReportAlert {
    pub alert_type: String,
    pub severity: String,
    pub session_id: Option<String>,
    pub asset_id: Option<String>,
    pub message: String,
}

impl ReportingService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn seat_utilization(&self) -> anyhow::Result<Vec<ReportSeatUtilization>> {
        let rows = sqlx::query_as::<_, ReportSeatUtilization>(
            r#"
            SELECT r.id as room_id,
                   r.location as location,
                   r.capacity as capacity,
                   COUNT(c.id) as allocated
            FROM rooms r
            LEFT JOIN candidates c ON JSON_UNQUOTE(JSON_EXTRACT(c.metadata, '$.room_id')) = r.id
            GROUP BY r.id, r.location, r.capacity
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn near_expiry_assets(
        &self,
        within_days: i64,
    ) -> anyhow::Result<Vec<ReportNearExpiryAsset>> {
        let cutoff = Utc::now().date_naive() + Duration::days(within_days.max(1));
        let rows = sqlx::query_as::<_, ReportNearExpiryAsset>(
            r#"
            SELECT id, booklet_code, expires_on
            FROM assets
            WHERE expires_on IS NOT NULL
              AND expires_on <= ?
            "#,
        )
        .bind(cutoff)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn incident_rates(&self) -> anyhow::Result<Vec<ReportIncidentRate>> {
        let rows = sqlx::query_as::<_, ReportIncidentRate>(
            r#"
            SELECT
                session_id,
                COALESCE(CAST(AVG(COALESCE(incident_count, 0)) AS DOUBLE), 0.0) as avg_incidents
            FROM assets
            WHERE session_id IS NOT NULL AND session_id <> ''
            GROUP BY session_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn return_rates(&self) -> anyhow::Result<Vec<ReportReturnRate>> {
        let rows = sqlx::query_as::<_, ReportReturnRate>(
            r#"
            SELECT
                session_id,
                COUNT(*) as total_assets,
                -- MySQL SUM(<int expr>) returns DECIMAL by default; cast to
                -- SIGNED so sqlx decodes directly into Rust i64 without a
                -- BIGINT/DECIMAL type mismatch.
                CAST(SUM(CASE WHEN tracking_status IN ('Collected', 'Archived') THEN 1 ELSE 0 END) AS SIGNED) as returned_assets,
                CAST(CASE
                    WHEN COUNT(*) = 0 THEN 0.0
                    ELSE ROUND((SUM(CASE WHEN tracking_status IN ('Collected', 'Archived') THEN 1 ELSE 0 END) / COUNT(*) * 100.0), 2)
                END AS DOUBLE) as return_rate_pct
            FROM assets
            WHERE session_id IS NOT NULL AND session_id <> ''
            GROUP BY session_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn materials_inventory(&self) -> anyhow::Result<Vec<ReportMaterialInventory>> {
        let rows = sqlx::query_as::<_, ReportMaterialInventory>(
            r#"
            SELECT
                id as asset_id,
                booklet_code,
                tracking_status,
                session_id,
                expires_on,
                incident_count
            FROM assets
            ORDER BY booklet_code ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn operations_alerts(&self, within_days: i64) -> anyhow::Result<Vec<ReportAlert>> {
        let expiry_assets = self.near_expiry_assets(within_days).await?;
        let incident_assets = self.materials_inventory().await?;
        let return_rates = self.return_rates().await?;

        let mut alerts = Vec::new();
        for a in expiry_assets {
            alerts.push(ReportAlert {
                alert_type: "NearExpiry".to_string(),
                severity: "High".to_string(),
                session_id: None,
                asset_id: Some(a.id),
                message: format!("Asset {} expires on {}", a.booklet_code, a.expires_on),
            });
        }

        for a in incident_assets
            .into_iter()
            .filter(|x| x.incident_count >= 2)
        {
            alerts.push(ReportAlert {
                alert_type: "HighIncident".to_string(),
                severity: "Medium".to_string(),
                session_id: Some(a.session_id),
                asset_id: Some(a.asset_id),
                message: format!(
                    "Asset {} has {} incidents",
                    a.booklet_code, a.incident_count
                ),
            });
        }

        for r in return_rates
            .into_iter()
            .filter(|x| x.return_rate_pct < 80.0)
        {
            alerts.push(ReportAlert {
                alert_type: "LowReturnRate".to_string(),
                severity: "Medium".to_string(),
                session_id: Some(r.session_id),
                asset_id: None,
                message: format!("Return rate is {}%", r.return_rate_pct),
            });
        }

        Ok(alerts)
    }
}
