mod common;

use chrono::{Duration, Utc};
use rocket::http::Status;
use serde_json::Value;

use common::{
    attach_auth, factory_asset, factory_room, factory_session, login_as, setup_app, user_id_for,
    Role, COORD_USERNAME,
};

async fn seed_expiring_asset(pool: &sqlx::MySqlPool, owner_id: &str) {
    factory_session(pool, "sess-near-exp", "Template A", 60, owner_id).await;
    let expires = (Utc::now() + Duration::days(10)).date_naive();
    sqlx::query(
        "INSERT INTO assets (id, booklet_code, tracking_status, session_id, expires_on, incident_count, created_by)
         VALUES ('asset-near-exp', 'BOOK-NEAR-EXP', 'Prepared', 'sess-near-exp', ?, 2, ?)",
    )
    .bind(expires)
    .bind(owner_id)
    .execute(pool)
    .await
    .expect("seed near-expiry asset");
}

#[rocket::async_test]
async fn reports_dashboard_success_returns_counts_for_coordinator() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_room(&app.pool, "room-dash", 40, "Dashboard", &coord_id).await;
    seed_expiring_asset(&app.pool, &coord_id).await;

    let resp = attach_auth(app.client.get("/api/v1/reports/dashboard"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.expect("json");
    assert!(body["seat_utilization_count"].is_number());
    assert!(body["near_expiry_count"].as_u64().unwrap_or(0) >= 1);
    assert!(body["alert_count"].is_number());
}

#[rocket::async_test]
async fn dashboard_summary_success_for_admin() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Admin).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_room(&app.pool, "room-summary", 40, "Summary", &coord_id).await;
    factory_session(&app.pool, "sess-summary", "Template A", 60, &coord_id).await;

    let resp = attach_auth(app.client.get("/api/v1/dashboard/summary"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.expect("json");
    assert!(body["total_rooms"].as_i64().unwrap_or(0) >= 1);
    assert!(body["upcoming_sessions"].is_array());
    assert!(body["recent_outputs"].is_array());
}

#[rocket::async_test]
async fn dashboard_summary_forbidden_for_proctor() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Proctor).await;
    let resp = attach_auth(app.client.get("/api/v1/dashboard/summary"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Forbidden);
    let body: Value = resp.into_json().await.expect("json");
    assert_eq!(body["code"], 403);
    assert!(body["message"].as_str().unwrap_or_default().contains("report"));
}

#[rocket::async_test]
async fn near_expiry_alerts_returns_assets_close_to_expiration() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    seed_expiring_asset(&app.pool, &coord_id).await;

    let resp = attach_auth(
        app.client.get("/api/v1/operations/near-expiry-alerts?limit=50"),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("rows");
    assert!(rows.iter().any(|r| r["id"] == "asset-near-exp"));
    assert!(rows
        .iter()
        .find(|r| r["id"] == "asset-near-exp")
        .unwrap()["booklet_code"]
        == "BOOK-NEAR-EXP");
}

#[rocket::async_test]
async fn near_expiry_alerts_unauthenticated_returns_401() {
    let app = setup_app().await.expect("setup");
    let resp = app.client.get("/api/v1/operations/near-expiry-alerts").dispatch().await;
    assert_eq!(resp.status(), Status::Unauthorized);
    let body: Value = resp.into_json().await.expect("json");
    assert_eq!(body["code"], 401);
}

#[rocket::async_test]
async fn incident_rates_returns_data_for_reporting_roles() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Auditor).await;

    let resp = attach_auth(
        app.client.get("/api/v1/operations/incident-rates?limit=10"),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("json");
    for row in &rows {
        assert!(row.get("session_id").is_some(), "incident rate row missing session_id: {row}");
    }
}

#[rocket::async_test]
async fn incident_rates_fallback_returns_array() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Admin).await;

    let resp = attach_auth(
        app.client.get("/api/v1/operations/incident-rates"),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("json");
    // Fallback branch returns up to 50 rows — shape check only.
    assert!(rows.len() <= 50);
}

#[rocket::async_test]
async fn return_rates_success_for_coordinator() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_session(&app.pool, "sess-ret", "Template A", 60, &coord_id).await;
    factory_asset(&app.pool, "asset-ret-1", "BOOK-RET-1", "sess-ret", &coord_id).await;
    sqlx::query("UPDATE assets SET tracking_status = 'Collected' WHERE id = 'asset-ret-1'")
        .execute(&app.pool)
        .await
        .unwrap();

    let resp = attach_auth(
        app.client.get("/api/v1/operations/return-rates?limit=100"),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("rows");
    assert!(rows.iter().any(|r| r["session_id"] == "sess-ret"));
}

#[rocket::async_test]
async fn return_rates_forbidden_for_proctor() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Proctor).await;

    let resp = attach_auth(
        app.client.get("/api/v1/operations/return-rates"),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Forbidden);
    let body: Value = resp.into_json().await.expect("json");
    assert_eq!(body["code"], 403);
}

#[rocket::async_test]
async fn materials_inventory_returns_asset_rows() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_session(&app.pool, "sess-mat", "Template A", 60, &coord_id).await;
    factory_asset(&app.pool, "asset-mat", "BOOK-MAT", "sess-mat", &coord_id).await;

    let resp = attach_auth(
        app.client.get("/api/v1/operations/materials-inventory?limit=100"),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("rows");
    assert!(rows.iter().any(|r| r["asset_id"] == "asset-mat"));
}

#[rocket::async_test]
async fn operations_alerts_returns_ok_with_within_days() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let resp = attach_auth(
        app.client.get("/api/v1/operations/alerts?within_days=60&limit=50"),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("rows");
    for row in &rows {
        assert!(
            row.get("alert_type").is_some() && row.get("severity").is_some(),
            "alerts row missing structured fields: {row}"
        );
    }
}
