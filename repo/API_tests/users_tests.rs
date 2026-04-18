mod common;

use rocket::http::Status;
use serde_json::{json, Value};

use common::{attach_auth, login_as, setup_app, Role};

#[rocket::async_test]
async fn admin_can_create_user_and_record_is_persisted() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Admin).await;

    let payload = json!({
        "username": "coord_created",
        "password": "StrongPass#2026!",
        "role": "coordinator"
    });
    let resp = attach_auth(app.client.post("/api/v1/users").json(&payload), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Created);

    let role: Option<String> = sqlx::query_scalar("SELECT role FROM users WHERE username = ?")
        .bind("coord_created")
        .fetch_optional(&app.pool)
        .await
        .expect("query");
    assert_eq!(
        role.as_deref(),
        Some("Coordinator"),
        "role must be normalized"
    );
}

#[rocket::async_test]
async fn create_user_with_invalid_role_returns_400() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Admin).await;

    let payload = json!({
        "username": "bad_role_user",
        "password": "StrongPass#2026!",
        "role": "superadmin"
    });
    let resp = attach_auth(app.client.post("/api/v1/users").json(&payload), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::BadRequest);
    let body: Value = resp.into_json().await.expect("error body");
    assert_eq!(body["code"], 400);
    assert!(body["message"]
        .as_str()
        .unwrap_or_default()
        .contains("role"));
}

#[rocket::async_test]
async fn create_user_weak_password_returns_400() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Admin).await;

    let payload = json!({
        "username": "weak_pass_user",
        "password": "abc",
        "role": "Proctor"
    });
    let resp = attach_auth(app.client.post("/api/v1/users").json(&payload), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::BadRequest);
    let body: Value = resp.into_json().await.expect("error body");
    assert_eq!(body["code"], 400);
    assert!(body["message"]
        .as_str()
        .unwrap_or_default()
        .contains("password"));
}

#[rocket::async_test]
async fn create_user_duplicate_username_returns_409() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Admin).await;

    let payload = json!({
        "username": "admin_local", // already seeded
        "password": "StrongPass#2026!",
        "role": "coordinator"
    });
    let resp = attach_auth(app.client.post("/api/v1/users").json(&payload), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Conflict);
    let body: Value = resp.into_json().await.expect("error body");
    assert_eq!(body["code"], 409);
    assert!(body["message"]
        .as_str()
        .unwrap_or_default()
        .contains("username"));
}

#[rocket::async_test]
async fn non_admin_cannot_create_user_returns_403() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let payload = json!({
        "username": "should_not_exist",
        "password": "StrongPass#2026!",
        "role": "proctor"
    });
    let resp = attach_auth(app.client.post("/api/v1/users").json(&payload), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Forbidden);

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE username = 'should_not_exist'")
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(count, 0, "forbidden create must not persist");
}

#[rocket::async_test]
async fn list_users_requires_auth() {
    let app = setup_app().await.expect("setup");
    let resp = app.client.get("/api/v1/users").dispatch().await;
    assert_eq!(resp.status(), Status::Unauthorized);
    let body: Value = resp.into_json().await.expect("error body");
    assert_eq!(body["code"], 401);
    assert!(
        body["message"]
            .as_str()
            .unwrap_or_default()
            .contains("missing credentials"),
        "unauthenticated user listing must explain the auth requirement"
    );
    assert!(
        body.get("jwt").is_none(),
        "error payload must not look like a login payload"
    );
}

#[rocket::async_test]
async fn list_users_returns_seeded_users_for_admin() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Admin).await;

    let resp = attach_auth(app.client.get("/api/v1/users"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("json");
    let usernames: Vec<&str> = rows.iter().filter_map(|r| r["username"].as_str()).collect();
    assert!(usernames.contains(&"admin_local"));
    assert!(usernames.contains(&"coord_local"));
    assert!(usernames.contains(&"proctor_local"));
    assert!(usernames.contains(&"auditor_local"));

    let admin_row = rows
        .iter()
        .find(|r| r["username"] == "admin_local")
        .unwrap();
    assert_eq!(admin_row["role"], "Admin");
    assert!(admin_row["id"].is_string());
    assert!(admin_row["failed_login_attempts"].as_i64().is_some());
}

#[rocket::async_test]
async fn list_users_forbidden_for_coordinator() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let resp = attach_auth(app.client.get("/api/v1/users"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Forbidden);
}

#[rocket::async_test]
async fn update_user_role_by_admin_persists_change() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Admin).await;

    let proctor_id: String =
        sqlx::query_scalar("SELECT id FROM users WHERE username = 'proctor_local'")
            .fetch_one(&app.pool)
            .await
            .expect("proctor id");

    let resp = attach_auth(
        app.client
            .put(format!("/api/v1/users/{proctor_id}"))
            .json(&json!({ "role": "coordinator" })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);

    let role: String = sqlx::query_scalar("SELECT role FROM users WHERE id = ?")
        .bind(&proctor_id)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(role, "Coordinator");
}

#[rocket::async_test]
async fn update_user_password_by_admin_allows_new_login() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Admin).await;

    let proctor_id: String =
        sqlx::query_scalar("SELECT id FROM users WHERE username = 'proctor_local'")
            .fetch_one(&app.pool)
            .await
            .expect("proctor id");

    let resp = attach_auth(
        app.client
            .put(format!("/api/v1/users/{proctor_id}"))
            .json(&json!({ "password": "Rotated#Pass2026!" })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);

    let (status, _) = common::login(&app.client, "proctor_local", "Rotated#Pass2026!").await;
    assert_eq!(status, Status::Ok, "rotated password must authenticate");
}

#[rocket::async_test]
async fn update_user_not_found_returns_404() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Admin).await;

    let resp = attach_auth(
        app.client
            .put("/api/v1/users/does-not-exist-id")
            .json(&json!({ "password": "Ignored#Pass2026!" })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::NotFound);
    let body: Value = resp.into_json().await.expect("error body");
    assert_eq!(body["code"], 404);
    assert!(body["message"]
        .as_str()
        .unwrap_or_default()
        .contains("user"));
}

#[rocket::async_test]
async fn update_user_forbidden_for_non_admin() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let target_id: String =
        sqlx::query_scalar("SELECT id FROM users WHERE username = 'proctor_local'")
            .fetch_one(&app.pool)
            .await
            .unwrap();

    let resp = attach_auth(
        app.client
            .put(format!("/api/v1/users/{target_id}"))
            .json(&json!({ "password": "Rotate#Fail2026!" })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Forbidden);
}

#[rocket::async_test]
async fn delete_user_by_admin_removes_row() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Admin).await;

    let user_id = common::factory_user(
        &app.client,
        &app.pool,
        "to_be_removed",
        "StrongPass#2026!",
        "Auditor",
    )
    .await;

    let resp = attach_auth(
        app.client.delete(format!("/api/v1/users/{user_id}")),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::NoContent);

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(remaining, 0);
}

#[rocket::async_test]
async fn delete_user_not_found_returns_404() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Admin).await;

    let resp = attach_auth(app.client.delete("/api/v1/users/no-such-user"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::NotFound);
    let body: Value = resp.into_json().await.expect("error body");
    assert_eq!(body["code"], 404);
    assert!(body["message"]
        .as_str()
        .unwrap_or_default()
        .contains("user"));
}

#[rocket::async_test]
async fn delete_user_forbidden_for_non_admin() {
    let app = setup_app().await.expect("setup");
    let coord_headers = login_as(&app.client, Role::Coordinator).await;

    let user_id = common::factory_user(
        &app.client,
        &app.pool,
        "protected_user",
        "StrongPass#2026!",
        "Auditor",
    )
    .await;

    let resp = attach_auth(
        app.client.delete(format!("/api/v1/users/{user_id}")),
        &coord_headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Forbidden);

    let still_there: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(still_there, 1);
}
