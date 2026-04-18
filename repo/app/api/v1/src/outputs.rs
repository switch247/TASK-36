use base64::Engine;
use chrono::Utc;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, State};

use sqlx::MySqlPool;

use app_core::file_policy::{
    validate_extension, validate_file_count, validate_file_size, CaptureMetadata,
};
use app_services::audit_service::AuditService;
use app_services::messaging_service::MessagingService;
use app_services::output_service::OutputService;
use app_services::rbac_service::RbacService;

use crate::errors::{ApiError, ApiResult};
use crate::pagination::PaginationParams;
use crate::shared::{actor_user_id, audit, is_admin, ApiContext};

#[derive(Debug, serde::Deserialize)]
pub struct GenerateOutputRequest {
    pub session_id: String,
    pub mode: String,
    pub output_type: String,
}

#[derive(Debug, serde::Serialize)]
pub struct GenerateOutputResponse {
    pub output_type: String,
    pub mode: String,
    pub watermark: Option<String>,
    pub content: String,
    pub template_id: String,
    pub template_version_no: i32,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct OutputRow {
    pub id: String,
    pub session_id: String,
    pub output_type: String,
    pub mode: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, serde::Deserialize)]
pub struct MessageDraftRequest {
    pub channel: String,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct AttachmentUploadRequest {
    pub record_type: String,
    pub record_id: String,
    pub file_name: String,
    pub extension: String,
    pub bytes_base64: String,
    pub operator_label: String,
    pub device_label: String,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct AttachmentRow {
    pub id: String,
    pub record_type: String,
    pub record_id: String,
    pub file_name: String,
    pub extension: String,
    pub size_bytes: i64,
    pub fingerprint_sha256: String,
    pub operator_label: String,
    pub device_label: String,
    pub captured_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, serde::Serialize)]
pub struct AttachmentFileResponse {
    pub id: String,
    pub file_name: String,
    pub extension: String,
    pub bytes_base64: String,
}

#[post("/outputs", data = "<payload>")]
pub async fn generate_output(
    payload: Json<GenerateOutputRequest>,
    output_service: &State<OutputService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<GenerateOutputResponse>> {
    RbacService::require_print(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot generate output"))?;
    let user_id = actor_user_id(&ctx)?;

    let output = output_service
        .generate_print_output(
            &payload.session_id,
            &payload.output_type,
            &payload.mode,
            user_id,
            &ctx.actor.role,
            is_admin(&ctx),
        )
        .await
        .map_err(|err| {
            let msg = err.to_string();
            if msg.contains("forbidden:") {
                ApiError::forbidden("role cannot final-print")
            } else {
                ApiError::internal("failed to generate output")
            }
        })?;

    audit(audit_service, &ctx, "generate_output", "/api/v1/outputs").await;
    Ok(Json(GenerateOutputResponse {
        output_type: output.output_type,
        mode: output.mode,
        watermark: output.watermark,
        content: output.content,
        template_id: output.template_id,
        template_version_no: output.template_version_no,
    }))
}

#[get("/outputs?<page>&<limit>&<sort_by>&<sort_order>&<filter>")]
pub async fn list_outputs(
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
    page: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    filter: Option<String>,
) -> ApiResult<Json<Vec<OutputRow>>> {
    RbacService::require_reporting(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot list outputs"))?;
    let params = PaginationParams {
        page,
        limit,
        sort_by,
        sort_order,
        filter,
    };

    let mut rows = sqlx::query_as::<_, OutputRow>(
        "SELECT id, session_id, output_type, mode, created_at FROM print_outputs ORDER BY created_at DESC",
    )
    .fetch_all(pool.inner())
    .await
    .unwrap_or_default();

    if let Some(filter) = params.filter.as_deref().map(str::to_lowercase) {
        rows.retain(|r| {
            r.session_id.to_lowercase().contains(&filter)
                || r.output_type.to_lowercase().contains(&filter)
                || r.mode.to_lowercase().contains(&filter)
        });
    }

    rows = rows
        .into_iter()
        .skip(params.offset() as usize)
        .take(params.limit() as usize)
        .collect();

    audit(audit_service, &ctx, "list_outputs", "/api/v1/outputs").await;
    Ok(Json(rows))
}

// Query-tolerant fallback route. Some clients send malformed trailing '&' query strings.
#[get("/outputs")]
pub async fn list_outputs_fallback(
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<Vec<OutputRow>>> {
    RbacService::require_reporting(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot list outputs"))?;

    let rows = sqlx::query_as::<_, OutputRow>(
        "SELECT id, session_id, output_type, mode, created_at FROM print_outputs ORDER BY created_at DESC LIMIT 50",
    )
    .fetch_all(pool.inner())
    .await
    .unwrap_or_default();

    audit(audit_service, &ctx, "list_outputs", "/api/v1/outputs").await;
    Ok(Json(rows))
}

#[post("/outputs/admit-cards", data = "<payload>")]
pub async fn print_admit_cards(
    payload: Json<GenerateOutputRequest>,
    output_service: &State<OutputService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<GenerateOutputResponse>> {
    generate_output(
        Json(GenerateOutputRequest {
            output_type: "AdmitCard".to_string(),
            ..payload.0
        }),
        output_service,
        audit_service,
        ctx,
    )
    .await
}

#[post("/outputs/seating-charts", data = "<payload>")]
pub async fn print_seating_charts(
    payload: Json<GenerateOutputRequest>,
    output_service: &State<OutputService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<GenerateOutputResponse>> {
    generate_output(
        Json(GenerateOutputRequest {
            output_type: "SeatingChart".to_string(),
            ..payload.0
        }),
        output_service,
        audit_service,
        ctx,
    )
    .await
}

#[post("/outputs/door-signs", data = "<payload>")]
pub async fn print_door_signs(
    payload: Json<GenerateOutputRequest>,
    output_service: &State<OutputService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<GenerateOutputResponse>> {
    generate_output(
        Json(GenerateOutputRequest {
            output_type: "DoorSign".to_string(),
            ..payload.0
        }),
        output_service,
        audit_service,
        ctx,
    )
    .await
}

#[post("/outputs/proctor-packet", data = "<payload>")]
pub async fn print_proctor_packet(
    payload: Json<GenerateOutputRequest>,
    output_service: &State<OutputService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<GenerateOutputResponse>> {
    generate_output(
        Json(GenerateOutputRequest {
            output_type: "ProctorPacket".to_string(),
            ..payload.0
        }),
        output_service,
        audit_service,
        ctx,
    )
    .await
}

#[post("/outputs/summary-report", data = "<payload>")]
pub async fn print_summary_report(
    payload: Json<GenerateOutputRequest>,
    output_service: &State<OutputService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<GenerateOutputResponse>> {
    generate_output(
        Json(GenerateOutputRequest {
            output_type: "SummaryReport".to_string(),
            ..payload.0
        }),
        output_service,
        audit_service,
        ctx,
    )
    .await
}

#[post("/messages/drafts", data = "<payload>")]
pub async fn create_message_draft(
    payload: Json<MessageDraftRequest>,
    messaging_service: &State<MessagingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot create message draft"))?;
    let user_id = actor_user_id(&ctx)?;

    messaging_service
        .create_message_draft(
            &payload.channel,
            &payload.recipient,
            payload.subject.as_deref(),
            &payload.body,
            user_id,
        )
        .await
        .map_err(|_| ApiError::internal("failed to create message draft"))?;

    audit(
        audit_service,
        &ctx,
        "create_message_draft",
        "/api/v1/messages/drafts",
    )
    .await;
    Ok(Status::Created)
}

#[post("/attachments", data = "<payload>")]
pub async fn upload_attachment(
    payload: Json<AttachmentUploadRequest>,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot upload attachments"))?;
    let user_id = actor_user_id(&ctx)?;
    let admin = is_admin(&ctx);

    validate_record_access(
        pool.inner(),
        user_id,
        admin,
        &payload.record_type,
        &payload.record_id,
    )
    .await?;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&payload.bytes_base64)
        .map_err(|_| ApiError::bad_request("bytes_base64 invalid"))?;

    validate_file_size(bytes.len())
        .map_err(|_| ApiError::bad_request("file exceeds 25MB limit"))?;
    validate_extension(&payload.extension)
        .map_err(|_| ApiError::bad_request("file extension is not allowed"))?;

    let existing_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM attachments WHERE record_type = ? AND record_id = ?",
    )
    .bind(&payload.record_type)
    .bind(&payload.record_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|_| ApiError::internal("failed counting attachments"))?;

    validate_file_count((existing_count + 1) as usize)
        .map_err(|_| ApiError::bad_request("Maximum 10 files per record exceeded"))?;

    let fingerprint = app_core::file_policy::sha256_fingerprint(&bytes);

    let duplicate_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM attachments WHERE record_type = ? AND record_id = ? AND fingerprint_sha256 = ?",
    )
    .bind(&payload.record_type)
    .bind(&payload.record_id)
    .bind(&fingerprint)
    .fetch_one(pool.inner())
    .await
    .map_err(|_| ApiError::internal("failed duplicate fingerprint check"))?;

    if duplicate_count > 0 {
        return Err(ApiError::conflict(
            "duplicate attachment fingerprint for this record",
        ));
    }

    let captured_at = Utc::now();
    let _metadata = CaptureMetadata {
        operator: payload.operator_label.clone(),
        captured_at,
        device_label: payload.device_label.clone(),
    };

    let new_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO attachments (id, record_type, record_id, file_name, extension, size_bytes, fingerprint_sha256, file_blob, operator_label, device_label, captured_at, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&new_id)
        .bind(&payload.record_type)
        .bind(&payload.record_id)
        .bind(&payload.file_name)
        .bind(&payload.extension)
        .bind(bytes.len() as i64)
        .bind(fingerprint.clone())
        .bind(bytes)
        .bind(&payload.operator_label)
        .bind(&payload.device_label)
        .bind(captured_at.naive_utc())
        .bind(user_id)
        .execute(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to save attachment metadata"))?;

    let _ = audit_service
        .record_change(
            "attachments",
            &new_id,
            "CREATE",
            None,
            Some(serde_json::json!({"record_type": payload.record_type, "record_id": payload.record_id, "fingerprint_sha256": fingerprint})),
            user_id,
        )
        .await;

    audit(
        audit_service,
        &ctx,
        "upload_attachment",
        "/api/v1/attachments",
    )
    .await;
    Ok(Status::Created)
}

#[get("/attachments?<record_type>&<record_id>")]
pub async fn list_attachments(
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
    record_type: Option<String>,
    record_id: Option<String>,
) -> ApiResult<Json<Vec<AttachmentRow>>> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot list attachments"))?;
    let user_id = actor_user_id(&ctx)?;
    let admin = is_admin(&ctx);

    let mut query = String::from(
        "SELECT id, record_type, record_id, file_name, extension, size_bytes, fingerprint_sha256, operator_label, device_label, captured_at, created_at FROM attachments WHERE 1=1",
    );
    if !admin {
        query.push_str(" AND created_by = ?");
    }
    if record_type.is_some() {
        query.push_str(" AND record_type = ?");
    }
    if record_id.is_some() {
        query.push_str(" AND record_id = ?");
    }
    query.push_str(" ORDER BY created_at DESC LIMIT 200");

    let mut q = sqlx::query_as::<_, AttachmentRow>(&query);
    if !admin {
        q = q.bind(user_id);
    }
    if let Some(v) = &record_type {
        q = q.bind(v);
    }
    if let Some(v) = &record_id {
        q = q.bind(v);
    }

    let rows = q
        .fetch_all(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to list attachments"))?;

    audit(
        audit_service,
        &ctx,
        "list_attachments",
        "/api/v1/attachments",
    )
    .await;
    Ok(Json(rows))
}

#[get("/attachments/<id>")]
pub async fn get_attachment(
    id: &str,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<AttachmentFileResponse>> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot get attachments"))?;
    let user_id = actor_user_id(&ctx)?;
    let admin = is_admin(&ctx);

    let row = if admin {
        sqlx::query_as::<_, (String, String, String, Vec<u8>)>(
            "SELECT id, file_name, extension, file_blob FROM attachments WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to fetch attachment"))?
    } else {
        sqlx::query_as::<_, (String, String, String, Vec<u8>)>(
            "SELECT id, file_name, extension, file_blob FROM attachments WHERE id = ? AND created_by = ?",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to fetch attachment"))?
    };

    let Some((attachment_id, file_name, extension, bytes)) = row else {
        return Err(ApiError::not_found("attachment not found"));
    };

    audit(
        audit_service,
        &ctx,
        "get_attachment",
        "/api/v1/attachments/{id}",
    )
    .await;
    Ok(Json(AttachmentFileResponse {
        id: attachment_id,
        file_name,
        extension,
        bytes_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
    }))
}

async fn validate_record_access(
    pool: &MySqlPool,
    user_id: &str,
    is_admin: bool,
    record_type: &str,
    record_id: &str,
) -> ApiResult<()> {
    let record_type_norm = record_type.trim().to_ascii_lowercase();
    let exists = match record_type_norm.as_str() {
        "candidate" | "candidates" => {
            if is_admin {
                sqlx::query_scalar::<_, i64>("SELECT 1 FROM candidates WHERE id = ? LIMIT 1")
                    .bind(record_id)
                    .fetch_optional(pool)
                    .await
                    .map_err(|_| ApiError::internal("failed candidate ownership check"))?
                    .is_some()
            } else {
                sqlx::query_scalar::<_, i64>(
                    "SELECT 1 FROM candidates WHERE id = ? AND created_by = ? LIMIT 1",
                )
                .bind(record_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await
                .map_err(|_| ApiError::internal("failed candidate ownership check"))?
                .is_some()
            }
        }
        "room" | "rooms" => {
            if is_admin {
                sqlx::query_scalar::<_, i64>("SELECT 1 FROM rooms WHERE id = ? LIMIT 1")
                    .bind(record_id)
                    .fetch_optional(pool)
                    .await
                    .map_err(|_| ApiError::internal("failed room ownership check"))?
                    .is_some()
            } else {
                sqlx::query_scalar::<_, i64>(
                    "SELECT 1 FROM rooms WHERE id = ? AND created_by = ? LIMIT 1",
                )
                .bind(record_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await
                .map_err(|_| ApiError::internal("failed room ownership check"))?
                .is_some()
            }
        }
        "session" | "sessions" => {
            if is_admin {
                sqlx::query_scalar::<_, i64>("SELECT 1 FROM exam_sessions WHERE id = ? LIMIT 1")
                    .bind(record_id)
                    .fetch_optional(pool)
                    .await
                    .map_err(|_| ApiError::internal("failed session ownership check"))?
                    .is_some()
            } else {
                sqlx::query_scalar::<_, i64>(
                    "SELECT 1 FROM exam_sessions WHERE id = ? AND created_by = ? LIMIT 1",
                )
                .bind(record_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await
                .map_err(|_| ApiError::internal("failed session ownership check"))?
                .is_some()
            }
        }
        "asset" | "assets" => {
            if is_admin {
                sqlx::query_scalar::<_, i64>("SELECT 1 FROM assets WHERE id = ? LIMIT 1")
                    .bind(record_id)
                    .fetch_optional(pool)
                    .await
                    .map_err(|_| ApiError::internal("failed asset ownership check"))?
                    .is_some()
            } else {
                sqlx::query_scalar::<_, i64>(
                    "SELECT 1 FROM assets WHERE id = ? AND created_by = ? LIMIT 1",
                )
                .bind(record_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await
                .map_err(|_| ApiError::internal("failed asset ownership check"))?
                .is_some()
            }
        }
        _ => return Err(ApiError::bad_request("unsupported attachment record_type")),
    };

    if !exists {
        return Err(ApiError::forbidden(
            "record does not exist or is not accessible to current user",
        ));
    }
    Ok(())
}
