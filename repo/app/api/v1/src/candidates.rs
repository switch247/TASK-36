use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put, State};

use sqlx::{MySql, MySqlPool, QueryBuilder};
use tracing::info;

use app_services::audit_service::AuditService;
use app_services::candidate_service::CandidateService;
use app_services::cleansing_service::CleansingService;
use app_services::rbac_service::RbacService;

use crate::errors::{ApiError, ApiResult};
use crate::pagination::PaginationParams;
use crate::shared::{actor_user_id, audit, is_admin, ApiContext};
use crate::template_validation::{validate_against_template, validate_against_template_partial};

#[derive(Debug, serde::Deserialize)]
pub struct CreateCandidateRequest {
    pub candidate_id: String,
    pub date_of_birth: String,
    pub national_id: String,
    pub scanned_barcode: String,
    pub metadata_json: String,
    pub template_id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateCandidateRequest {
    pub scanned_barcode: String,
    pub metadata_json: String,
    pub template_id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct MergeCandidateRequest {
    pub left_candidate_id: String,
    pub right_candidate_id: String,
    pub similarity_score: f64,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct CandidateRow {
    pub id: String,
    pub dob_masked: String,
    pub national_id: String,
    pub scanned_barcode: String,
    pub metadata: serde_json::Value,
    pub created_by: String,
}

async fn parse_and_enrich_candidate_metadata(
    metadata_json: &str,
    cleansing_service: &CleansingService,
    normalized_dob: &str,
) -> ApiResult<String> {
    let mut metadata: serde_json::Value = serde_json::from_str(metadata_json)
        .map_err(|_| ApiError::bad_request("metadata_json must be valid JSON"))?;

    // Field mapping pipeline: map common aliases to canonical fields.
    if metadata.get("zip_code").is_none() {
        if let Some(alias_zip) = metadata
            .get("zipcode")
            .or_else(|| metadata.get("zip"))
            .and_then(serde_json::Value::as_str)
        {
            metadata["zip_code"] = serde_json::Value::String(alias_zip.trim().to_string());
        }
    }
    if metadata.get("city").is_none() {
        if let Some(alias_city) = metadata
            .get("town")
            .or_else(|| metadata.get("city_name"))
            .and_then(serde_json::Value::as_str)
        {
            metadata["city"] = serde_json::Value::String(alias_city.trim().to_string());
        }
    }
    if metadata.get("name").is_none() {
        if let Some(alias_name) = metadata
            .get("full_name")
            .or_else(|| metadata.get("candidate_name"))
            .and_then(serde_json::Value::as_str)
        {
            metadata["name"] = serde_json::Value::String(alias_name.trim().to_string());
        }
    }

    let zip_code = metadata.get("zip_code").and_then(serde_json::Value::as_str);
    let city = metadata.get("city").and_then(serde_json::Value::as_str);
    let zip_city_valid = match (zip_code, city) {
        (Some(zip), Some(cty)) => cleansing_service
            .validate_zip_city(zip, cty)
            .await
            .map_err(|_| ApiError::internal("failed ZIP/city validation"))?,
        _ => true,
    };
    if !zip_city_valid {
        return Err(ApiError::bad_request("ZIP/city pair is invalid"));
    }

    if !metadata.is_object() {
        metadata = serde_json::json!({});
    }

    // Missing-value defaults (explicitly captured for operators/reviewers).
    let mut defaults_applied: Vec<String> = Vec::new();
    if metadata
        .get("name")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        metadata["name"] = serde_json::Value::String("Unknown Candidate".to_string());
        defaults_applied.push("name".to_string());
    }
    if metadata
        .get("room_id")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        metadata["room_id"] = serde_json::Value::String("unassigned".to_string());
        defaults_applied.push("room_id".to_string());
    }

    // Optional normalized numeric/date fields when present.
    let has_norm_bundle = [
        "measurement_value",
        "measurement_unit",
        "amount",
        "currency",
        "fx_rate_to_usd",
        "effective_date",
    ]
    .iter()
    .any(|k| metadata.get(*k).is_some());
    let mut normalized_record = serde_json::Value::Null;
    if has_norm_bundle {
        let value = metadata
            .get("measurement_value")
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| {
                ApiError::bad_request(
                    "measurement_value is required when normalization fields are provided",
                )
            })?;
        let unit = metadata
            .get("measurement_unit")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                ApiError::bad_request(
                    "measurement_unit is required when normalization fields are provided",
                )
            })?;
        let amount = metadata
            .get("amount")
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| {
                ApiError::bad_request("amount is required when normalization fields are provided")
            })?;
        let currency = metadata
            .get("currency")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                ApiError::bad_request("currency is required when normalization fields are provided")
            })?;
        let fx_rate = metadata
            .get("fx_rate_to_usd")
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| {
                ApiError::bad_request(
                    "fx_rate_to_usd is required when normalization fields are provided",
                )
            })?;
        let effective_date = metadata
            .get("effective_date")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                ApiError::bad_request(
                    "effective_date is required when normalization fields are provided",
                )
            })?;
        let norm = cleansing_service
            .normalize_record(value, unit, amount, currency, fx_rate, effective_date)
            .map_err(|_| ApiError::bad_request("failed to normalize record fields"))?;
        normalized_record = serde_json::json!({
            "value": norm.normalized_value,
            "unit": norm.normalized_unit,
            "amount_usd": norm.normalized_amount_usd,
            "normalized_date": norm.normalized_date
        });
    }

    metadata["cleansing"] = serde_json::json!({
        "normalized_dob": normalized_dob,
        "zip_city_valid": zip_city_valid,
        "defaults_applied": defaults_applied,
        "field_mapping_applied": true,
        "normalized_record": normalized_record
    });

    serde_json::to_string(&metadata)
        .map_err(|_| ApiError::internal("failed to serialize candidate metadata"))
}

#[post("/candidates", data = "<payload>")]
pub async fn create_candidate(
    payload: Json<CreateCandidateRequest>,
    pool: &State<MySqlPool>,
    candidate_service: &State<CandidateService>,
    cleansing_service: &State<CleansingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot create candidate"))?;
    let user_id = actor_user_id(&ctx)?;
    let normalized_dob = candidate_service
        .normalize_dob_mmddyyyy(&payload.date_of_birth)
        .map_err(|_| ApiError::bad_request("date_of_birth must use MM/DD/YYYY"))?;

    let enriched_metadata_json = parse_and_enrich_candidate_metadata(
        &payload.metadata_json,
        cleansing_service.inner(),
        &normalized_dob,
    )
    .await?;
    let metadata_obj: serde_json::Value = serde_json::from_str(&enriched_metadata_json)
        .map_err(|_| ApiError::internal("failed to parse enriched metadata"))?;
    let mut template_payload = std::collections::HashMap::new();
    template_payload.insert(
        "date_of_birth".to_string(),
        serde_json::Value::String(payload.date_of_birth.clone()),
    );
    template_payload.insert(
        "national_id".to_string(),
        serde_json::Value::String(payload.national_id.clone()),
    );
    template_payload.insert(
        "scanned_barcode".to_string(),
        serde_json::Value::String(payload.scanned_barcode.clone()),
    );
    if let Some(name) = metadata_obj.get("name").cloned() {
        template_payload.insert("name".to_string(), name);
    }
    if let Some(room_id) = metadata_obj.get("room_id").cloned() {
        template_payload.insert("room_id".to_string(), room_id);
    }
    let template_id = payload
        .template_id
        .as_deref()
        .filter(|x| !x.trim().is_empty())
        .unwrap_or("candidate-registration");
    validate_against_template(pool.inner(), template_id, template_payload).await?;

    if let Some(existing_candidate_id) = candidate_service
        .find_exact_duplicate(&payload.national_id, &payload.scanned_barcode)
        .await
        .map_err(|_| ApiError::internal("failed duplicate detection"))?
    {
        return Err(ApiError::new(
            Status::Conflict,
            "duplicate candidate detected (exact national ID or barcode)",
            Some(serde_json::json!({
                "existing_candidate_id": existing_candidate_id,
                "match_type": "exact"
            })),
        ));
    }

    let incoming_name = serde_json::from_str::<serde_json::Value>(&enriched_metadata_json)
        .ok()
        .and_then(|v| {
            v.get("name")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        });
    if let Some((existing_candidate_id, similarity_score)) = candidate_service
        .find_guided_merge_duplicate(
            &payload.national_id,
            &normalized_dob,
            incoming_name.as_deref(),
        )
        .await
        .map_err(|_| ApiError::internal("failed guided merge detection"))?
    {
        return Err(ApiError::new(
            Status::Conflict,
            "possible duplicate candidate; guided merge review required",
            Some(serde_json::json!({
                "existing_candidate_id": existing_candidate_id,
                "match_type": "guided_merge",
                "similarity_score": similarity_score
            })),
        ));
    }

    candidate_service
        .create_candidate(
            &payload.candidate_id,
            &normalized_dob,
            &payload.national_id,
            &payload.scanned_barcode,
            &enriched_metadata_json,
            user_id,
        )
        .await
        .map_err(|_| ApiError::conflict("candidate already exists or payload invalid"))?;

    let _ = audit_service
        .record_change(
            "candidates",
            &payload.candidate_id,
            "CREATE",
            None,
            Some(serde_json::json!({"candidate_id": payload.candidate_id})),
            user_id,
        )
        .await;

    info!(action = "create_candidate", candidate_id = %payload.candidate_id, "candidate created");
    audit(
        audit_service,
        &ctx,
        "create_candidate",
        "/api/v1/candidates",
    )
    .await;
    Ok(Status::Created)
}

#[get("/candidates?<page>&<limit>&<sort_by>&<sort_order>&<filter>")]
pub async fn list_candidates(
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
    page: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    filter: Option<String>,
) -> ApiResult<Json<Vec<CandidateRow>>> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot list candidates"))?;
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
        Some("scanned_barcode") => "scanned_barcode",
        Some("national_id") => "national_id",
        _ => "created_at",
    };

    let filter_value = params
        .filter
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| format!("%{}%", v));

    let mut qb = QueryBuilder::<MySql>::new(
        "SELECT id, '**/**/****' AS dob_masked, national_id, scanned_barcode, metadata, created_by FROM candidates",
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
            .push(" OR scanned_barcode LIKE ")
            .push_bind(like.clone())
            .push(" OR national_id LIKE ")
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
        .build_query_as::<CandidateRow>()
        .fetch_all(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to list candidates"))?;

    audit(audit_service, &ctx, "list_candidates", "/api/v1/candidates").await;
    Ok(Json(rows))
}

#[get("/candidates/<id>")]
pub async fn get_candidate(
    id: &str,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Json<CandidateRow>> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot read candidate"))?;
    let user_id = actor_user_id(&ctx)?;

    let row = if is_admin(&ctx) {
        sqlx::query_as::<_, CandidateRow>(
            "SELECT id, '**/**/****' AS dob_masked, national_id, scanned_barcode, metadata, created_by FROM candidates WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to read candidate"))?
    } else {
        sqlx::query_as::<_, CandidateRow>(
            "SELECT id, '**/**/****' AS dob_masked, national_id, scanned_barcode, metadata, created_by FROM candidates WHERE id = ? AND created_by = ?",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to read candidate"))?
    };

    let row = row.ok_or_else(|| ApiError::not_found("candidate not found"))?;
    audit(
        audit_service,
        &ctx,
        "get_candidate",
        "/api/v1/candidates/{id}",
    )
    .await;
    Ok(Json(row))
}

#[put("/candidates/<id>", data = "<payload>")]
pub async fn update_candidate(
    id: &str,
    payload: Json<UpdateCandidateRequest>,
    pool: &State<MySqlPool>,
    cleansing_service: &State<CleansingService>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot update candidate"))?;
    let user_id = actor_user_id(&ctx)?;
    let existing_metadata: Option<serde_json::Value> = if is_admin(&ctx) {
        sqlx::query_scalar("SELECT metadata FROM candidates WHERE id = ?")
            .bind(id)
            .fetch_optional(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to read existing candidate metadata"))?
    } else {
        sqlx::query_scalar("SELECT metadata FROM candidates WHERE id = ? AND created_by = ?")
            .bind(id)
            .bind(user_id)
            .fetch_optional(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to read existing candidate metadata"))?
    };
    let existing_metadata =
        existing_metadata.ok_or_else(|| ApiError::not_found("candidate not found"))?;
    let existing_normalized_dob = existing_metadata
        .get("cleansing")
        .and_then(|v| v.get("normalized_dob"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");

    let existing_barcode_owner: Option<String> = sqlx::query_scalar(
        "SELECT id FROM candidates WHERE scanned_barcode = ? AND id <> ? LIMIT 1",
    )
    .bind(&payload.scanned_barcode)
    .bind(id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| ApiError::internal("failed duplicate barcode check"))?;
    if let Some(conflict_id) = existing_barcode_owner {
        return Err(ApiError::new(
            Status::Conflict,
            "duplicate scanned_barcode detected",
            Some(serde_json::json!({ "existing_candidate_id": conflict_id })),
        ));
    }

    let enriched_metadata_json = parse_and_enrich_candidate_metadata(
        &payload.metadata_json,
        cleansing_service.inner(),
        existing_normalized_dob,
    )
    .await?;
    let metadata: serde_json::Value = serde_json::from_str(&enriched_metadata_json)
        .map_err(|_| ApiError::internal("failed to parse enriched metadata"))?;
    let mut template_payload = std::collections::HashMap::new();
    template_payload.insert(
        "scanned_barcode".to_string(),
        serde_json::Value::String(payload.scanned_barcode.clone()),
    );
    if let Some(name) = metadata.get("name").cloned() {
        template_payload.insert("name".to_string(), name);
    }
    if let Some(room_id) = metadata.get("room_id").cloned() {
        template_payload.insert("room_id".to_string(), room_id);
    }
    let template_id = payload
        .template_id
        .as_deref()
        .filter(|x| !x.trim().is_empty())
        .unwrap_or("candidate-registration");
    validate_against_template_partial(pool.inner(), template_id, template_payload).await?;
    let metadata_json = serde_json::to_string(&metadata)
        .map_err(|_| ApiError::internal("failed to serialize candidate metadata"))?;

    let result = if is_admin(&ctx) {
        sqlx::query(
            "UPDATE candidates SET scanned_barcode = ?, metadata = CAST(? AS JSON) WHERE id = ?",
        )
        .bind(&payload.scanned_barcode)
        .bind(&metadata_json)
        .bind(id)
        .execute(pool.inner())
        .await
        .map_err(|_| ApiError::internal("failed to update candidate"))?
    } else {
        sqlx::query("UPDATE candidates SET scanned_barcode = ?, metadata = CAST(? AS JSON) WHERE id = ? AND created_by = ?")
            .bind(&payload.scanned_barcode)
            .bind(&metadata_json)
            .bind(id)
            .bind(user_id)
            .execute(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to update candidate"))?
    };

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("candidate not found"));
    }

    let _ = audit_service
        .record_change(
            "candidates",
            id,
            "UPDATE",
            None,
            Some(serde_json::json!({"scanned_barcode": payload.scanned_barcode})),
            user_id,
        )
        .await;

    audit(
        audit_service,
        &ctx,
        "update_candidate",
        "/api/v1/candidates/{id}",
    )
    .await;
    Ok(Status::Ok)
}

#[delete("/candidates/<id>")]
pub async fn delete_candidate(
    id: &str,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot delete candidate"))?;
    let user_id = actor_user_id(&ctx)?;

    let result = if is_admin(&ctx) {
        sqlx::query("DELETE FROM candidates WHERE id = ?")
            .bind(id)
            .execute(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to delete candidate"))?
    } else {
        sqlx::query("DELETE FROM candidates WHERE id = ? AND created_by = ?")
            .bind(id)
            .bind(user_id)
            .execute(pool.inner())
            .await
            .map_err(|_| ApiError::internal("failed to delete candidate"))?
    };

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("candidate not found"));
    }

    let _ = audit_service
        .record_change("candidates", id, "DELETE", None, None, user_id)
        .await;

    audit(
        audit_service,
        &ctx,
        "delete_candidate",
        "/api/v1/candidates/{id}",
    )
    .await;
    Ok(Status::NoContent)
}

#[post("/candidates/merge", data = "<payload>")]
pub async fn create_merge_candidate(
    payload: Json<MergeCandidateRequest>,
    pool: &State<MySqlPool>,
    audit_service: &State<AuditService>,
    ctx: ApiContext,
) -> ApiResult<Status> {
    RbacService::require_manage_inventory(&ctx.actor.role)
        .map_err(|_| ApiError::forbidden("role cannot create merge candidate"))?;
    let user_id = actor_user_id(&ctx)?;

    sqlx::query("INSERT INTO merge_candidates (id, left_candidate_id, right_candidate_id, similarity_score, created_by) VALUES (?, ?, ?, ?, ?)")
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&payload.left_candidate_id)
        .bind(&payload.right_candidate_id)
        .bind(payload.similarity_score)
        .bind(user_id)
        .execute(pool.inner())
        .await
        .map_err(|_| ApiError::conflict("merge candidate already exists or invalid"))?;

    let _ = audit_service
        .record_change(
            "merge_candidates",
            &format!(
                "{}:{}",
                payload.left_candidate_id, payload.right_candidate_id
            ),
            "CREATE",
            None,
            Some(serde_json::json!({"similarity_score": payload.similarity_score})),
            user_id,
        )
        .await;

    audit(
        audit_service,
        &ctx,
        "create_merge_candidate",
        "/api/v1/candidates/merge",
    )
    .await;
    Ok(Status::Created)
}
