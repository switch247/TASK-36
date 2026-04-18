#[path = "../API_tests/common.rs"]
mod common;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::common;

    use serde_json::json;

    use app_api_v1::{validate_against_template, validate_against_template_partial};

    #[rocket::async_test]
    async fn template_validation_accepts_complete_payload_and_partial_updates() {
        let app = common::setup_app().await.expect("setup");

        let full = HashMap::from([
            ("date_of_birth".to_string(), json!("03/27/2001")),
            ("national_id".to_string(), json!("ID-TPL-1")),
            ("scanned_barcode".to_string(), json!("BAR-TPL-1")),
            ("name".to_string(), json!("Template Candidate")),
        ]);
        validate_against_template(&app.pool, "candidate-registration", full)
            .await
            .expect("full payload should validate");

        let partial = HashMap::from([("name".to_string(), json!("Template Candidate"))]);
        validate_against_template_partial(&app.pool, "candidate-registration", partial)
            .await
            .expect("partial payload should validate only supplied fields");
    }

    #[rocket::async_test]
    async fn template_validation_rejects_missing_template_and_missing_required_fields() {
        let app = common::setup_app().await.expect("setup");

        let missing_template = validate_against_template(&app.pool, "missing-template", HashMap::new())
            .await
            .expect_err("missing template must fail");
        assert_eq!(missing_template.status.code, 400);
        assert!(missing_template.body.message.contains("template version not found"));

        let incomplete = HashMap::from([
            ("date_of_birth".to_string(), json!("03/27/2001")),
            ("national_id".to_string(), json!("ID-TPL-2")),
        ]);
        let err = validate_against_template(&app.pool, "candidate-registration", incomplete)
            .await
            .expect_err("missing fields must fail");
        assert_eq!(err.status.code, 400);
        assert!(err.body.message.contains("template validation failed"));
    }

    #[rocket::async_test]
    async fn template_validation_rejects_malformed_snapshot_rules() {
        let app = common::setup_app().await.expect("setup");
        let admin_id = common::user_id_for(&app.pool, common::ADMIN_USERNAME).await;

        sqlx::query(
            "INSERT INTO template_versions (id, template_id, version_no, snapshot, locked_for_final_print, created_by)
             VALUES (?, ?, ?, CAST(? AS JSON), FALSE, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind("broken-template")
        .bind(1_i32)
        .bind(serde_json::json!({"rules":{"name":"Required"}}).to_string())
        .bind(admin_id)
        .execute(&app.pool)
        .await
        .expect("insert malformed template");

        let err = validate_against_template(
            &app.pool,
            "broken-template",
            HashMap::from([("name".to_string(), json!("Broken"))]),
        )
        .await
        .expect_err("malformed rules must fail");
        assert_eq!(err.status.code, 400);
        assert!(err.body.message.contains("form rules"));
    }
}
