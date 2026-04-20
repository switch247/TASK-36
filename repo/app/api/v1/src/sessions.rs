use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put, State};

use sqlx::{MySql, MySqlPool, QueryBuilder};

use app_core::types::UserRole;
use app_services::audit_service::AuditService;
use app_services::cleansing_service::CleansingService;
use app_services::rbac_service::RbacService;

use crate::errors::{ApiError, ApiResult};
use crate::pagination::PaginationParams;
use crate::shared::{actor_user_id, audit, is_admin, parse_prompt_datetime, ApiContext};
use crate::template_validation::validate_against_template;
use crate::validators::validate_session_duration;

#[derive(Debug, serde::Deserialize)]
pub struct CreateSessionRequest {
    pub id: String,
    pub template_name: String,
    pub duration_minutes: i32,
    pub status: String,
    pub starts_at: String,
    pub ends_at: String,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct SessionRow {
    pub id: String,
    pub template_name: String,
    pub duration_minutes: i32,
    pub status: String,
    pub starts_at: chrono::NaiveDateTime,
    pub ends_at: chrono::NaiveDateTime,
    pub locked_for_final_print: bool,
    pub created_by: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct AssignSessionRequest {
    pub user_id: String,
}

fn normalize_session_status(status: &str) -> String {
    match status.trim().to_ascii_lowercase().as_str() {
        "draft" => "Draft".to_string(),
        "scheduled" | "" => "Scheduled".to_string(),
        "active" => "Active".to_string(),
        "completed" => "Completed".to_string(),
        "cancelled" | "canceled" => "Cancelled".to_string(),
        "finalprinted" | "final_printed" | "final printed" => "FinalPrinted".to_string(),
        _ => status.trim().to_string(),
    }
}

#[post("/sessions", data = "<payload>")]
pub async fn create_session(
    payload: Json<CreateSessionRequest>,
    pool: &State<MySqlPool>,
    _cleansing_service: &State<CleansingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot create session"))?;
    validate_session_duration(payload.duration_minutes)?;
    let user_id = actor_user_id(&ctx)?;

    let starts_at = parse_prompt_datetime(&payload.starts_at)?;
    let ends_at = parse_prompt_datetime(&payload.ends_at)?;
    let normalized_status = normalize_session_status(&payload.status);
    let existing_durations: Vec<i32> =
        sqlx::query_scalar("SELECT duration_minutes FROM exam_sessions")
            .fetch_all(pool.inner())
            .await
            .unwrap_or_default();
    let duration_outlier = if existing_durations.is_empty() {
        false
    } else {
        let avg = existing_durations.iter().map(|d| *d as f64).sum::<f64>()
            / existing_durations.len() as f64;
        (payload.duration_minutes as f64) > (avg * 2.5)
    };
    let mut template_payload = std::collections::HashMap::new();
    template_payload.insert(
        "id".to_string(),
        serde_json::Value::String(payload.id.clone()),
    );
    template_payload.insert(
        "duration_minutes".to_string(),
        serde_json::Value::Number(payload.duration_minutes.into()),
    );
    template_payload.insert(
        "status".to_string(),
        serde_json::Value::String(normalized_status.clone()),
    );
    template_payload.insert(
        "starts_at".to_string(),
        serde_json::Value::String(payload.starts_at.clone()),
    );
    template_payload.insert(
        "ends_at".to_string(),
        serde_json::Value::String(payload.ends_at.clone()),
    );
    validate_against_template(pool.inner(), &payload.template_name, template_payload).await?;

    sqlx::query(
        "INSERT INTO exam_sessions (id, template_name, duration_minutes, status, starts_at, ends_at, created_by) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&payload.id)
    .bind(&payload.template_name)
    .bind(payload.duration_minutes)
    .bind(&normalized_status)
    .bind(starts_at)
    .bind(ends_at)
    .bind(user_id)
    .execute(pool.inner())
    .await
    .map_err(|_| ApiError::conflict("session already exists"))?;

    let _ = audit_service
        .record_change(
            "exam_sessions",
            &payload.id,
            "CREATE",
            None,
            Some(serde_json::json!({
                "template_name": payload.template_name,
                "status": normalized_status,
                "duration_minutes": payload.duration_minutes,
                "cleansing": {
                    "duration_outlier": duration_outlier,
                    "field_mapping_applied": true,
                    "defaults_applied": if payload.status.trim().is_empty() { vec!["status".to_string()] } else { Vec::<String>::new() }
                }
            })),
            user_id,
        )
        .await;

    audit(audit_service, &ctx, "create_session", "/api/v1/sessions").await;
    Ok(Status::Created)
}

#[get("/sessions?<page>&<limit>&<sort_by>&<sort_order>&<filter>")]
#[allow(clippy::too_many_arguments)] // Rocket binds each query param as a separate argument.
pub async fn list_sessions(
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
    page: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    filter: Option<String>,
) -> ApiResult<Json<Vec<SessionRow>>> {
    if RbacService::require_manage_inventory(&ctx.actor.role).is_err()
        && RbacService::require_print(&ctx.actor.role).is_err()
    {
        return Err(ApiError::forbidden("role cannot list sessions"));
    }
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
        Some("status") => "status",
        Some("template_name") => "template_name",
        Some("starts_at") => "starts_at",
        _ => "created_at",
    };

    let filter_value = params
        .filter
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| format!("%{}%", v));

    let mut qb = QueryBuilder::<MySql>::new(
        "SELECT id, template_name, duration_minutes, status, starts_at, ends_at, locked_for_final_print, created_by FROM exam_sessions",
    );

    let mut has_where = false;
    if !is_admin(&ctx) {
        if matches!(ctx.actor.role, UserRole::Proctor) {
            qb.push(
                " WHERE (created_by = ",
            )
            .push_bind(user_id)
            .push(
                " OR EXISTS (SELECT 1 FROM exam_session_assignments esa WHERE esa.session_id = exam_sessions.id AND esa.user_id = ",
            )
            .push_bind(user_id)
            .push("))");
            has_where = true;
        } else {
            qb.push(" WHERE created_by = ").push_bind(user_id);
            has_where = true;
        }
    }

    if let Some(ref like) = filter_value {
        if has_where {
            qb.push(" AND (");
        } else {
            qb.push(" WHERE (");
        }
        qb.push("id LIKE ")
            .push_bind(like.clone())
            .push(" OR status LIKE ")
            .push_bind(like.clone())
            .push(" OR template_name LIKE ")
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
        .build_query_as::<SessionRow>()
        .fetch_all(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to list sessions"))?;

    audit(audit_service, &ctx, "list_sessions", "/api/v1/sessions").await;
    Ok(Json(rows))
}

#[post("/sessions/<id>/assignments", data = "<payload>")]
pub async fn assign_session(
    id: &str,
    payload: Json<AssignSessionRequest>,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot assign sessions"))?;
    let user_id = actor_user_id(&ctx)?;

    let session_exists = if is_admin(&ctx) {
        sqlx::query_scalar::<_, i64>("SELECT 1 FROM exam_sessions WHERE id = ? LIMIT 1")
            .bind(id)
            .fetch_optional(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to verify session"))?
            .is_some()
    } else {
        sqlx::query_scalar::<_, i64>(
            "SELECT 1 FROM exam_sessions WHERE id = ? AND created_by = ? LIMIT 1",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to verify session"))?
        .is_some()
    };
    if !session_exists {
        return Err(ApiError::not_found("session not found"));
    }

    let is_proctor = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM users WHERE id = ? AND role = 'Proctor' LIMIT 1",
    )
    .bind(payload.user_id.trim())
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| ApiError::internal("failed to verify assignee"))?
    .is_some();
    if !is_proctor {
        return Err(ApiError::bad_request("assignee must be a Proctor"));
    }

    sqlx::query(
        "INSERT IGNORE INTO exam_session_assignments (id, session_id, user_id, assigned_by) VALUES (?, ?, ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(id)
    .bind(payload.user_id.trim())
    .bind(user_id)
    .execute(pool.inner())
    .await
    .map_err(|_| ApiError::internal("failed to assign session"))?;

    let _ = audit_service
        .record_change(
            "exam_session_assignments",
            id,
            "CREATE",
            None,
            Some(serde_json::json!({
                "session_id": id,
                "user_id": payload.user_id.trim(),
                "assigned_by": user_id
            })),
            user_id,
        )
        .await;

    audit(
        audit_service,
        &ctx,
        "assign_session",
        "/api/v1/sessions/{id}/assignments",
    )
    .await;
    Ok(Status::Created)
}

#[put("/sessions/<id>", data = "<payload>")]
pub async fn update_session(
    id: &str,
    payload: Json<CreateSessionRequest>,
    pool: &State<MySqlPool>,
    _cleansing_service: &State<CleansingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot update session"))?;
    validate_session_duration(payload.duration_minutes)?;
    let user_id = actor_user_id(&ctx)?;

    let starts_at = parse_prompt_datetime(&payload.starts_at)?;
    let ends_at = parse_prompt_datetime(&payload.ends_at)?;
    let normalized_status = normalize_session_status(&payload.status);
    let existing_durations: Vec<i32> =
        sqlx::query_scalar("SELECT duration_minutes FROM exam_sessions WHERE id <> ?")
            .bind(id)
            .fetch_all(pool.inner())
            .await
            .unwrap_or_default();
    let duration_outlier = if existing_durations.is_empty() {
        false
    } else {
        let avg = existing_durations.iter().map(|d| *d as f64).sum::<f64>()
            / existing_durations.len() as f64;
        (payload.duration_minutes as f64) > (avg * 2.5)
    };
    let mut template_payload = std::collections::HashMap::new();
    template_payload.insert("id".to_string(), serde_json::Value::String(id.to_string()));
    template_payload.insert(
        "duration_minutes".to_string(),
        serde_json::Value::Number(payload.duration_minutes.into()),
    );
    template_payload.insert(
        "status".to_string(),
        serde_json::Value::String(normalized_status.clone()),
    );
    template_payload.insert(
        "starts_at".to_string(),
        serde_json::Value::String(payload.starts_at.clone()),
    );
    template_payload.insert(
        "ends_at".to_string(),
        serde_json::Value::String(payload.ends_at.clone()),
    );
    validate_against_template(pool.inner(), &payload.template_name, template_payload).await?;

    let result = if is_admin(&ctx) {
        sqlx::query(
            "UPDATE exam_sessions SET template_name = ?, duration_minutes = ?, status = ?, starts_at = ?, ends_at = ? WHERE id = ?",
        )
        .bind(&payload.template_name)
        .bind(payload.duration_minutes)
        .bind(&normalized_status)
        .bind(starts_at)
        .bind(ends_at)
        .bind(id)
        .execute(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to update session"))?
    } else {
        sqlx::query(
            "UPDATE exam_sessions SET template_name = ?, duration_minutes = ?, status = ?, starts_at = ?, ends_at = ? WHERE id = ? AND created_by = ?",
        )
        .bind(&payload.template_name)
        .bind(payload.duration_minutes)
        .bind(&normalized_status)
        .bind(starts_at)
        .bind(ends_at)
        .bind(id)
        .bind(user_id)
        .execute(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to update session"))?
    };

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("session not found"));
    }

    let _ = audit_service
        .record_change(
            "exam_sessions",
            id,
            "UPDATE",
            None,
            Some(serde_json::json!({
                "template_name": payload.template_name,
                "status": normalized_status,
                "duration_minutes": payload.duration_minutes,
                "cleansing": {
                    "duration_outlier": duration_outlier,
                    "field_mapping_applied": true,
                    "defaults_applied": if payload.status.trim().is_empty() { vec!["status".to_string()] } else { Vec::<String>::new() }
                }
            })),
            user_id,
        )
        .await;

    audit(
        audit_service,
        &ctx,
        "update_session",
        "/api/v1/sessions/{id}",
    )
    .await;
    Ok(Status::Ok)
}

#[delete("/sessions/<id>")]
pub async fn delete_session(
    id: &str,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot delete session"))?;
    let user_id = actor_user_id(&ctx)?;

    let result = if is_admin(&ctx) {
        sqlx::query("DELETE FROM exam_sessions WHERE id = ?")
            .bind(id)
            .execute(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to delete session"))?
    } else {
        sqlx::query("DELETE FROM exam_sessions WHERE id = ? AND created_by = ?")
            .bind(id)
            .bind(user_id)
            .execute(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to delete session"))?
    };

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("session not found"));
    }

    let _ = audit_service
        .record_change("exam_sessions", id, "DELETE", None, None, user_id)
        .await;

    audit(
        audit_service,
        &ctx,
        "delete_session",
        "/api/v1/sessions/{id}",
    )
    .await;
    Ok(Status::NoContent)
}
