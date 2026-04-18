mod common;

use rocket::http::Status;

use common::{
    auth_headers, login, setup_app, ADMIN_PASSWORD, ADMIN_USERNAME,
};

#[rocket::async_test]
async fn login_success_returns_tokens() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, ADMIN_USERNAME, ADMIN_PASSWORD).await;
    assert_eq!(status, Status::Ok);

    let body = body.expect("json body");
    let jwt = body["jwt"].as_str().expect("jwt string");
    assert!(jwt.split('.').count() == 3, "JWT must be three dot-separated segments");
    assert!(
        uuid::Uuid::parse_str(body["session_id"].as_str().expect("session id")).is_ok(),
        "session_id must be a UUID"
    );
    assert!(body["jwt_expires_at"].as_str().unwrap_or("").len() >= 20, "jwt_expires_at must look like RFC3339");
    assert!(body["session_expires_at"].as_str().unwrap_or("").len() >= 20);

    // DB-level verification: a user_sessions row must exist for the returned session id.
    let session_row_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_sessions WHERE id = ?")
        .bind(body["session_id"].as_str().unwrap())
        .fetch_one(&app.pool)
        .await
        .expect("session count");
    assert_eq!(session_row_count, 1, "login must persist a session row");
}

#[rocket::async_test]
async fn login_failure_wrong_password_returns_401() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, ADMIN_USERNAME, "WrongPass#2026!").await;
    assert_eq!(status, Status::Unauthorized);

    let body = body.expect("json body");
    assert_eq!(body["code"].as_u64(), Some(401));
    assert_eq!(body["message"], "authentication failed");
}

#[rocket::async_test]
async fn login_lockout_after_5_failed_attempts() {
    let app = setup_app().await.expect("Failed to initialize test app");

    for _ in 0..5 {
        let (status, _) = login(&app.client, ADMIN_USERNAME, "WrongPass#2026!").await;
        assert_eq!(status, Status::Unauthorized);
    }

    let (status, body) = login(&app.client, ADMIN_USERNAME, ADMIN_PASSWORD).await;
    assert_eq!(status, Status::Unauthorized);
    let body = body.expect("json body");
    assert_eq!(body["code"].as_u64(), Some(401));
    assert!(body["message"].as_str().unwrap_or_default().contains("locked"));
}

#[rocket::async_test]
async fn protected_without_token_returns_401() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let response = app.client.get("/api/v1/candidates").dispatch().await;
    assert_eq!(response.status(), Status::Unauthorized);
    let body = response.into_json::<serde_json::Value>().await.expect("json body");
    assert_eq!(body["code"], 401);
    assert!(body["message"].as_str().unwrap_or_default().contains("missing credentials"));
}

#[rocket::async_test]
async fn protected_with_invalid_token_returns_401() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let response = app
        .client
        .get("/api/v1/candidates")
        .header(rocket::http::Header::new("Authorization", "Bearer not-a-real-token"))
        .header(rocket::http::Header::new("x-session-id", "bad-session"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Unauthorized);
    let body = response.into_json::<serde_json::Value>().await.expect("json body");
    assert_eq!(body["code"], 401);
    assert!(body["message"].as_str().unwrap_or_default().contains("invalid"));
}

#[rocket::async_test]
async fn token_can_access_protected_route() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, ADMIN_USERNAME, ADMIN_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let body = body.expect("json");
    let headers = auth_headers(&body);
    let candidate_id = "cand-auth-token";

    common::factory_candidate_http(
        &app.client,
        &headers,
        candidate_id,
        "ID-AUTH-TOKEN-001",
        "BAR-AUTH-TOKEN-001",
    )
    .await;

    let response = common::attach_auth(app.client.get("/api/v1/candidates"), &headers)
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Ok);
    let rows = response
        .into_json::<Vec<serde_json::Value>>()
        .await
        .expect("candidate list");
    let created = rows
        .iter()
        .find(|row| row["id"] == candidate_id)
        .expect("created candidate should be visible");
    assert_eq!(created["national_id"], "ID-AUTH-TOKEN-001");
    assert_eq!(created["scanned_barcode"], "BAR-AUTH-TOKEN-001");
    assert!(created["metadata"].is_object(), "candidate payload must include metadata object");
}

#[rocket::async_test]
async fn jwt_only_can_access_protected_route() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, ADMIN_USERNAME, ADMIN_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let body = body.expect("json");
    let jwt = body["jwt"].as_str().expect("jwt");
    let headers = auth_headers(&body);

    common::factory_candidate_http(
        &app.client,
        &headers,
        "cand-auth-jwt",
        "ID-AUTH-JWT-001",
        "BAR-AUTH-JWT-001",
    )
    .await;

    let response = app
        .client
        .get("/api/v1/candidates")
        .header(rocket::http::Header::new("Authorization", format!("Bearer {jwt}")))
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Ok);
    let rows = response
        .into_json::<Vec<serde_json::Value>>()
        .await
        .expect("candidate list");
    assert!(
        rows.iter().any(|row| row["id"] == "cand-auth-jwt"),
        "jwt-only auth must authorize the same protected data path"
    );
    assert!(rows.iter().all(|row| row.get("id").is_some()));
}

#[rocket::async_test]
async fn session_only_can_access_protected_route() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, ADMIN_USERNAME, ADMIN_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let body = body.expect("json");
    let session_id = body["session_id"].as_str().expect("session").to_string();
    let headers = auth_headers(&body);

    common::factory_candidate_http(
        &app.client,
        &headers,
        "cand-auth-session",
        "ID-AUTH-SESSION-001",
        "BAR-AUTH-SESSION-001",
    )
    .await;

    let response = app
        .client
        .get("/api/v1/candidates")
        .header(rocket::http::Header::new("x-session-id", session_id))
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Ok);
    let rows = response
        .into_json::<Vec<serde_json::Value>>()
        .await
        .expect("candidate list");
    let created = rows
        .iter()
        .find(|row| row["id"] == "cand-auth-session")
        .expect("session-only auth must reach protected handler");
    assert_eq!(created["national_id"], "ID-AUTH-SESSION-001");
    assert!(created["created_at"].is_string());
}
