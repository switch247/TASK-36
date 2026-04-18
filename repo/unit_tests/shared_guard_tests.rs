#[path = "../API_tests/common.rs"]
mod common;

#[cfg(test)]
mod tests {
    use std::env;

    use rocket::http::{Header, Status};
    use rocket::local::asynchronous::Client;
    use rocket::serde::json::Json;
    use serde_json::{json, Value};

    use app_api_v1::shared::ApiContext;
    use app_services::auth_service::AuthService;

    #[rocket::get("/whoami")]
    async fn whoami(ctx: ApiContext) -> Json<Value> {
        Json(json!({
            "user_id": ctx.actor.user_id,
            "role": format!("{:?}", ctx.actor.role),
            "ip_address": ctx.ip_address
        }))
    }

    async fn guarded_client(pool: sqlx::MySqlPool) -> Client {
        let jwt_secret = env::var("TEST_JWT_SECRET")
            .ok()
            .or_else(|| env::var("JWT_SECRET").ok())
            .unwrap_or_else(|| "test-jwt-secret-change-me".to_string());
        let auth_service = AuthService::new(pool, jwt_secret);
        let rocket = rocket::build().manage(auth_service).mount("/", rocket::routes![whoami]);
        rocket::local::asynchronous::Client::tracked(rocket)
            .await
            .expect("tracked client")
    }

    #[rocket::async_test]
    async fn api_context_rejects_missing_and_malformed_credentials() {
        let app = common::setup_app().await.expect("setup");
        let client = guarded_client(app.pool.clone()).await;

        let missing = client.get("/whoami").dispatch().await;
        assert_eq!(missing.status(), Status::Unauthorized);
        let body: Value = missing.into_json().await.expect("json");
        assert!(body["message"].as_str().unwrap_or_default().contains("missing credentials"));

        let malformed = client
            .get("/whoami")
            .header(Header::new("Authorization", "Token abc"))
            .dispatch()
            .await;
        assert_eq!(malformed.status(), Status::Unauthorized);
        let body: Value = malformed.into_json().await.expect("json");
        assert!(body["message"].as_str().unwrap_or_default().contains("invalid authorization scheme"));
    }

    #[rocket::async_test]
    async fn api_context_accepts_session_and_jwt_and_refreshes_session_binding() {
        let app = common::setup_app().await.expect("setup");
        let client = guarded_client(app.pool.clone()).await;

        let (status, body) = common::login(&app.client, common::ADMIN_USERNAME, common::ADMIN_PASSWORD).await;
        assert_eq!(status, Status::Ok);
        let body = body.expect("login body");
        let session_id = body["session_id"].as_str().expect("session").to_string();
        let jwt = body["jwt"].as_str().expect("jwt").to_string();

        let before: chrono::NaiveDateTime =
            sqlx::query_scalar("SELECT expires_at FROM user_sessions WHERE id = ?")
                .bind(&session_id)
                .fetch_one(&app.pool)
                .await
                .expect("expires before");

        let session_only = client
            .get("/whoami")
            .header(Header::new("x-session-id", session_id.clone()))
            .dispatch()
            .await;
        assert_eq!(session_only.status(), Status::Ok);
        let body: Value = session_only.into_json().await.expect("json");
        assert_eq!(body["role"], "Admin");

        let jwt_and_session = client
            .get("/whoami")
            .header(Header::new("Authorization", format!("Bearer {jwt}")))
            .header(Header::new("x-session-id", session_id.clone()))
            .dispatch()
            .await;
        assert_eq!(jwt_and_session.status(), Status::Ok);
        let body: Value = jwt_and_session.into_json().await.expect("json");
        assert!(body["user_id"].is_string());

        let after: chrono::NaiveDateTime =
            sqlx::query_scalar("SELECT expires_at FROM user_sessions WHERE id = ?")
                .bind(&session_id)
                .fetch_one(&app.pool)
                .await
                .expect("expires after");
        assert!(after >= before, "guard should refresh session expiry");
    }

    #[rocket::async_test]
    async fn api_context_rejects_invalid_session_binding() {
        let app = common::setup_app().await.expect("setup");
        let client = guarded_client(app.pool.clone()).await;

        let invalid = client
            .get("/whoami")
            .header(Header::new("x-session-id", "not-a-real-session"))
            .dispatch()
            .await;
        assert_eq!(invalid.status(), Status::Unauthorized);
        let body: Value = invalid.into_json().await.expect("json");
        assert_eq!(body["code"], 401);
        assert!(body["message"].as_str().unwrap_or_default().contains("invalid session"));
    }
}
