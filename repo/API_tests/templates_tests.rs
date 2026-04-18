mod common;

use rocket::http::Status;
use serde_json::{json, Value};

use common::{attach_auth, login_as, setup_app, Role};

#[rocket::async_test]
async fn list_templates_returns_seeded_and_requires_admin_or_coord() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let resp = attach_auth(app.client.get("/api/v1/templates"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("rows");
    let template_ids: Vec<&str> = rows
        .iter()
        .filter_map(|r| r["template_id"].as_str())
        .collect();
    assert!(template_ids.contains(&"base-template"));
    assert!(template_ids.contains(&"Template A"));
}

#[rocket::async_test]
async fn list_templates_forbidden_for_proctor() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Proctor).await;
    let resp = attach_auth(app.client.get("/api/v1/templates"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Forbidden);
}

#[rocket::async_test]
async fn list_templates_unauthenticated_returns_401() {
    let app = setup_app().await.expect("setup");
    let resp = app.client.get("/api/v1/templates").dispatch().await;
    assert_eq!(resp.status(), Status::Unauthorized);
}

#[rocket::async_test]
async fn create_template_new_version_persists() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let payload = json!({
        "template_id": "bespoke-template",
        "version_no": 1,
        "snapshot": {
            "rules": { "id": ["Required"] },
            "summary_report": { "title": "Bespoke" }
        },
        "lock_for_final_print": false
    });
    let resp = attach_auth(
        app.client.post("/api/v1/templates").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Created);

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM template_versions WHERE template_id = 'bespoke-template' AND version_no = 1",
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert_eq!(count, 1);
}

#[rocket::async_test]
async fn create_template_duplicate_version_returns_409() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    // base-template v1 is seeded.
    let payload = json!({
        "template_id": "base-template",
        "version_no": 1,
        "snapshot": {"rules":{"id":["Required"]}},
        "lock_for_final_print": false
    });
    let resp = attach_auth(
        app.client.post("/api/v1/templates").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Conflict);
    let body: Value = resp.into_json().await.expect("error body");
    assert_eq!(body["code"], 409);
    assert!(body["message"]
        .as_str()
        .unwrap_or_default()
        .contains("template"));
}

#[rocket::async_test]
async fn create_template_forbidden_for_proctor() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Proctor).await;

    let payload = json!({
        "template_id": "proctor-bespoke",
        "version_no": 1,
        "snapshot": {},
        "lock_for_final_print": false
    });
    let resp = attach_auth(
        app.client.post("/api/v1/templates").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Forbidden);
}

#[rocket::async_test]
async fn lock_template_creates_a_version() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let payload = json!({
        "version_no": 2,
        "snapshot": {"rules":{"id":["Required"]}, "summary_report": {"title": "Locked"}},
        "lock_for_final_print": true
    });
    let resp = attach_auth(
        app.client
            .post("/api/v1/templates/base-template/lock")
            .json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Created);

    let locked: Option<bool> = sqlx::query_scalar(
        "SELECT locked_for_final_print FROM template_versions WHERE template_id = 'base-template' AND version_no = 2",
    )
    .fetch_optional(&app.pool)
    .await
    .unwrap();
    assert_eq!(locked, Some(true));
}

#[rocket::async_test]
async fn lock_template_forbidden_for_auditor() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Auditor).await;

    let payload = json!({
        "version_no": 3,
        "snapshot": {"rules":{"id":["Required"]}},
        "lock_for_final_print": true
    });
    let resp = attach_auth(
        app.client
            .post("/api/v1/templates/base-template/lock")
            .json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Forbidden);
}

#[rocket::async_test]
async fn update_template_success_path_mutates_unlocked_version() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let create_payload = json!({
        "template_id": "mutable-template",
        "version_no": 1,
        "snapshot": {"rules": {"id": ["Required"]}, "summary_report": {"title": "Original"}},
        "lock_for_final_print": false
    });
    let created = attach_auth(
        app.client.post("/api/v1/templates").json(&create_payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(created.status(), Status::Created);

    let update_payload = json!({
        "snapshot": {"rules":{"id":["Required"]}, "summary_report": {"title": "Mutated"}},
        "lock_for_final_print": false
    });
    let resp = attach_auth(
        app.client
            .put("/api/v1/templates/mutable-template/1")
            .json(&update_payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);

    let snapshot: serde_json::Value = sqlx::query_scalar(
        "SELECT snapshot FROM template_versions WHERE template_id = 'mutable-template' AND version_no = 1",
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert_eq!(snapshot["summary_report"]["title"], "Mutated");
}

#[rocket::async_test]
async fn update_template_missing_version_returns_404() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let payload = json!({
        "snapshot": {"rules":{"id":["Required"]}},
        "lock_for_final_print": false
    });
    let resp = attach_auth(
        app.client
            .put("/api/v1/templates/ghost-template/99")
            .json(&payload),
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
        .contains("template"));
}

#[rocket::async_test]
async fn delete_template_success_removes_unlocked_version() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let create_payload = json!({
        "template_id": "delete-me",
        "version_no": 7,
        "snapshot": {"rules":{"id":["Required"]}},
        "lock_for_final_print": false
    });
    attach_auth(
        app.client.post("/api/v1/templates").json(&create_payload),
        &headers,
    )
    .dispatch()
    .await;

    let resp = attach_auth(app.client.delete("/api/v1/templates/delete-me/7"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::NoContent);

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM template_versions WHERE template_id = 'delete-me' AND version_no = 7",
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert_eq!(count, 0);
}

#[rocket::async_test]
async fn delete_template_forbidden_for_proctor() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Proctor).await;

    let resp = attach_auth(
        app.client.delete("/api/v1/templates/base-template/1"),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Forbidden);
}
