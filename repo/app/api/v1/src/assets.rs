use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put, State};

use chrono::NaiveDate;
use sqlx::{MySql, MySqlPool, QueryBuilder};

use app_services::audit_service::AuditService;
use app_services::rbac_service::RbacService;

use crate::errors::{ApiError, ApiResult};
use crate::pagination::PaginationParams;
use crate::shared::{actor_user_id, audit, is_admin, ApiContext};

#[derive(Debug, serde::Deserialize)]
pub struct CreateAssetRequest {
    pub id: String,
    pub booklet_code: String,
    pub tracking_status: String,
    pub session_id: String,
    pub expires_on: Option<String>,
    pub incident_count: i32,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct AssetRow {
    pub id: String,
    pub booklet_code: String,
    pub tracking_status: String,
    pub session_id: String,
    pub expires_on: Option<NaiveDate>,
    pub incident_count: i32,
    pub created_by: String,
}

#[post("/assets", data = "<payload>")]
pub async fn create_asset(
    payload: Json<CreateAssetRequest>,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot create asset"))?;
    let user_id = actor_user_id(&ctx)?;

    let expires_on = match &payload.expires_on {
        Some(v) => Some(
            NaiveDate::parse_from_str(v, "%Y-%m-%d")
                .map_err(|_| ApiError::bad_request("expires_on must be YYYY-MM-DD"))?,
        ),
        None => None,
    };

    sqlx::query(
        "INSERT INTO assets (id, booklet_code, tracking_status, session_id, expires_on, incident_count, created_by) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&payload.id)
    .bind(&payload.booklet_code)
    .bind(&payload.tracking_status)
    .bind(&payload.session_id)
    .bind(expires_on)
    .bind(payload.incident_count)
    .bind(user_id)
    .execute(pool.inner())
    .await
    .map_err(|_| ApiError::conflict("asset already exists"))?;

    let _ = audit_service
        .record_change(
            "assets",
            &payload.id,
            "CREATE",
            None,
            Some(serde_json::json!({
                "booklet_code": payload.booklet_code,
                "tracking_status": payload.tracking_status,
                "session_id": payload.session_id,
                "incident_count": payload.incident_count
            })),
            user_id,
        )
        .await;

    audit(audit_service, &ctx, "create_asset", "/api/v1/assets").await;
    Ok(Status::Created)
}

#[get("/assets?<page>&<limit>&<sort_by>&<sort_order>&<filter>")]
#[allow(clippy::too_many_arguments)] // Rocket binds each query param as a separate argument.
pub async fn list_assets(
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
    page: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    filter: Option<String>,
) -> ApiResult<Json<Vec<AssetRow>>> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot list assets"))?;
    let user_id = actor_user_id(&ctx)?;

    let params = PaginationParams {
        page,
        limit,
        sort_by,
        sort_order,
        filter,
    };
    let order = params.sort_order_sql();
    let col = match params.sort_by.as_deref() {
        Some("id") => "id",
        Some("booklet_code") => "booklet_code",
        Some("tracking_status") => "tracking_status",
        Some("expires_on") => "expires_on",
        _ => "created_at",
    };

    let filter_value = params
        .filter
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| format!("%{}%", v));

    let mut qb = QueryBuilder::<MySql>::new(
        "SELECT id, booklet_code, tracking_status, session_id, expires_on, incident_count, created_by FROM assets",
    );

    let mut has_where = false;
    if !is_admin(&ctx) {
        qb.push(" WHERE created_by = ").push_bind(user_id);
        has_where = true;
    }

    if let Some(ref like) = filter_value {
        if has_where {
            qb.push(" AND (");
        } else {
            qb.push(" WHERE (");
        }
        qb.push("id LIKE ")
            .push_bind(like.clone())
            .push(" OR booklet_code LIKE ")
            .push_bind(like.clone())
            .push(" OR tracking_status LIKE ")
            .push_bind(like.clone())
            .push(")");
    }

    qb.push(" ORDER BY ")
        .push(col)
        .push(" ")
        .push(order)
        .push(" LIMIT ")
        .push_bind(params.limit() as i64)
        .push(" OFFSET ")
        .push_bind(params.offset() as i64);

    let rows = qb
        .build_query_as::<AssetRow>()
        .fetch_all(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to list assets"))?;

    audit(audit_service, &ctx, "list_assets", "/api/v1/assets").await;
    Ok(Json(rows))
}

#[put("/assets/<id>", data = "<payload>")]
pub async fn update_asset(
    id: &str,
    payload: Json<CreateAssetRequest>,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot update asset"))?;
    let user_id = actor_user_id(&ctx)?;

    let expires_on = match &payload.expires_on {
        Some(v) => Some(
            NaiveDate::parse_from_str(v, "%Y-%m-%d")
                .map_err(|_| ApiError::bad_request("expires_on must be YYYY-MM-DD"))?,
        ),
        None => None,
    };

    let result = if is_admin(&ctx) {
        sqlx::query(
            "UPDATE assets SET booklet_code = ?, tracking_status = ?, session_id = ?, expires_on = ?, incident_count = ? WHERE id = ?",
        )
        .bind(&payload.booklet_code)
        .bind(&payload.tracking_status)
        .bind(&payload.session_id)
        .bind(expires_on)
        .bind(payload.incident_count)
        .bind(id)
        .execute(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to update asset"))?
    } else {
        sqlx::query(
            "UPDATE assets SET booklet_code = ?, tracking_status = ?, session_id = ?, expires_on = ?, incident_count = ? WHERE id = ? AND created_by = ?",
        )
        .bind(&payload.booklet_code)
        .bind(&payload.tracking_status)
        .bind(&payload.session_id)
        .bind(expires_on)
        .bind(payload.incident_count)
        .bind(id)
        .bind(user_id)
        .execute(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to update asset"))?
    };

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("asset not found"));
    }

    let _ = audit_service
        .record_change(
            "assets",
            id,
            "UPDATE",
            None,
            Some(serde_json::json!({
                "booklet_code": payload.booklet_code,
                "tracking_status": payload.tracking_status,
                "session_id": payload.session_id,
                "incident_count": payload.incident_count
            })),
            user_id,
        )
        .await;

    audit(audit_service, &ctx, "update_asset", "/api/v1/assets/{id}").await;
    Ok(Status::Ok)
}

#[delete("/assets/<id>")]
pub async fn delete_asset(
    id: &str,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot delete asset"))?;
    let user_id = actor_user_id(&ctx)?;

    let result = if is_admin(&ctx) {
        sqlx::query("DELETE FROM assets WHERE id = ?")
            .bind(id)
            .execute(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to delete asset"))?
    } else {
        sqlx::query("DELETE FROM assets WHERE id = ? AND created_by = ?")
            .bind(id)
            .bind(user_id)
            .execute(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to delete asset"))?
    };

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("asset not found"));
    }

    let _ = audit_service
        .record_change("assets", id, "DELETE", None, None, user_id)
        .await;

    audit(audit_service, &ctx, "delete_asset", "/api/v1/assets/{id}").await;
    Ok(Status::NoContent)
}
