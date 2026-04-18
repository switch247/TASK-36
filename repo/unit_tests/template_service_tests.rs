#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;

    use app_services::template_service::TemplateService;

    #[test]
    fn sample_template_accepts_valid_payload() {
        let template = TemplateService::sample_template();
        let payload = HashMap::from([
            ("room_capacity".to_string(), json!(120)),
            ("operator".to_string(), json!("coord_local")),
        ]);

        TemplateService::validate_submission(&template, &payload)
            .expect("sample template should accept valid payload");
    }

    #[test]
    fn sample_template_rejects_missing_required_field() {
        let template = TemplateService::sample_template();
        let payload = HashMap::from([("room_capacity".to_string(), json!(120))]);

        let err = TemplateService::validate_submission(&template, &payload)
            .expect_err("missing required field must fail");
        assert!(err.to_string().contains("required field missing"));
    }

    #[test]
    fn lock_for_final_print_marks_template_locked_and_returns_snapshot() {
        let mut template = TemplateService::sample_template();

        let snapshot = TemplateService::lock_for_final_print(&mut template);

        assert!(template.is_locked, "template should be locked in place");
        assert!(snapshot.locked_for_final_print);
        assert_eq!(snapshot.template_id, template.template_id);
        assert_eq!(snapshot.version, template.version);
        assert_eq!(snapshot.snapshot_json["template_id"], json!(template.template_id));
    }
}
