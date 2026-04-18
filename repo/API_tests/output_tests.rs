mod common;

use rocket::http::Status;
use serde_json::json;

use common::{
    attach_auth, auth_headers, login, setup_app, COORD_PASSWORD, COORD_USERNAME, PROCTOR_PASSWORD,
    PROCTOR_USERNAME,
};

#[rocket::async_test]
async fn final_print_allowed_for_proctor_and_coordinator() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let coord_id: String = sqlx::query_scalar("SELECT id FROM users WHERE username = ?")
        .bind(COORD_USERNAME)
        .fetch_one(&app.pool)
        .await
        .expect("coord user id");

    let session_id = "sess-final-lock-test";
    sqlx::query(
        "INSERT INTO exam_sessions (id, template_name, duration_minutes, status, starts_at, ends_at, locked_for_final_print, created_by)
         VALUES (?, 'Template A', 60, 'Scheduled', UTC_TIMESTAMP(), DATE_ADD(UTC_TIMESTAMP(), INTERVAL 60 MINUTE), FALSE, ?)",
    )
    .bind(session_id)
    .bind(&coord_id)
    .execute(&app.pool)
    .await
    .expect("insert session");

    let (status, proctor_login) = login(&app.client, PROCTOR_USERNAME, PROCTOR_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let proctor_body = proctor_login.expect("proctor login body");
    let proctor_headers = auth_headers(&proctor_body);

    let proctor_id: String = sqlx::query_scalar("SELECT id FROM users WHERE username = ?")
        .bind(PROCTOR_USERNAME)
        .fetch_one(&app.pool)
        .await
        .expect("proctor user id");

    let assign_response = attach_auth(
        app.client
            .post(format!("/api/v1/sessions/{session_id}/assignments"))
            .json(&json!({ "user_id": proctor_id })),
        &proctor_headers,
    )
    .dispatch()
    .await;
    assert_eq!(assign_response.status(), Status::Forbidden);

    let response = attach_auth(app.client.post("/api/v1/outputs"), &proctor_headers)
        .json(&json!({
            "session_id": session_id,
            "output_type": "AdmitCard",
            "mode": "FinalPrint"
        }))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Forbidden);

    let (status2, coord_login) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status2, Status::Ok);
    let coord_body = coord_login.expect("coord login body");
    let coord_headers = auth_headers(&coord_body);

    let assign_by_coord = attach_auth(
        app.client
            .post(format!("/api/v1/sessions/{session_id}/assignments"))
            .json(&json!({ "user_id": proctor_id })),
        &coord_headers,
    )
    .dispatch()
    .await;
    assert_eq!(assign_by_coord.status(), Status::Created);

    let proctor_retry = attach_auth(app.client.post("/api/v1/outputs"), &proctor_headers)
        .json(&json!({
            "session_id": session_id,
            "output_type": "AdmitCard",
            "mode": "FinalPrint"
        }))
        .dispatch()
        .await;
    assert_eq!(proctor_retry.status(), Status::Ok);

    let ok_response = attach_auth(app.client.post("/api/v1/outputs"), &coord_headers)
        .json(&json!({
            "session_id": session_id,
            "output_type": "AdmitCard",
            "mode": "FinalPrint"
        }))
        .dispatch()
        .await;

    assert_eq!(ok_response.status(), Status::Ok);

    let output_row: Option<(String, i32)> = sqlx::query_as(
        "SELECT template_id, template_version_no FROM print_outputs WHERE session_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(session_id)
    .fetch_optional(&app.pool)
    .await
    .expect("print output row");
    assert_eq!(output_row, Some(("Template A".to_string(), 1)));

    let template_locked: Option<i64> = sqlx::query_scalar(
        "SELECT locked_for_final_print FROM template_versions WHERE template_id = 'Template A' AND version_no = 1",
    )
    .fetch_optional(&app.pool)
    .await
    .expect("template lock state");
    assert_eq!(template_locked, Some(1));

    let update_resp = attach_auth(
        app.client
            .put("/api/v1/templates/Template A/1")
            .json(&json!({"snapshot": {"admit_card": {"title": "Mutated"}}, "lock_for_final_print": true})),
        &coord_headers,
    )
    .dispatch()
    .await;
    assert_eq!(update_resp.status(), Status::Conflict);

    let delete_resp = attach_auth(
        app.client.delete("/api/v1/templates/Template A/1"),
        &coord_headers,
    )
    .dispatch()
    .await;
    assert_eq!(delete_resp.status(), Status::Conflict);
}

#[rocket::async_test]
async fn unassigned_proctor_cannot_print_foreign_session() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let coord_id: String = sqlx::query_scalar("SELECT id FROM users WHERE username = ?")
        .bind(COORD_USERNAME)
        .fetch_one(&app.pool)
        .await
        .expect("coord user id");

    let session_id = "sess-unassigned-proctor-test";
    sqlx::query(
        "INSERT INTO exam_sessions (id, template_name, duration_minutes, status, starts_at, ends_at, locked_for_final_print, created_by)
         VALUES (?, 'Template A', 60, 'Scheduled', UTC_TIMESTAMP(), DATE_ADD(UTC_TIMESTAMP(), INTERVAL 60 MINUTE), FALSE, ?)",
    )
    .bind(session_id)
    .bind(&coord_id)
    .execute(&app.pool)
    .await
    .expect("insert session");

    let (status, proctor_login) = login(&app.client, PROCTOR_USERNAME, PROCTOR_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let proctor_body = proctor_login.expect("proctor login body");
    let proctor_headers = auth_headers(&proctor_body);

    let response = attach_auth(app.client.post("/api/v1/outputs"), &proctor_headers)
        .json(&json!({
            "session_id": session_id,
            "output_type": "AdmitCard",
            "mode": "FinalPrint"
        }))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Forbidden);
}

#[rocket::async_test]
async fn export_csv_writes_audit_log() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, admin_login) =
        login(&app.client, common::ADMIN_USERNAME, common::ADMIN_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let admin_body = admin_login.expect("admin login body");
    let admin_headers = auth_headers(&admin_body);

    let response = attach_auth(app.client.post("/api/v1/exports/csv"), &admin_headers)
        .json(&json!({
            "report": "incident_rates",
            "limit": 100
        }))
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Ok);

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_logs WHERE action = 'export_data' AND resource = '/api/v1/exports/csv'",
    )
    .fetch_one(&app.pool)
    .await
    .expect("count audit log");

    assert!(count >= 1);
}
