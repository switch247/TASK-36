mod common;

use rocket::http::Status;
use serde_json::{json, Value};

use common::{attach_auth, login_as, setup_app, Role};

#[rocket::async_test]
async fn export_excel_returns_tsv_with_expected_header() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Admin).await;

    let resp = attach_auth(
        app.client.post("/api/v1/exports/excel").json(&json!({
            "report": "incident_rates",
            "limit": 50
        })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.expect("json");
    let content = body["content"].as_str().unwrap_or_default();
    assert!(content.contains("session_id"), "excel content must include column header");

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_logs WHERE action = 'export_data' AND resource = '/api/v1/exports/excel'",
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert!(audit_count >= 1, "excel export must be audit-logged");
}

#[rocket::async_test]
async fn export_pdf_returns_placeholder_with_title() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Admin).await;

    let resp = attach_auth(
        app.client.post("/api/v1/exports/pdf").json(&json!({
            "report": "return_rates",
            "limit": 10
        })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.expect("json");
    let content = body["content"].as_str().unwrap_or_default();
    assert!(
        content.to_ascii_lowercase().contains("return_rates")
            || content.to_ascii_lowercase().contains("return rates"),
        "pdf output should echo the requested report title: {content}"
    );
}

#[rocket::async_test]
async fn export_csv_unsupported_report_returns_400() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Admin).await;

    let resp = attach_auth(
        app.client.post("/api/v1/exports/csv").json(&json!({
            "report": "unknown_report_type",
            "limit": 50
        })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::BadRequest);
}

#[rocket::async_test]
async fn export_excel_forbidden_for_proctor() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Proctor).await;

    let resp = attach_auth(
        app.client.post("/api/v1/exports/excel").json(&json!({
            "report": "incident_rates",
            "limit": 10
        })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Forbidden);
}

#[rocket::async_test]
async fn export_pdf_unauthenticated_returns_401() {
    let app = setup_app().await.expect("setup");
    let resp = app
        .client
        .post("/api/v1/exports/pdf")
        .json(&json!({"report": "incident_rates"}))
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Unauthorized);
}
