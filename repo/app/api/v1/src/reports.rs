use rocket::serde::json::Json;
use rocket::{get, State};

use sqlx::MySqlPool;

use app_models::entities::ReportSeatUtilization;
use app_services::audit_service::AuditService;
use app_services::rbac_service::RbacService;
use app_services::reporting_service::{
    ReportAlert, ReportIncidentRate, ReportMaterialInventory, ReportNearExpiryAsset,
    ReportReturnRate, ReportingService,
};

use crate::errors::{ApiError, ApiResult};
use crate::pagination::PaginationParams;
use crate::shared::{audit, ApiContext};

#[derive(Debug, serde::Serialize)]
pub struct DashboardSummary {
    pub seat_utilization_count: usize,
    pub near_expiry_count: usize,
    pub incident_rate_count: usize,
    pub return_rate_count: usize,
    pub material_inventory_count: usize,
    pub alert_count: usize,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct UpcomingSessionRow {
    pub id: String,
    pub template_name: String,
    pub status: String,
    pub starts_at: chrono::NaiveDateTime,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct RecentOutputRow {
    pub id: String,
    pub output_type: String,
    pub mode: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, serde::Serialize)]
pub struct DashboardSummaryV2 {
    pub total_candidates: i64,
    pub total_rooms: i64,
    pub total_sessions_this_week: i64,
    pub seat_utilization_count: usize,
    pub near_expiry_count: usize,
    pub incident_rate_count: usize,
    pub return_rate_count: usize,
    pub material_inventory_count: usize,
    pub alert_count: usize,
    pub upcoming_sessions: Vec<UpcomingSessionRow>,
    pub recent_outputs: Vec<RecentOutputRow>,
}

#[get("/reports/dashboard")]
pub async fn reports_dashboard(
    reporting_service: &State<ReportingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<DashboardSummary>> {
    RbacService::require_reporting(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot view dashboard"))?;

    let seat = reporting_service
        .seat_utilization()
        .await
        .unwrap_or_else(|err| {
            tracing::error!(error = %err, "reports_dashboard seat utilization query failed");
            Vec::new()
        });
    let expiry = reporting_service
        .near_expiry_assets(30)
        .await
        .unwrap_or_else(|err| {
            tracing::error!(error = %err, "reports_dashboard near expiry query failed");
            Vec::new()
        });
    let incidents = reporting_service
        .incident_rates()
        .await
        .unwrap_or_else(|err| {
            tracing::error!(error = %err, "reports_dashboard incident rates query failed");
            Vec::new()
        });
    let returns = reporting_service
        .return_rates()
        .await
        .unwrap_or_else(|err| {
            tracing::error!(error = %err, "reports_dashboard return rates query failed");
            Vec::new()
        });
    let materials = reporting_service
        .materials_inventory()
        .await
        .unwrap_or_else(|err| {
            tracing::error!(error = %err, "reports_dashboard materials inventory query failed");
            Vec::new()
        });
    let alerts = reporting_service
        .operations_alerts(30)
        .await
        .unwrap_or_else(|err| {
            tracing::error!(error = %err, "reports_dashboard alerts query failed");
            Vec::new()
        });

    audit(
        audit_service,
        &ctx,
        "reports_dashboard",
        "/api/v1/reports/dashboard",
    )
    .await;
    Ok(Json(DashboardSummary {
        seat_utilization_count: seat.len(),
        near_expiry_count: expiry.len(),
        incident_rate_count: incidents.len(),
        return_rate_count: returns.len(),
        material_inventory_count: materials.len(),
        alert_count: alerts.len(),
    }))
}

#[get("/dashboard/summary")]
pub async fn dashboard_summary(
    pool: &State<MySqlPool>,
    reporting_service: &State<ReportingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<DashboardSummaryV2>> {
    RbacService::require_reporting(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot view dashboard"))?;

    let total_candidates: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM candidates")
        .fetch_one(pool.inner())
        .await
        .map_err(|err| {
            tracing::error!(error = %err, "dashboard_summary total_candidates query failed");
            ApiError::internal("failed to load dashboard summary")
        })?;

    let total_rooms: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rooms")
        .fetch_one(pool.inner())
        .await
        .map_err(|err| {
            tracing::error!(error = %err, "dashboard_summary total_rooms query failed");
            ApiError::internal("failed to load dashboard summary")
        })?;

    let total_sessions_this_week: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM exam_sessions WHERE YEARWEEK(starts_at, 1) = YEARWEEK(UTC_TIMESTAMP(), 1)",
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|err| {
        tracing::error!(error = %err, "dashboard_summary total_sessions_this_week query failed");
        ApiError::internal("failed to load dashboard summary")
    })?;

    let upcoming_sessions = sqlx::query_as::<_, UpcomingSessionRow>(
        "SELECT id, template_name, status, starts_at FROM exam_sessions WHERE starts_at >= UTC_TIMESTAMP() ORDER BY starts_at ASC LIMIT 3",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|err| {
        tracing::error!(error = %err, "dashboard_summary upcoming_sessions query failed");
        ApiError::internal("failed to load dashboard summary")
    })?;

    let recent_outputs = sqlx::query_as::<_, RecentOutputRow>(
        "SELECT id, output_type, mode, created_at FROM print_outputs ORDER BY created_at DESC LIMIT 5",
    )
    .fetch_all(pool.inner())
    .await
    .unwrap_or_default();

    let seat = reporting_service
        .seat_utilization()
        .await
        .unwrap_or_default();
    let expiry = reporting_service
        .near_expiry_assets(30)
        .await
        .unwrap_or_default();
    let incidents = reporting_service.incident_rates().await.unwrap_or_default();
    let returns = reporting_service.return_rates().await.unwrap_or_default();
    let materials = reporting_service
        .materials_inventory()
        .await
        .unwrap_or_default();
    let alerts = reporting_service
        .operations_alerts(30)
        .await
        .unwrap_or_default();

    audit(
        audit_service,
        &ctx,
        "dashboard_summary",
        "/api/v1/dashboard/summary",
    )
    .await;
    Ok(Json(DashboardSummaryV2 {
        total_candidates,
        total_rooms,
        total_sessions_this_week,
        seat_utilization_count: seat.len(),
        near_expiry_count: expiry.len(),
        incident_rate_count: incidents.len(),
        return_rate_count: returns.len(),
        material_inventory_count: materials.len(),
        alert_count: alerts.len(),
        upcoming_sessions,
        recent_outputs,
    }))
}

#[get("/operations/seat-utilization?<page>&<limit>&<sort_by>&<sort_order>&<filter>")]
pub async fn seat_utilization(
    reporting_service: &State<ReportingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
    page: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    filter: Option<String>,
) -> ApiResult<Json<Vec<ReportSeatUtilization>>> {
    RbacService::require_reporting(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot view seat utilization"))?;

    let params = PaginationParams {
        page,
        limit,
        sort_by,
        sort_order,
        filter,
    };

    let mut rows = reporting_service.seat_utilization().await.map_err(|err| {
        tracing::error!(error = %err, "seat_utilization query failed");
        ApiError::internal("failed to load seat utilization")
    })?;

    if let Some(filter) = params.filter.as_deref().map(str::to_lowercase) {
        rows.retain(|r| {
            r.room_id.to_lowercase().contains(&filter)
                || r.location.to_lowercase().contains(&filter)
        });
    }

    let asc = params.sort_order_sql() == "ASC";
    match params.sort_by.as_deref() {
        Some("location") => rows.sort_by(|a, b| a.location.cmp(&b.location)),
        Some("capacity") => rows.sort_by_key(|r| r.capacity),
        Some("allocated") => rows.sort_by_key(|r| r.allocated),
        _ => rows.sort_by(|a, b| a.room_id.cmp(&b.room_id)),
    }
    if !asc {
        rows.reverse();
    }

    rows = rows
        .into_iter()
        .skip(params.offset() as usize)
        .take(params.limit() as usize)
        .collect();

    audit(
        audit_service,
        &ctx,
        "seat_utilization",
        "/api/v1/operations/seat-utilization",
    )
    .await;
    Ok(Json(rows))
}

#[get("/operations/near-expiry-alerts?<page>&<limit>&<sort_by>&<sort_order>&<filter>")]
pub async fn near_expiry_alerts(
    reporting_service: &State<ReportingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
    page: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    filter: Option<String>,
) -> ApiResult<Json<Vec<ReportNearExpiryAsset>>> {
    RbacService::require_reporting(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot view near expiry alerts"))?;

    let params = PaginationParams {
        page,
        limit,
        sort_by,
        sort_order,
        filter,
    };

    let mut rows = reporting_service
        .near_expiry_assets(30)
        .await
        .map_err(|err| {
            tracing::error!(error = %err, "near_expiry_alerts query failed");
            ApiError::internal("failed to load near expiry alerts")
        })?;

    if let Some(filter) = params.filter.as_deref().map(str::to_lowercase) {
        rows.retain(|r| {
            r.id.to_lowercase().contains(&filter) || r.booklet_code.to_lowercase().contains(&filter)
        });
    }

    let asc = params.sort_order_sql() == "ASC";
    match params.sort_by.as_deref() {
        Some("booklet_code") => rows.sort_by(|a, b| a.booklet_code.cmp(&b.booklet_code)),
        Some("expires_on") => rows.sort_by_key(|r| r.expires_on),
        _ => rows.sort_by(|a, b| a.id.cmp(&b.id)),
    }
    if !asc {
        rows.reverse();
    }

    rows = rows
        .into_iter()
        .skip(params.offset() as usize)
        .take(params.limit() as usize)
        .collect();

    audit(
        audit_service,
        &ctx,
        "near_expiry_alerts",
        "/api/v1/operations/near-expiry-alerts",
    )
    .await;
    Ok(Json(rows))
}

#[get("/operations/incident-rates?<page>&<limit>&<sort_by>&<sort_order>&<filter>")]
pub async fn incident_rates(
    reporting_service: &State<ReportingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
    page: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    filter: Option<String>,
) -> ApiResult<Json<Vec<ReportIncidentRate>>> {
    RbacService::require_reporting(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot view incident rates"))?;

    let params = PaginationParams {
        page,
        limit,
        sort_by,
        sort_order,
        filter,
    };

    let mut rows = reporting_service
        .incident_rates()
        .await
        .unwrap_or_else(|err| {
            tracing::error!(error = %err, "incident_rates query failed");
            Vec::new()
        });

    if let Some(filter) = params.filter.as_deref().map(str::to_lowercase) {
        rows.retain(|r| r.session_id.to_lowercase().contains(&filter));
    }

    let asc = params.sort_order_sql() == "ASC";
    match params.sort_by.as_deref() {
        Some("avg_incidents") => rows.sort_by(|a, b| a.avg_incidents.total_cmp(&b.avg_incidents)),
        _ => rows.sort_by(|a, b| a.session_id.cmp(&b.session_id)),
    }
    if !asc {
        rows.reverse();
    }

    rows = rows
        .into_iter()
        .skip(params.offset() as usize)
        .take(params.limit() as usize)
        .collect();

    audit(
        audit_service,
        &ctx,
        "incident_rates",
        "/api/v1/operations/incident-rates",
    )
    .await;
    Ok(Json(rows))
}

#[get("/operations/return-rates?<page>&<limit>&<sort_by>&<sort_order>&<filter>")]
pub async fn return_rates(
    reporting_service: &State<ReportingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
    page: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    filter: Option<String>,
) -> ApiResult<Json<Vec<ReportReturnRate>>> {
    RbacService::require_reporting(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot view return rates"))?;

    let params = PaginationParams {
        page,
        limit,
        sort_by,
        sort_order,
        filter,
    };

    let mut rows = reporting_service
        .return_rates()
        .await
        .unwrap_or_else(|err| {
            tracing::error!(error = %err, "return_rates query failed");
            Vec::new()
        });

    if let Some(filter) = params.filter.as_deref().map(str::to_lowercase) {
        rows.retain(|r| r.session_id.to_lowercase().contains(&filter));
    }

    let asc = params.sort_order_sql() == "ASC";
    match params.sort_by.as_deref() {
        Some("total_assets") => rows.sort_by_key(|r| r.total_assets),
        Some("returned_assets") => rows.sort_by_key(|r| r.returned_assets),
        Some("return_rate_pct") => {
            rows.sort_by(|a, b| a.return_rate_pct.total_cmp(&b.return_rate_pct))
        }
        _ => rows.sort_by(|a, b| a.session_id.cmp(&b.session_id)),
    }
    if !asc {
        rows.reverse();
    }

    rows = rows
        .into_iter()
        .skip(params.offset() as usize)
        .take(params.limit() as usize)
        .collect();

    audit(
        audit_service,
        &ctx,
        "return_rates",
        "/api/v1/operations/return-rates",
    )
    .await;
    Ok(Json(rows))
}

#[get("/operations/materials-inventory?<page>&<limit>&<sort_by>&<sort_order>&<filter>")]
pub async fn materials_inventory(
    reporting_service: &State<ReportingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
    page: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    filter: Option<String>,
) -> ApiResult<Json<Vec<ReportMaterialInventory>>> {
    RbacService::require_reporting(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot view materials inventory"))?;
    let params = PaginationParams {
        page,
        limit,
        sort_by,
        sort_order,
        filter,
    };

    let mut rows = reporting_service
        .materials_inventory()
        .await
        .unwrap_or_else(|err| {
            tracing::error!(error = %err, "materials_inventory query failed");
            Vec::new()
        });

    if let Some(filter) = params.filter.as_deref().map(str::to_lowercase) {
        rows.retain(|r| {
            r.asset_id.to_lowercase().contains(&filter)
                || r.booklet_code.to_lowercase().contains(&filter)
                || r.tracking_status.to_lowercase().contains(&filter)
                || r.session_id.to_lowercase().contains(&filter)
        });
    }

    let asc = params.sort_order_sql() == "ASC";
    match params.sort_by.as_deref() {
        Some("tracking_status") => rows.sort_by(|a, b| a.tracking_status.cmp(&b.tracking_status)),
        Some("incident_count") => rows.sort_by_key(|r| r.incident_count),
        Some("session_id") => rows.sort_by(|a, b| a.session_id.cmp(&b.session_id)),
        _ => rows.sort_by(|a, b| a.booklet_code.cmp(&b.booklet_code)),
    }
    if !asc {
        rows.reverse();
    }

    rows = rows
        .into_iter()
        .skip(params.offset() as usize)
        .take(params.limit() as usize)
        .collect();

    audit(
        audit_service,
        &ctx,
        "materials_inventory",
        "/api/v1/operations/materials-inventory",
    )
    .await;
    Ok(Json(rows))
}

#[get("/operations/alerts?<within_days>&<page>&<limit>")]
pub async fn operations_alerts(
    reporting_service: &State<ReportingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
    within_days: Option<i64>,
    page: Option<u32>,
    limit: Option<u32>,
) -> ApiResult<Json<Vec<ReportAlert>>> {
    RbacService::require_reporting(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot view operations alerts"))?;

    let mut rows = reporting_service
        .operations_alerts(within_days.unwrap_or(30))
        .await
        .unwrap_or_else(|err| {
            tracing::error!(error = %err, "operations_alerts query failed");
            Vec::new()
        });

    let params = PaginationParams {
        page,
        limit,
        sort_by: None,
        sort_order: None,
        filter: None,
    };
    rows = rows
        .into_iter()
        .skip(params.offset() as usize)
        .take(params.limit() as usize)
        .collect();

    audit(
        audit_service,
        &ctx,
        "operations_alerts",
        "/api/v1/operations/alerts",
    )
    .await;
    Ok(Json(rows))
}

#[get("/operations/incident-rates")]
pub async fn incident_rates_fallback(
    reporting_service: &State<ReportingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<Vec<ReportIncidentRate>>> {
    RbacService::require_reporting(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot view incident rates"))?;

    let rows = reporting_service
        .incident_rates()
        .await
        .unwrap_or_else(|err| {
            tracing::error!(error = %err, "incident_rates_fallback query failed");
            Vec::new()
        });

    audit(
        audit_service,
        &ctx,
        "incident_rates",
        "/api/v1/operations/incident-rates",
    )
    .await;
    Ok(Json(rows))
}
