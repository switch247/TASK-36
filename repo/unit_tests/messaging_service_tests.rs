#[path = "../API_tests/common.rs"]
mod common;

#[cfg(test)]
mod tests {
    use super::common;
    use app_services::messaging_service::MessagingService;

    #[rocket::async_test]
    async fn create_message_draft_persists_email_draft() {
        let app = common::setup_app().await.expect("setup");
        let service = MessagingService::new(app.pool.clone());

        let draft = service
            .create_message_draft(
                "Email",
                "ops@example.com",
                Some("Operations Alert"),
                "Review incident queue",
                "user-1",
            )
            .await
            .expect("create draft");

        assert_eq!(draft.channel, "Email");
        assert_eq!(draft.recipient, "ops@example.com");
        assert_eq!(draft.subject.as_deref(), Some("Operations Alert"));

        let row: (String, String, Option<String>, String, String) = sqlx::query_as(
            "SELECT channel, recipient, subject, body, created_by FROM message_drafts WHERE id = ?",
        )
        .bind(&draft.id)
        .fetch_one(&app.pool)
        .await
        .expect("draft row");
        assert_eq!(row.0, "Email");
        assert_eq!(row.1, "ops@example.com");
        assert_eq!(row.2.as_deref(), Some("Operations Alert"));
        assert_eq!(row.3, "Review incident queue");
        assert_eq!(row.4, "user-1");
    }

    #[rocket::async_test]
    async fn create_message_draft_supports_missing_subject() {
        let app = common::setup_app().await.expect("setup");
        let service = MessagingService::new(app.pool.clone());

        let draft = service
            .create_message_draft("SMS", "+15551234567", None, "Door signs ready", "user-2")
            .await
            .expect("create draft");

        assert!(draft.subject.is_none());

        let row: Option<String> =
            sqlx::query_scalar("SELECT subject FROM message_drafts WHERE id = ?")
                .bind(&draft.id)
                .fetch_one(&app.pool)
                .await
                .expect("draft subject");
        assert!(row.is_none());
    }

    #[rocket::async_test]
    async fn create_message_draft_returns_stable_message_contract() {
        let app = common::setup_app().await.expect("setup");
        let service = MessagingService::new(app.pool.clone());

        let draft = service
            .create_message_draft(
                "Email",
                "alerts@example.com",
                Some("Nightly Digest"),
                "All systems nominal",
                "user-3",
            )
            .await
            .expect("create draft");

        assert!(
            uuid::Uuid::parse_str(&draft.id).is_ok(),
            "draft ids must be UUIDs"
        );
        assert_eq!(draft.body, "All systems nominal");
        assert_eq!(draft.subject.as_deref(), Some("Nightly Digest"));
    }
}
