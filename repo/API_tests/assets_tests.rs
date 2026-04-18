mod common;

use rocket::http::Status;
use serde_json::{json, Value};

use common::{
    attach_auth, factory_asset, factory_session, login_as, setup_app, user_id_for, Role,
    COORD_USERNAME,
};

fn asset_payload(id: &str, booklet: &str, session_id: &str) -> Value {
    json!({
        "id": id,
        "booklet_code": booklet,
        "tracking_status": "Prepared",
        "session_id": session_id,
        "expires_on": "2030-12-31",
        "incident_count": 0
    })
}

#[rocket::async_test]
async fn create_asset_by_coordinator_persists() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_session(&app.pool, "sess-asset-1", "Template A", 60, &coord_id).await;

    let resp = attach_auth(
        app.client
            .post("/api/v1/assets")
            .json(&asset_payload("asset-create-1", "BOOK-CREATE-1", "sess-asset-1")),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Created);

    let tracking: String = sqlx::query_scalar("SELECT tracking_status FROM assets WHERE id = ?")
        .bind("asset-create-1")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(tracking, "Prepared");
}

#[rocket::async_test]
async fn create_asset_bad_date_returns_400() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_session(&app.pool, "sess-asset-2", "Template A", 60, &coord_id).await;

    let payload = json!({
        "id": "asset-bad-date",
        "booklet_code": "BOOK-BAD-DATE",
        "tracking_status": "Prepared",
        "session_id": "sess-asset-2",
        "expires_on": "not-a-date",
        "incident_count": 0
    });
    let resp = attach_auth(app.client.post("/api/v1/assets").json(&payload), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::BadRequest);
    let body: Value = resp.into_json().await.expect("error body");
    assert_eq!(body["code"], 400);
    assert!(body["message"].as_str().unwrap_or_default().contains("expires_on"));
}

#[rocket::async_test]
async fn create_asset_forbidden_for_proctor() {
    let app = setup_app().await.expect("setup");
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_session(&app.pool, "sess-asset-3", "Template A", 60, &coord_id).await;

    let headers = login_as(&app.client, Role::Proctor).await;
    let resp = attach_auth(
        app.client
            .post("/api/v1/assets")
            .json(&asset_payload("asset-forbidden", "BOOK-FORBID", "sess-asset-3")),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Forbidden);
}

#[rocket::async_test]
async fn create_asset_duplicate_booklet_code_returns_409() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_session(&app.pool, "sess-asset-dup", "Template A", 60, &coord_id).await;

    let first = attach_auth(
        app.client
            .post("/api/v1/assets")
            .json(&asset_payload("asset-one", "BOOK-DUP-A", "sess-asset-dup")),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(first.status(), Status::Created);

    let second = attach_auth(
        app.client
            .post("/api/v1/assets")
            .json(&asset_payload("asset-two", "BOOK-DUP-A", "sess-asset-dup")),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(second.status(), Status::Conflict, "unique booklet_code must trigger 409");
    let body: Value = second.into_json().await.expect("error body");
    assert_eq!(body["code"], 409);
    assert_eq!(body["message"], "asset already exists");
}

#[rocket::async_test]
async fn list_assets_restricted_by_ownership() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_session(&app.pool, "sess-assets-list", "Template A", 60, &coord_id).await;
    factory_asset(&app.pool, "asset-mine", "BOOK-MINE", "sess-assets-list", &coord_id).await;

    // Another coordinator + their own asset
    let other_id = common::factory_user(
        &app.client,
        &app.pool,
        "coord_assets_other",
        "StrongPass#2026!",
        "Coordinator",
    )
    .await;
    factory_session(&app.pool, "sess-assets-other", "Template A", 60, &other_id).await;
    factory_asset(&app.pool, "asset-theirs", "BOOK-THEIRS", "sess-assets-other", &other_id).await;

    let resp = attach_auth(app.client.get("/api/v1/assets?limit=100"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("rows");
    let ids: Vec<&str> = rows.iter().filter_map(|r| r["id"].as_str()).collect();
    assert!(ids.contains(&"asset-mine"));
    assert!(!ids.contains(&"asset-theirs"));
}

#[rocket::async_test]
async fn list_assets_admin_sees_all() {
    let app = setup_app().await.expect("setup");
    let admin_headers = login_as(&app.client, Role::Admin).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_session(&app.pool, "sess-assets-admin", "Template A", 60, &coord_id).await;
    factory_asset(&app.pool, "asset-seen-by-admin", "BOOK-ADMIN", "sess-assets-admin", &coord_id).await;

    let resp = attach_auth(app.client.get("/api/v1/assets?limit=200"), &admin_headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("rows");
    assert!(rows.iter().any(|r| r["id"] == "asset-seen-by-admin"));
}

#[rocket::async_test]
async fn update_asset_by_owner_changes_tracking_status() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_session(&app.pool, "sess-asset-upd", "Template A", 60, &coord_id).await;
    factory_asset(&app.pool, "asset-upd", "BOOK-UPD", "sess-asset-upd", &coord_id).await;

    let payload = json!({
        "id": "asset-upd",
        "booklet_code": "BOOK-UPD",
        "tracking_status": "Delivered",
        "session_id": "sess-asset-upd",
        "expires_on": "2031-06-30",
        "incident_count": 2
    });
    let resp = attach_auth(
        app.client.put("/api/v1/assets/asset-upd").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);

    let (status, incidents): (String, i32) =
        sqlx::query_as("SELECT tracking_status, incident_count FROM assets WHERE id = ?")
            .bind("asset-upd")
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(status, "Delivered");
    assert_eq!(incidents, 2);
}

#[rocket::async_test]
async fn update_asset_not_found_returns_404() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_session(&app.pool, "sess-asset-nf", "Template A", 60, &coord_id).await;

    let payload = asset_payload("missing-asset", "BOOK-MISS", "sess-asset-nf");
    let resp = attach_auth(
        app.client.put("/api/v1/assets/missing-asset").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::NotFound);
    let body: Value = resp.into_json().await.expect("error body");
    assert_eq!(body["code"], 404);
    assert!(body["message"].as_str().unwrap_or_default().contains("asset"));
}

#[rocket::async_test]
async fn delete_asset_by_owner_returns_204() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_session(&app.pool, "sess-asset-del", "Template A", 60, &coord_id).await;
    factory_asset(&app.pool, "asset-del", "BOOK-DEL", "sess-asset-del", &coord_id).await;

    let resp = attach_auth(app.client.delete("/api/v1/assets/asset-del"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::NoContent);

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM assets WHERE id = ?")
        .bind("asset-del")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(remaining, 0);
}

#[rocket::async_test]
async fn delete_asset_forbidden_for_proctor() {
    let app = setup_app().await.expect("setup");
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_session(&app.pool, "sess-asset-proctor", "Template A", 60, &coord_id).await;
    factory_asset(&app.pool, "asset-proc", "BOOK-PROC", "sess-asset-proctor", &coord_id).await;

    let proctor_headers = login_as(&app.client, Role::Proctor).await;
    let resp = attach_auth(app.client.delete("/api/v1/assets/asset-proc"), &proctor_headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Forbidden);
}

#[rocket::async_test]
async fn delete_asset_foreign_ownership_returns_404() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let other_id = common::factory_user(
        &app.client,
        &app.pool,
        "coord_assets_foreign",
        "StrongPass#2026!",
        "Coordinator",
    )
    .await;
    factory_session(&app.pool, "sess-assets-foreign", "Template A", 60, &other_id).await;
    factory_asset(&app.pool, "asset-foreign", "BOOK-FOREIGN", "sess-assets-foreign", &other_id).await;

    let resp = attach_auth(app.client.delete("/api/v1/assets/asset-foreign"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::NotFound);
}
