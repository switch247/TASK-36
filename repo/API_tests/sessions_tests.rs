mod common;

use rocket::http::Status;
use serde_json::{json, Value};

use common::{
    attach_auth, factory_session, login_as, setup_app, user_id_for, Role, COORD_USERNAME,
};

fn session_create_payload(id: &str) -> Value {
    json!({
        "id": id,
        "template_name": "base-template",
        "duration_minutes": 90,
        "status": "Scheduled",
        "starts_at": "03/27/2026 09:00 AM",
        "ends_at": "03/27/2026 10:30 AM"
    })
}

#[rocket::async_test]
async fn create_session_invalid_duration_returns_400() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let payload = json!({
        "id": "sess-dur-bad",
        "template_name": "base-template",
        "duration_minutes": 5,
        "status": "Scheduled",
        "starts_at": "03/27/2026 09:00 AM",
        "ends_at": "03/27/2026 10:30 AM"
    });
    let resp = attach_auth(app.client.post("/api/v1/sessions").json(&payload), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::BadRequest);
    let body: Value = resp.into_json().await.expect("error body");
    assert_eq!(body["code"], 400);
    assert_eq!(body["message"], "validation failed");
    assert_eq!(body["details"]["field"], "duration_minutes");
    assert!(body["details"]["message"].as_str().unwrap_or_default().contains("between 15 and 360"));
}

#[rocket::async_test]
async fn create_session_bad_datetime_returns_400() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let payload = json!({
        "id": "sess-badtime",
        "template_name": "base-template",
        "duration_minutes": 60,
        "status": "Scheduled",
        "starts_at": "2026-03-27T09:00:00Z",
        "ends_at": "2026-03-27T10:00:00Z"
    });
    let resp = attach_auth(app.client.post("/api/v1/sessions").json(&payload), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::BadRequest);
    let body: Value = resp.into_json().await.expect("error body");
    assert_eq!(body["code"], 400);
    assert!(body["message"].as_str().unwrap_or_default().contains("datetime"));
}

#[rocket::async_test]
async fn create_session_forbidden_for_proctor() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Proctor).await;
    let resp = attach_auth(
        app.client.post("/api/v1/sessions").json(&session_create_payload("sess-proc")),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Forbidden);
}

#[rocket::async_test]
async fn list_sessions_coordinator_sees_only_own() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_session(&app.pool, "sess-mine-1", "base-template", 45, &coord_id).await;

    let other_id = common::factory_user(
        &app.client,
        &app.pool,
        "coord_sessions_other",
        "StrongPass#2026!",
        "Coordinator",
    )
    .await;
    factory_session(&app.pool, "sess-other-1", "base-template", 45, &other_id).await;

    let resp = attach_auth(app.client.get("/api/v1/sessions?limit=100"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("rows");
    let ids: Vec<&str> = rows.iter().filter_map(|r| r["id"].as_str()).collect();
    assert!(ids.contains(&"sess-mine-1"));
    assert!(!ids.contains(&"sess-other-1"));
}

#[rocket::async_test]
async fn list_sessions_proctor_sees_assigned() {
    let app = setup_app().await.expect("setup");
    let coord_headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    let proctor_id = user_id_for(&app.pool, "proctor_local").await;
    factory_session(&app.pool, "sess-assign-me", "base-template", 45, &coord_id).await;
    factory_session(&app.pool, "sess-unassigned", "base-template", 45, &coord_id).await;

    let assign_resp = attach_auth(
        app.client
            .post("/api/v1/sessions/sess-assign-me/assignments")
            .json(&json!({ "user_id": proctor_id })),
        &coord_headers,
    )
    .dispatch()
    .await;
    assert_eq!(assign_resp.status(), Status::Created);

    let proctor_headers = login_as(&app.client, Role::Proctor).await;
    let resp = attach_auth(app.client.get("/api/v1/sessions?limit=100"), &proctor_headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("rows");
    let ids: Vec<&str> = rows.iter().filter_map(|r| r["id"].as_str()).collect();
    assert!(ids.contains(&"sess-assign-me"));
    assert!(!ids.contains(&"sess-unassigned"), "proctor must not see unassigned sessions");
}

#[rocket::async_test]
async fn list_sessions_forbidden_for_auditor() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Auditor).await;
    let resp = attach_auth(app.client.get("/api/v1/sessions"), &headers).dispatch().await;
    assert_eq!(resp.status(), Status::Forbidden);
}

#[rocket::async_test]
async fn update_session_by_coordinator_persists() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_session(&app.pool, "sess-upd", "base-template", 60, &coord_id).await;

    let payload = json!({
        "id": "sess-upd",
        "template_name": "base-template",
        "duration_minutes": 120,
        "status": "Active",
        "starts_at": "03/27/2026 09:00 AM",
        "ends_at": "03/27/2026 11:00 AM"
    });
    let resp = attach_auth(app.client.put("/api/v1/sessions/sess-upd").json(&payload), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);

    let (dur, status): (i32, String) =
        sqlx::query_as("SELECT duration_minutes, status FROM exam_sessions WHERE id = ?")
            .bind("sess-upd")
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(dur, 120);
    assert_eq!(status, "Active");
}

#[rocket::async_test]
async fn update_session_not_found_returns_404() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let resp = attach_auth(
        app.client.put("/api/v1/sessions/ghost").json(&session_create_payload("ghost")),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::NotFound);
}

#[rocket::async_test]
async fn delete_session_by_owner_returns_204() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_session(&app.pool, "sess-del", "base-template", 60, &coord_id).await;

    let resp = attach_auth(app.client.delete("/api/v1/sessions/sess-del"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::NoContent);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM exam_sessions WHERE id = ?")
        .bind("sess-del")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[rocket::async_test]
async fn delete_session_not_found_returns_404() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let resp = attach_auth(app.client.delete("/api/v1/sessions/ghost"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::NotFound);
}

#[rocket::async_test]
async fn delete_session_foreign_ownership_returns_404() {
    let app = setup_app().await.expect("setup");
    let other_id = common::factory_user(
        &app.client,
        &app.pool,
        "coord_session_foreign",
        "StrongPass#2026!",
        "Coordinator",
    )
    .await;
    factory_session(&app.pool, "sess-foreign", "base-template", 60, &other_id).await;

    let headers = login_as(&app.client, Role::Coordinator).await;
    let resp = attach_auth(app.client.delete("/api/v1/sessions/sess-foreign"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::NotFound);
}

#[rocket::async_test]
async fn assign_session_requires_proctor_user_as_assignee() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_session(&app.pool, "sess-assign-bad", "base-template", 60, &coord_id).await;

    // Assigning a coordinator (not a proctor) should be a 400.
    let resp = attach_auth(
        app.client
            .post("/api/v1/sessions/sess-assign-bad/assignments")
            .json(&json!({ "user_id": coord_id })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::BadRequest);
    let body: Value = resp.into_json().await.expect("error body");
    assert_eq!(body["code"], 400);
    assert_eq!(body["message"], "assignee must be a Proctor");
}
