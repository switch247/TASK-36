use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put, State};

use sqlx::MySqlPool;
use tracing::info;

use app_services::audit_service::AuditService;
use app_services::auth_service::AuthService;
use app_services::rbac_service::RbacService;

use crate::errors::{ApiError, ApiResult};
use crate::shared::{actor_user_id, audit, ApiContext};
use crate::template_validation::validate_against_template;

fn normalize_role(role: &str) -> Option<&'static str> {
    match role.trim().to_ascii_lowercase().as_str() {
        "admin" | "administrator" => Some("Admin"),
        "coordinator" | "exam coordinator" => Some("Coordinator"),
        "proctor" => Some("Proctor"),
        "auditor" => Some("Auditor"),
        _ => None,
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: String,
    pub template_id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateUserRequest {
    pub password: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct UserRow {
    pub id: String,
    pub username: String,
    pub role: String,
    pub failed_login_attempts: i32,
    pub lockout_until: Option<chrono::NaiveDateTime>,
}

#[post("/users", data = "<payload>")]
pub async fn create_user(
    payload: Json<CreateUserRequest>,
    pool: &State<MySqlPool>,
    auth_service: &State<AuthService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_users(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot manage users"))?;
    let actor_id = actor_user_id(&ctx)?;
    let normalized_role =
        normalize_role(&payload.role).ok_or_else(|| ApiError::bad_request("invalid role"))?;
    if normalized_role == "Proctor" {
        let mut template_payload = std::collections::HashMap::new();
        template_payload.insert(
            "username".to_string(),
            serde_json::Value::String(payload.username.clone()),
        );
        template_payload.insert(
            "role".to_string(),
            serde_json::Value::String(normalized_role.to_string()),
        );
        let template_id = payload
            .template_id
            .as_deref()
            .filter(|x| !x.trim().is_empty())
            .unwrap_or("proctor-profile");
        validate_against_template(pool.inner(), template_id, template_payload).await?;
    }

    let hashed = auth_service
        .hash_password(&payload.password)
        .map_err(|_| ApiError::bad_request("password does not meet policy"))?;

    let new_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, role, failed_login_attempts) VALUES (?, ?, ?, ?, 0)",
    )
    .bind(&new_id)
    .bind(&payload.username)
    .bind(hashed)
    .bind(normalized_role)
    .execute(pool.inner())
    .await
    .map_err(|_| ApiError::conflict("username already exists"))?;

    let _ = audit_service
        .record_change(
            "users",
            &new_id,
            "CREATE",
            None,
            Some(serde_json::json!({"role": normalized_role})),
            actor_id,
        )
        .await;

    info!(action = "create_user", user_id = %new_id, "user created");
    audit(audit_service, &ctx, "create_user", "/api/v1/users").await;
    Ok(Status::Created)
}

#[get("/users")]
pub async fn list_users(
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<Vec<UserRow>>> {
    RbacService::require_manage_users(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot view users"))?;

    let rows = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, role, failed_login_attempts, lockout_until FROM users",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|_| ApiError::internal("failed to list users"))?;

    audit(audit_service, &ctx, "list_users", "/api/v1/users").await;
    Ok(Json(rows))
}

#[put("/users/<id>", data = "<payload>")]
pub async fn update_user(
    id: &str,
    payload: Json<UpdateUserRequest>,
    pool: &State<MySqlPool>,
    auth_service: &State<AuthService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_users(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot update users"))?;
    let actor_id = actor_user_id(&ctx)?;

    let mut touched = false;

    if let Some(role) = &payload.role {
        let normalized_role =
            normalize_role(role).ok_or_else(|| ApiError::bad_request("invalid role"))?;
        if normalized_role == "Proctor" {
            let username: Option<String> =
                sqlx::query_scalar("SELECT username FROM users WHERE id = ?")
                    .bind(id)
                    .fetch_optional(pool.inner())
                    .await
                    .map_err(|_| {
                        ApiError::internal("failed to read user for template validation")
                    })?;
            let Some(username) = username else {
                return Err(ApiError::not_found("user not found"));
            };
            let mut template_payload = std::collections::HashMap::new();
            template_payload.insert("username".to_string(), serde_json::Value::String(username));
            template_payload.insert(
                "role".to_string(),
                serde_json::Value::String(normalized_role.to_string()),
            );
            validate_against_template(pool.inner(), "proctor-profile", template_payload).await?;
        }
        let result = sqlx::query("UPDATE users SET role = ? WHERE id = ?")
            .bind(normalized_role)
            .bind(id)
            .execute(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to update user role"))?;
        touched = touched || result.rows_affected() > 0;
    }

    if let Some(password) = &payload.password {
        let hashed = auth_service
            .hash_password(password)
            .map_err(|_| ApiError::bad_request("password does not meet policy"))?;

        let result = sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
            .bind(hashed)
            .bind(id)
            .execute(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to update password"))?;
        touched = touched || result.rows_affected() > 0;
    }

    if !touched {
        return Err(ApiError::not_found("user not found"));
    }

    let _ = audit_service
        .record_change(
            "users",
            id,
            "UPDATE",
            None,
            Some(serde_json::json!({"role": payload.role})),
            actor_id,
        )
        .await;

    audit(audit_service, &ctx, "update_user", "/api/v1/users/{id}").await;
    Ok(Status::Ok)
}

#[delete("/users/<id>")]
pub async fn delete_user(
    id: &str,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_users(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot delete users"))?;
    let actor_id = actor_user_id(&ctx)?;

    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to delete user"))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("user not found"));
    }

    let _ = audit_service
        .record_change("users", id, "DELETE", None, None, actor_id)
        .await;

    audit(audit_service, &ctx, "delete_user", "/api/v1/users/{id}").await;
    Ok(Status::NoContent)
}
