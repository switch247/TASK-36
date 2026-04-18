use std::collections::HashMap;

use app_core::template::{FieldRule, FormTemplate};
use app_services::template_service::TemplateService;
use serde_json::Value;
use sqlx::MySqlPool;

use crate::errors::{ApiError, ApiResult};

#[derive(sqlx::FromRow)]
struct TemplateRow {
    template_id: String,
    version_no: i32,
    snapshot: Value,
    locked_for_final_print: bool,
}

#[derive(serde::Deserialize)]
struct SnapshotFormTemplate {
    template_id: Option<String>,
    version: Option<i32>,
    rules: HashMap<String, Vec<FieldRule>>,
}

pub async fn validate_against_template(
    pool: &MySqlPool,
    template_id: &str,
    payload: HashMap<String, Value>,
) -> ApiResult<()> {
    validate_against_template_with_mode(pool, template_id, payload, false).await
}

pub async fn validate_against_template_partial(
    pool: &MySqlPool,
    template_id: &str,
    payload: HashMap<String, Value>,
) -> ApiResult<()> {
    validate_against_template_with_mode(pool, template_id, payload, true).await
}

async fn validate_against_template_with_mode(
    pool: &MySqlPool,
    template_id: &str,
    payload: HashMap<String, Value>,
    partial: bool,
) -> ApiResult<()> {
    let row = sqlx::query_as::<_, TemplateRow>(
        "SELECT template_id, version_no, snapshot, locked_for_final_print
         FROM template_versions
         WHERE template_id = ?
         ORDER BY version_no DESC
         LIMIT 1",
    )
    .bind(template_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| ApiError::internal("failed loading template version"))?
    .ok_or_else(|| ApiError::bad_request("template version not found for entity validation"))?;

    let template = parse_form_template(&row)
        .ok_or_else(|| ApiError::bad_request("template snapshot does not contain form rules"))?;
    let filtered_template = if partial {
        let rules = template
            .rules
            .into_iter()
            .filter(|(k, _)| payload.contains_key(k))
            .collect();
        FormTemplate {
            template_id: template.template_id,
            version: template.version,
            is_locked: template.is_locked,
            rules,
        }
    } else {
        template
    };

    TemplateService::validate_submission(&filtered_template, &payload)
        .map_err(|err| ApiError::bad_request(format!("template validation failed: {err}")))?;
    Ok(())
}

fn parse_form_template(row: &TemplateRow) -> Option<FormTemplate> {
    if let Some(form_t) = row.snapshot.get("form_template") {
        if let Ok(parsed) = serde_json::from_value::<SnapshotFormTemplate>(form_t.clone()) {
            return Some(FormTemplate {
                template_id: parsed
                    .template_id
                    .unwrap_or_else(|| row.template_id.clone()),
                version: parsed.version.unwrap_or(row.version_no),
                is_locked: row.locked_for_final_print,
                rules: parsed.rules,
            });
        }
    }

    let rules_val = row.snapshot.get("rules")?.clone();
    let rules = serde_json::from_value::<HashMap<String, Vec<FieldRule>>>(rules_val).ok()?;
    Some(FormTemplate {
        template_id: row.template_id.clone(),
        version: row.version_no,
        is_locked: row.locked_for_final_print,
        rules,
    })
}
