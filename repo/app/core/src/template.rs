use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::errors::CoreError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldRule {
    Required,
    Range { min: f64, max: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormTemplate {
    pub template_id: String,
    pub version: i32,
    pub is_locked: bool,
    pub rules: HashMap<String, Vec<FieldRule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSnapshot {
    pub template_id: String,
    pub version: i32,
    pub snapshot_json: serde_json::Value,
    pub locked_for_final_print: bool,
}

pub fn validate_form_payload(
    template: &FormTemplate,
    payload: &HashMap<String, serde_json::Value>,
) -> Result<(), CoreError> {
    for (field, rules) in &template.rules {
        for rule in rules {
            match rule {
                FieldRule::Required => {
                    let present = payload.get(field).is_some_and(|v| !v.is_null());
                    if !present {
                        return Err(CoreError::TemplateValidationError(format!(
                            "required field missing: {field}"
                        )));
                    }
                }
                FieldRule::Range { min, max } => {
                    let value = payload.get(field).and_then(|v| v.as_f64()).ok_or_else(|| {
                        CoreError::TemplateValidationError(format!(
                            "field {} must be numeric for range checks",
                            field
                        ))
                    })?;

                    if value < *min || value > *max {
                        return Err(CoreError::TemplateValidationError(format!(
                            "field {} value {} outside [{}, {}]",
                            field, value, min, max
                        )));
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn lock_template_for_final_print(template: &mut FormTemplate) -> TemplateSnapshot {
    template.is_locked = true;

    let snapshot = serde_json::json!({
        "template_id": template.template_id,
        "version": template.version,
        "rules": template.rules,
    });

    TemplateSnapshot {
        template_id: template.template_id.clone(),
        version: template.version,
        snapshot_json: snapshot,
        locked_for_final_print: true,
    }
}
