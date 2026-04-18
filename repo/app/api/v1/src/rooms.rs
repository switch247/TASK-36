use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put, State};

use sqlx::{MySql, MySqlPool, QueryBuilder};

use app_services::audit_service::AuditService;
use app_services::cleansing_service::CleansingService;
use app_services::rbac_service::RbacService;

use crate::errors::{ApiError, ApiResult};
use crate::pagination::PaginationParams;
use crate::shared::{actor_user_id, audit, is_admin, ApiContext};
use crate::template_validation::validate_against_template;
use crate::validators::validate_room_capacity;

#[derive(Debug, serde::Deserialize)]
pub struct CreateRoomRequest {
    pub id: String,
    pub capacity: i32,
    pub location: String,
    pub template_id: Option<String>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct RoomRow {
    pub id: String,
    pub capacity: i32,
    pub location: String,
    pub created_by: String,
}

#[post("/rooms", data = "<payload>")]
pub async fn create_room(
    payload: Json<CreateRoomRequest>,
    pool: &State<MySqlPool>,
    _cleansing_service: &State<CleansingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot create room"))?;
    validate_room_capacity(payload.capacity)?;
    let user_id = actor_user_id(&ctx)?;
    let mut template_payload = std::collections::HashMap::new();
    template_payload.insert("id".to_string(), serde_json::Value::String(payload.id.clone()));
    template_payload.insert(
        "capacity".to_string(),
        serde_json::Value::Number(payload.capacity.into()),
    );
    template_payload.insert(
        "location".to_string(),
        serde_json::Value::String(payload.location.clone()),
    );
    let template_id = payload
        .template_id
        .as_deref()
        .filter(|x| !x.trim().is_empty())
        .unwrap_or("room-config");
    validate_against_template(pool.inner(), template_id, template_payload).await?;

    let existing_capacities: Vec<i32> = sqlx::query_scalar("SELECT capacity FROM rooms")
        .fetch_all(pool.inner())
        .await
        .unwrap_or_default();
    let capacity_outlier = if existing_capacities.is_empty() {
        false
    } else {
        CleansingService::is_room_capacity_outlier(payload.capacity, &existing_capacities)
            .unwrap_or(false)
    };
    let normalized_location = payload.location.trim();
    let (location_to_save, defaults_applied) = if normalized_location.is_empty() {
        ("Unspecified Room".to_string(), vec!["location".to_string()])
    } else {
        (normalized_location.to_string(), Vec::new())
    };

    sqlx::query("INSERT INTO rooms (id, capacity, location, created_by) VALUES (?, ?, ?, ?)")
        .bind(&payload.id)
        .bind(payload.capacity)
        .bind(&location_to_save)
        .bind(user_id)
        .execute(pool.inner())
        .await
        .map_err(|_| ApiError::conflict("room already exists"))?;

    let _ = audit_service
        .record_change(
            "rooms",
            &payload.id,
            "CREATE",
            None,
            Some(serde_json::json!({
                "capacity": payload.capacity,
                "location": location_to_save,
                "cleansing": {
                    "capacity_outlier": capacity_outlier,
                    "defaults_applied": defaults_applied,
                    "field_mapping_applied": true
                }
            })),
            user_id,
        )
        .await;

    audit(audit_service, &ctx, "create_room", "/api/v1/rooms").await;
    Ok(Status::Created)
}

#[get("/rooms?<page>&<limit>&<sort_by>&<sort_order>&<filter>")]
pub async fn list_rooms(
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
    page: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    filter: Option<String>,
) -> ApiResult<Json<Vec<RoomRow>>> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot list rooms"))?;
    let user_id = actor_user_id(&ctx)?;

    let params = PaginationParams { page, limit, sort_by, sort_order, filter };
    let order = params.sort_order_sql();
    let col = match params.sort_by.as_deref() {
        Some("id") => "id",
        Some("capacity") => "capacity",
        Some("location") => "location",
        _ => "created_at",
    };

    let filter_value = params
        .filter
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| format!("%{}%", v));

    let mut qb = QueryBuilder::<MySql>::new("SELECT id, capacity, location, created_by FROM rooms");

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
            .push(" OR location LIKE ")
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
        .build_query_as::<RoomRow>()
        .fetch_all(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to list rooms"))?;

    audit(audit_service, &ctx, "list_rooms", "/api/v1/rooms").await;
    Ok(Json(rows))
}

#[put("/rooms/<id>", data = "<payload>")]
pub async fn update_room(
    id: &str,
    payload: Json<CreateRoomRequest>,
    pool: &State<MySqlPool>,
    _cleansing_service: &State<CleansingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot update room"))?;
    validate_room_capacity(payload.capacity)?;
    let user_id = actor_user_id(&ctx)?;
    let mut template_payload = std::collections::HashMap::new();
    template_payload.insert("id".to_string(), serde_json::Value::String(id.to_string()));
    template_payload.insert(
        "capacity".to_string(),
        serde_json::Value::Number(payload.capacity.into()),
    );
    template_payload.insert(
        "location".to_string(),
        serde_json::Value::String(payload.location.clone()),
    );
    let template_id = payload
        .template_id
        .as_deref()
        .filter(|x| !x.trim().is_empty())
        .unwrap_or("room-config");
    validate_against_template(pool.inner(), template_id, template_payload).await?;

    let existing_capacities: Vec<i32> = sqlx::query_scalar("SELECT capacity FROM rooms WHERE id <> ?")
        .bind(id)
        .fetch_all(pool.inner())
        .await
        .unwrap_or_default();
    let capacity_outlier = if existing_capacities.is_empty() {
        false
    } else {
        CleansingService::is_room_capacity_outlier(payload.capacity, &existing_capacities)
            .unwrap_or(false)
    };
    let normalized_location = payload.location.trim();
    let (location_to_save, defaults_applied) = if normalized_location.is_empty() {
        ("Unspecified Room".to_string(), vec!["location".to_string()])
    } else {
        (normalized_location.to_string(), Vec::new())
    };

    let result = if is_admin(&ctx) {
        sqlx::query("UPDATE rooms SET capacity = ?, location = ? WHERE id = ?")
            .bind(payload.capacity)
            .bind(&location_to_save)
            .bind(id)
            .execute(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to update room"))?
    } else {
        sqlx::query("UPDATE rooms SET capacity = ?, location = ? WHERE id = ? AND created_by = ?")
            .bind(payload.capacity)
            .bind(&location_to_save)
            .bind(id)
            .bind(user_id)
            .execute(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to update room"))?
    };

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("room not found"));
    }

    let _ = audit_service
        .record_change(
            "rooms",
            id,
            "UPDATE",
            None,
            Some(serde_json::json!({
                "capacity": payload.capacity,
                "location": location_to_save,
                "cleansing": {
                    "capacity_outlier": capacity_outlier,
                    "defaults_applied": defaults_applied,
                    "field_mapping_applied": true
                }
            })),
            user_id,
        )
        .await;

    audit(audit_service, &ctx, "update_room", "/api/v1/rooms/{id}").await;
    Ok(Status::Ok)
}

#[delete("/rooms/<id>")]
pub async fn delete_room(
    id: &str,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot delete room"))?;
    let user_id = actor_user_id(&ctx)?;

    let result = if is_admin(&ctx) {
        sqlx::query("DELETE FROM rooms WHERE id = ?")
            .bind(id)
            .execute(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to delete room"))?
    } else {
        sqlx::query("DELETE FROM rooms WHERE id = ? AND created_by = ?")
            .bind(id)
            .bind(user_id)
            .execute(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to delete room"))?
    };

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("room not found"));
    }

    let _ = audit_service
        .record_change("rooms", id, "DELETE", None, None, user_id)
        .await;

    audit(audit_service, &ctx, "delete_room", "/api/v1/rooms/{id}").await;
    Ok(Status::NoContent)
}
