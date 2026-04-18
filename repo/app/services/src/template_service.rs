use std::collections::HashMap;

use anyhow::Result;
use serde_json::Value;

use app_core::template::{lock_template_for_final_print, validate_form_payload, FieldRule, FormTemplate, TemplateSnapshot};

pub struct TemplateService;

impl TemplateService {
    pub fn validate_submission(
        template: &FormTemplate,
        payload: &HashMap<String, Value>,
    ) -> Result<()> {
        validate_form_payload(template, payload)?;
        Ok(())
    }

    pub fn lock_for_final_print(template: &mut FormTemplate) -> TemplateSnapshot {
        lock_template_for_final_print(template)
    }

    pub fn sample_template() -> FormTemplate {
        let mut rules = HashMap::new();
        rules.insert(
            "room_capacity".to_string(),
            vec![FieldRule::Required, FieldRule::Range { min: 1.0, max: 500.0 }],
        );
        rules.insert("operator".to_string(), vec![FieldRule::Required]);

        FormTemplate {
            template_id: "exam-room-config".to_string(),
            version: 1,
            is_locked: false,
            rules,
        }
    }
}
