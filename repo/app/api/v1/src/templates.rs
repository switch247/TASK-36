use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put, State};
use sqlx::MySqlPool;

use app_services::audit_service::AuditService;
use app_services::rbac_service::RbacService;

use crate::errors::{ApiError, ApiResult};
use crate::shared::{actor_user_id, audit, ApiContext};

#[derive(Debug, serde::Deserialize)]
pub struct CreateTemplateRequest {
    pub template_id: String,
    pub version_no: i32,
    pub snapshot: serde_json::Value,
    pub lock_for_final_print: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateTemplateRequest {
    pub snapshot: serde_json::Value,
    pub lock_for_final_print: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct LockTemplateRequest {
    pub version_no: i32,
    pub snapshot: serde_json::Value,
    pub lock_for_final_print: bool,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct TemplateVersionRow {
    pub id: String,
    pub template_id: String,
    pub version_no: i32,
    pub snapshot: serde_json::Value,
    pub locked_for_final_print: bool,
    pub created_by: String,
    pub created_at: chrono::NaiveDateTime,
}

async fn ensure_template_version_mutable(
    pool: &MySqlPool,
    template_id: &str,
    version_no: i32,
) -> ApiResult<()> {
    let locked: Option<bool> = sqlx::query_scalar(
        "SELECT locked_for_final_print FROM template_versions WHERE template_id = ? AND version_no = ?",
    )
    .bind(template_id)
    .bind(version_no)
    .fetch_optional(pool)
    .await
    .map_err(|_| ApiError::internal("failed to check template lock state"))?;

    let Some(is_locked) = locked else {
        return Err(ApiError::not_found("template version not found"));
    };
    if is_locked {
        return Err(ApiError::conflict(
            "template version is locked for final print and cannot be modified",
        ));
    }

    let final_print_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM print_outputs WHERE template_id = ? AND template_version_no = ? AND mode = 'FinalPrint'",
    )
    .bind(template_id)
    .bind(version_no)
    .fetch_one(pool)
    .await
    .map_err(|_| ApiError::internal("failed to check template print references"))?;

    if final_print_count > 0 {
        return Err(ApiError::conflict(
            "template version is referenced by final prints and cannot be modified",
        ));
    }

    Ok(())
}

#[post("/templates", data = "<payload>")]
pub async fn create_template(
    payload: Json<CreateTemplateRequest>,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("Only coordinators and admins can manage templates"))?;
    let user_id = actor_user_id(&ctx)?;

    sqlx::query("INSERT INTO template_versions (id, template_id, version_no, snapshot, locked_for_final_print, created_by) VALUES (?, ?, ?, CAST(? AS JSON), ?, ?)")
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&payload.template_id)
        .bind(payload.version_no)
        .bind(payload.snapshot.to_string())
        .bind(payload.lock_for_final_print)
        .bind(user_id)
        .execute(pool.inner())
        .await
        .map_err(|_| ApiError::conflict("template version already exists"))?;

    let _ = audit_service
        .record_change(
            "template_versions",
            &payload.template_id,
            "CREATE",
            None,
            Some(payload.snapshot.clone()),
            user_id,
        )
        .await;

    audit(audit_service, &ctx, "create_template", "/api/v1/templates").await;
    Ok(Status::Created)
}

#[get("/templates")]
pub async fn list_templates(
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<Vec<TemplateVersionRow>>> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("Only coordinators and admins can view templates"))?;

    let rows = sqlx::query_as::<_, TemplateVersionRow>(
        "SELECT id, template_id, version_no, snapshot, locked_for_final_print, created_by, created_at FROM template_versions ORDER BY created_at DESC",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|_| ApiError::internal("failed to list templates"))?;

    audit(audit_service, &ctx, "list_templates", "/api/v1/templates").await;
    Ok(Json(rows))
}

#[put("/templates/<template_id>/<version_no>", data = "<payload>")]
pub async fn update_template(
    template_id: &str,
    version_no: i32,
    payload: Json<UpdateTemplateRequest>,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("Only coordinators and admins can update templates"))?;
    let user_id = actor_user_id(&ctx)?;
    ensure_template_version_mutable(pool.inner(), template_id, version_no).await?;

    let result = sqlx::query(
        "UPDATE template_versions SET snapshot = CAST(? AS JSON), locked_for_final_print = ? WHERE template_id = ? AND version_no = ?",
    )
    .bind(payload.snapshot.to_string())
    .bind(payload.lock_for_final_print)
    .bind(template_id)
    .bind(version_no)
    .execute(pool.inner())
    .await
    .map_err(|_| ApiError::internal("failed to update template"))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("template version not found"));
    }

    let _ = audit_service
        .record_change(
            "template_versions",
            template_id,
            "UPDATE",
            None,
            Some(payload.snapshot.clone()),
            user_id,
        )
        .await;

    audit(
        audit_service,
        &ctx,
        "update_template",
        "/api/v1/templates/{template_id}/{version_no}",
    )
    .await;
    Ok(Status::Ok)
}

#[delete("/templates/<template_id>/<version_no>")]
pub async fn delete_template(
    template_id: &str,
    version_no: i32,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("Only coordinators and admins can delete templates"))?;
    let user_id = actor_user_id(&ctx)?;
    ensure_template_version_mutable(pool.inner(), template_id, version_no).await?;

    let result =
        sqlx::query("DELETE FROM template_versions WHERE template_id = ? AND version_no = ?")
            .bind(template_id)
            .bind(version_no)
            .execute(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to delete template version"))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("template version not found"));
    }

    let _ = audit_service
        .record_change(
            "template_versions",
            template_id,
            "DELETE",
            None,
            None,
            user_id,
        )
        .await;

    audit(
        audit_service,
        &ctx,
        "delete_template",
        "/api/v1/templates/{template_id}/{version_no}",
    )
    .await;
    Ok(Status::NoContent)
}

#[post("/templates/<template_id>/lock", data = "<payload>")]
pub async fn lock_template(
    template_id: &str,
    payload: Json<LockTemplateRequest>,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    create_template(
        Json(CreateTemplateRequest {
            template_id: template_id.to_string(),
            version_no: payload.version_no,
            snapshot: payload.snapshot.clone(),
            lock_for_final_print: payload.lock_for_final_print,
        }),
        pool,
        audit_service,
        ctx,
    )
    .await
}
