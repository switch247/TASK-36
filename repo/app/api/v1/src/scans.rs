use rocket::serde::json::Json;
use rocket::{post, State};
use sqlx::MySqlPool;

use app_services::audit_service::AuditService;
use app_services::rbac_service::RbacService;

use crate::errors::{ApiError, ApiResult};
use crate::shared::{actor_user_id, audit, is_admin, ApiContext};

#[derive(Debug, serde::Deserialize)]
pub struct ScanLookupRequest {
    pub code: String,
    pub intent: Option<String>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct CandidateScanRow {
    pub id: String,
    pub scanned_barcode: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, serde::Serialize)]
pub struct ScanLookupResponse {
    pub code: String,
    pub found: bool,
    pub candidate_id: Option<String>,
    pub asset_id: Option<String>,
    pub asset_status: Option<String>,
    pub scanned_barcode: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub message: String,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct AssetScanRow {
    pub id: String,
    pub booklet_code: String,
    pub tracking_status: String,
    pub session_id: String,
    pub incident_count: i32,
}

#[post("/scans/lookup", data = "<payload>")]
pub async fn lookup_scan(
    payload: Json<ScanLookupRequest>,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<ScanLookupResponse>> {
    RbacService::require_print(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot perform scan lookup"))?;
    let user_id = actor_user_id(&ctx)?;
    let admin = is_admin(&ctx);

    let intent = payload
        .intent
        .as_deref()
        .unwrap_or("candidate_lookup")
        .to_ascii_lowercase();

    let response = match intent.as_str() {
        "candidate_lookup" => {
            let row = if admin {
                sqlx::query_as::<_, CandidateScanRow>(
                    "SELECT id, scanned_barcode, metadata FROM candidates WHERE scanned_barcode = ? LIMIT 1",
                )
                .bind(payload.code.trim())
                .fetch_optional(pool.inner())
                .await
                .map_err(|_| ApiError::internal("failed to look up candidate scan code"))?
            } else {
                sqlx::query_as::<_, CandidateScanRow>(
                    "SELECT id, scanned_barcode, metadata FROM candidates WHERE scanned_barcode = ? AND created_by = ? LIMIT 1",
                )
                .bind(payload.code.trim())
                .bind(user_id)
                .fetch_optional(pool.inner())
                .await
                .map_err(|_| ApiError::internal("failed to look up candidate scan code"))?
            };

            match row {
                Some(candidate) => ScanLookupResponse {
                    code: payload.code.clone(),
                    found: true,
                    candidate_id: Some(candidate.id),
                    asset_id: None,
                    asset_status: None,
                    scanned_barcode: Some(candidate.scanned_barcode),
                    metadata: Some(candidate.metadata),
                    message: "Scan matched candidate".to_string(),
                },
                None => ScanLookupResponse {
                    code: payload.code.clone(),
                    found: false,
                    candidate_id: None,
                    asset_id: None,
                    asset_status: None,
                    scanned_barcode: None,
                    metadata: None,
                    message: "No candidate matched scan code".to_string(),
                },
            }
        }
        "asset_lookup" | "booklet_lookup" => {
            let row = if admin {
                sqlx::query_as::<_, AssetScanRow>(
                    "SELECT id, booklet_code, tracking_status, session_id, incident_count FROM assets WHERE booklet_code = ? LIMIT 1",
                )
                .bind(payload.code.trim())
                .fetch_optional(pool.inner())
                .await
                .map_err(|_| ApiError::internal("failed to look up asset scan code"))?
            } else {
                sqlx::query_as::<_, AssetScanRow>(
                    "SELECT id, booklet_code, tracking_status, session_id, incident_count FROM assets WHERE booklet_code = ? AND created_by = ? LIMIT 1",
                )
                .bind(payload.code.trim())
                .bind(user_id)
                .fetch_optional(pool.inner())
                .await
                .map_err(|_| ApiError::internal("failed to look up asset scan code"))?
            };

            match row {
                Some(asset) => ScanLookupResponse {
                    code: payload.code.clone(),
                    found: true,
                    candidate_id: None,
                    asset_id: Some(asset.id),
                    asset_status: Some(asset.tracking_status),
                    scanned_barcode: Some(asset.booklet_code),
                    metadata: Some(serde_json::json!({
                        "session_id": asset.session_id,
                        "incident_count": asset.incident_count
                    })),
                    message: "Scan matched asset/booklet".to_string(),
                },
                None => ScanLookupResponse {
                    code: payload.code.clone(),
                    found: false,
                    candidate_id: None,
                    asset_id: None,
                    asset_status: None,
                    scanned_barcode: None,
                    metadata: None,
                    message: "No asset/booklet matched scan code".to_string(),
                },
            }
        }
        _ => return Err(ApiError::bad_request("unsupported scan intent")),
    };

    audit(audit_service, &ctx, "lookup_scan", "/api/v1/scans/lookup").await;
    Ok(Json(response))
}
