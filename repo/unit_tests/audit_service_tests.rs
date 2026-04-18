#[path = "../API_tests/common.rs"]
mod common;

#[cfg(test)]
mod tests {
    use super::common;
    use app_services::audit_service::AuditService;
    use serde_json::json;

    #[rocket::async_test]
    async fn record_api_call_persists_audit_log_row() {
        let app = common::setup_app().await.expect("setup");
        let service = AuditService::new(app.pool.clone());

        service
            .record_api_call(
                Some("user-1"),
                "list_candidates",
                "/api/v1/candidates",
                "127.0.0.1",
            )
            .await
            .expect("record api call");

        let row: (Option<String>, String, String, String) = sqlx::query_as(
            "SELECT actor_user_id, action, resource, ip_address FROM audit_logs ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_one(&app.pool)
        .await
        .expect("audit row");
        assert_eq!(row.0.as_deref(), Some("user-1"));
        assert_eq!(row.1, "list_candidates");
        assert_eq!(row.2, "/api/v1/candidates");
        assert_eq!(row.3, "127.0.0.1");
    }

    #[rocket::async_test]
    async fn record_change_persists_before_and_after_state() {
        let app = common::setup_app().await.expect("setup");
        let service = AuditService::new(app.pool.clone());

        service
            .record_change(
                "candidates",
                "cand-1",
                "UPDATE",
                Some(json!({"name":"Old"})),
                Some(json!({"name":"New"})),
                "user-1",
            )
            .await
            .expect("record change");

        let row: (String, String, serde_json::Value, serde_json::Value, String) = sqlx::query_as(
            "SELECT entity_name, entity_id, previous_state, new_state, changed_by FROM entity_change_history ORDER BY changed_at DESC LIMIT 1",
        )
        .fetch_one(&app.pool)
        .await
        .expect("change row");
        assert_eq!(row.0, "candidates");
        assert_eq!(row.1, "cand-1");
        assert_eq!(row.2["name"], "Old");
        assert_eq!(row.3["name"], "New");
        assert_eq!(row.4, "user-1");
    }

    #[rocket::async_test]
    async fn record_api_call_supports_anonymous_actor() {
        let app = common::setup_app().await.expect("setup");
        let service = AuditService::new(app.pool.clone());

        service
            .record_api_call(None, "healthcheck", "/api/v1/health", "unknown")
            .await
            .expect("record anonymous api call");

        let row: (Option<String>, String, String) = sqlx::query_as(
            "SELECT actor_user_id, action, resource FROM audit_logs ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_one(&app.pool)
        .await
        .expect("audit row");
        assert!(
            row.0.is_none(),
            "anonymous audit rows must keep actor_user_id null"
        );
        assert_eq!(row.1, "healthcheck");
        assert_eq!(row.2, "/api/v1/health");
    }
}
