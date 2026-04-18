use rocket::post;
use rocket::serde::json::Json;
use rocket::State;

use app_services::audit_service::AuditService;
use app_services::output_service::OutputService;
use app_services::rbac_service::RbacService;
use app_services::reporting_service::ReportingService;

use crate::errors::{ApiError, ApiResult};
use crate::shared::{audit, ApiContext};

#[derive(Debug, serde::Deserialize)]
pub struct ExportRequest {
    pub report: String,
    pub within_days: Option<i64>,
    pub filter: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Serialize)]
pub struct ExportResponse {
    pub content: String,
}

#[post("/exports/csv", data = "<payload>")]
pub async fn export_csv(
    payload: Json<ExportRequest>,
    reporting_service: &State<ReportingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<ExportResponse>> {
    RbacService::require_reporting(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("Role cannot export data"))?;

    let (rows, fields) = build_export_dataset(reporting_service, &payload)
        .await
        .map_err(|err| ApiError::bad_request(err.as_str()))?;
    let field_refs: Vec<&str> = fields.iter().map(String::as_str).collect();

    let content = OutputService::export_csv_whitelisted(&rows, &field_refs)
        .map_err(|_| ApiError::bad_request("invalid export request"))?;

    audit(audit_service, &ctx, "export_data", "/api/v1/exports/csv").await;
    Ok(Json(ExportResponse { content }))
}

#[post("/exports/excel", data = "<payload>")]
pub async fn export_excel(
    payload: Json<ExportRequest>,
    reporting_service: &State<ReportingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<ExportResponse>> {
    RbacService::require_reporting(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("Role cannot export data"))?;

    let (rows, fields) = build_export_dataset(reporting_service, &payload)
        .await
        .map_err(|err| ApiError::bad_request(err.as_str()))?;
    let field_refs: Vec<&str> = fields.iter().map(String::as_str).collect();

    let content = OutputService::export_excel_like_tsv(&rows, &field_refs)
        .map_err(|_| ApiError::bad_request("invalid export request"))?;

    audit(audit_service, &ctx, "export_data", "/api/v1/exports/excel").await;
    Ok(Json(ExportResponse { content }))
}

#[post("/exports/pdf", data = "<payload>")]
pub async fn export_pdf(
    payload: Json<ExportRequest>,
    reporting_service: &State<ReportingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<ExportResponse>> {
    RbacService::require_reporting(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("Role cannot export data"))?;

    let (rows, fields) = build_export_dataset(reporting_service, &payload)
        .await
        .map_err(|err| ApiError::bad_request(err.as_str()))?;
    let field_refs: Vec<&str> = fields.iter().map(String::as_str).collect();

    let csv = OutputService::export_csv_whitelisted(&rows, &field_refs)
        .map_err(|_| ApiError::bad_request("invalid export request"))?;
    let content =
        OutputService::export_pdf_placeholder(&format!("{} Export", payload.report), &csv);

    audit(audit_service, &ctx, "export_data", "/api/v1/exports/pdf").await;
    Ok(Json(ExportResponse { content }))
}

async fn build_export_dataset(
    reporting_service: &State<ReportingService>,
    payload: &ExportRequest,
) -> Result<(Vec<serde_json::Value>, Vec<String>), String> {
    let report_key = payload.report.trim().to_ascii_lowercase();
    let limit = payload.limit.unwrap_or(500).clamp(1, 500) as usize;
    let filter = payload.filter.as_deref().map(str::to_lowercase);

    let mut rows = match report_key.as_str() {
        "seat_utilization" => reporting_service
            .seat_utilization()
            .await
            .map_err(|_| "failed to load seat utilization report".to_string())?
            .into_iter()
            .map(|r| serde_json::json!({
                "room_id": r.room_id,
                "location": r.location,
                "capacity": r.capacity,
                "allocated": r.allocated
            }))
            .collect::<Vec<_>>(),
        "near_expiry_assets" => reporting_service
            .near_expiry_assets(payload.within_days.unwrap_or(30))
            .await
            .map_err(|_| "failed to load near expiry assets report".to_string())?
            .into_iter()
            .map(|r| serde_json::json!({
                "asset_id": r.id,
                "booklet_code": r.booklet_code,
                "expires_on": r.expires_on
            }))
            .collect::<Vec<_>>(),
        "incident_rates" => reporting_service
            .incident_rates()
            .await
            .map_err(|_| "failed to load incident rates report".to_string())?
            .into_iter()
            .map(|r| serde_json::json!({
                "session_id": r.session_id,
                "avg_incidents": r.avg_incidents
            }))
            .collect::<Vec<_>>(),
        "return_rates" => reporting_service
            .return_rates()
            .await
            .map_err(|_| "failed to load return rates report".to_string())?
            .into_iter()
            .map(|r| serde_json::json!({
                "session_id": r.session_id,
                "total_assets": r.total_assets,
                "returned_assets": r.returned_assets,
                "return_rate_pct": r.return_rate_pct
            }))
            .collect::<Vec<_>>(),
        "materials_inventory" => reporting_service
            .materials_inventory()
            .await
            .map_err(|_| "failed to load materials inventory report".to_string())?
            .into_iter()
            .map(|r| serde_json::json!({
                "asset_id": r.asset_id,
                "booklet_code": r.booklet_code,
                "tracking_status": r.tracking_status,
                "session_id": r.session_id,
                "expires_on": r.expires_on,
                "incident_count": r.incident_count
            }))
            .collect::<Vec<_>>(),
        "operations_alerts" => reporting_service
            .operations_alerts(payload.within_days.unwrap_or(30))
            .await
            .map_err(|_| "failed to load operations alerts report".to_string())?
            .into_iter()
            .map(|r| serde_json::json!({
                "alert_type": r.alert_type,
                "severity": r.severity,
                "session_id": r.session_id,
                "asset_id": r.asset_id,
                "message": r.message
            }))
            .collect::<Vec<_>>(),
        _ => {
            return Err(
                "unsupported report; use seat_utilization, near_expiry_assets, incident_rates, return_rates, materials_inventory, or operations_alerts"
                    .to_string(),
            )
        }
    };

    if let Some(filter_text) = filter {
        rows.retain(|row| row.to_string().to_lowercase().contains(&filter_text));
    }

    rows.truncate(limit);

    let fields = match report_key.as_str() {
        "seat_utilization" => vec![
            "room_id".to_string(),
            "location".to_string(),
            "capacity".to_string(),
            "allocated".to_string(),
        ],
        "near_expiry_assets" => vec![
            "asset_id".to_string(),
            "booklet_code".to_string(),
            "expires_on".to_string(),
        ],
        "incident_rates" => vec!["session_id".to_string(), "avg_incidents".to_string()],
        "return_rates" => vec![
            "session_id".to_string(),
            "total_assets".to_string(),
            "returned_assets".to_string(),
            "return_rate_pct".to_string(),
        ],
        "materials_inventory" => vec![
            "asset_id".to_string(),
            "booklet_code".to_string(),
            "tracking_status".to_string(),
            "session_id".to_string(),
            "expires_on".to_string(),
            "incident_count".to_string(),
        ],
        "operations_alerts" => vec![
            "alert_type".to_string(),
            "severity".to_string(),
            "session_id".to_string(),
            "asset_id".to_string(),
            "message".to_string(),
        ],
        _ => unreachable!(),
    };

    Ok((rows, fields))
}
