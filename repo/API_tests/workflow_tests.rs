mod common;

use base64::Engine;
use rocket::http::Status;
use serde_json::{json, Value};

use common::{auth_headers, login, setup_app, COORD_PASSWORD, COORD_USERNAME};

#[rocket::async_test]
async fn workflow_create_candidate_session_output_export() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));

    let create_candidate = json!({
        "candidate_id": "cand-flow",
        "date_of_birth": "03/27/2001",
        "national_id": "IDFLOW123",
        "scanned_barcode": "BAR-FLOW",
        "metadata_json": "{\"name\":\"Cand Flow\",\"room_id\":\"room-x\"}"
    });
    let response = common::attach_auth(
        app.client
            .post("/api/v1/candidates")
            .json(&create_candidate),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(response.status(), Status::Created);

    let create_session = json!({
        "id": "sess-flow",
        "template_name": "base-template",
        "duration_minutes": 90,
        "status": "Scheduled",
        "starts_at": "03/27/2026 09:00 AM",
        "ends_at": "03/27/2026 10:30 AM"
    });
    let response = common::attach_auth(
        app.client.post("/api/v1/sessions").json(&create_session),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(response.status(), Status::Created);

    let output_request = json!({
        "session_id": "sess-flow",
        "mode": "FinalPrint",
        "output_type": "AdmitCard"
    });
    let output_response = common::attach_auth(
        app.client.post("/api/v1/outputs").json(&output_request),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(output_response.status(), Status::Ok);
    let output_body: Value = output_response.into_json().await.expect("output json");
    assert_eq!(output_body["output_type"], "AdmitCard");
    assert_eq!(output_body["mode"], "FinalPrint");

    let export_request = json!({
        "report": "incident_rates",
        "limit": 100
    });
    let export_response = common::attach_auth(
        app.client.post("/api/v1/exports/csv").json(&export_request),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(export_response.status(), Status::Ok);
    let body: Value = export_response.into_json().await.expect("export json");
    assert!(body["content"]
        .as_str()
        .unwrap_or_default()
        .contains("session_id"));

    let locked = sqlx::query_scalar::<_, i64>(
        "SELECT locked_for_final_print FROM exam_sessions WHERE id = 'sess-flow'",
    )
    .fetch_optional(&app.pool)
    .await
    .expect("lock query");
    assert_eq!(locked, Some(1));
}

#[rocket::async_test]
async fn reports_pagination_filtering_and_sorting_boundaries() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));
    let coord_id: String = sqlx::query_scalar("SELECT id FROM users WHERE username = ?")
        .bind(COORD_USERNAME)
        .fetch_one(&app.pool)
        .await
        .expect("coord id");

    sqlx::query("INSERT INTO rooms (id, capacity, location, created_by) VALUES (?, ?, ?, ?)")
        .bind("room-order-a")
        .bind(50)
        .bind("Alpha")
        .bind(&coord_id)
        .execute(&app.pool)
        .await
        .ok();
    sqlx::query("INSERT INTO rooms (id, capacity, location, created_by) VALUES (?, ?, ?, ?)")
        .bind("room-order-b")
        .bind(200)
        .bind("Zulu")
        .bind(&coord_id)
        .execute(&app.pool)
        .await
        .ok();

    let response = common::attach_auth(
        app.client.get(
            "/api/v1/operations/seat-utilization?page=1&limit=10&sort_by=capacity&sort_order=asc",
        ),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(response.status(), Status::Ok);
    let rows: Vec<Value> = response.into_json().await.expect("rows");
    if rows.len() >= 2 {
        let first = rows[0]["capacity"].as_i64().unwrap_or(0);
        let second = rows[1]["capacity"].as_i64().unwrap_or(0);
        assert!(first <= second);
    }

    let response = common::attach_auth(
        app.client
            .get("/api/v1/candidates?page=0&limit=999&sort_by=scanned_barcode&sort_order=desc&filter=no-match"),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(response.status(), Status::Ok);
    let candidates: Vec<Value> = response.into_json().await.expect("candidate rows");
    assert!(candidates.is_empty());

    let inventory_resp = common::attach_auth(
        app.client
            .get("/api/v1/operations/materials-inventory?page=1&limit=20&sort_by=incident_count&sort_order=desc"),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(inventory_resp.status(), Status::Ok);
    let inventory_rows: Vec<Value> = inventory_resp.into_json().await.expect("inventory json");
    // Sort descending by incident_count — verify monotonicity if >= 2 rows exist.
    for pair in inventory_rows.windows(2) {
        let a = pair[0]["incident_count"].as_i64().unwrap_or(0);
        let b = pair[1]["incident_count"].as_i64().unwrap_or(0);
        assert!(a >= b, "expected desc sort on incident_count ({a} vs {b})");
    }

    let alerts_resp = common::attach_auth(
        app.client
            .get("/api/v1/operations/alerts?within_days=30&page=1&limit=20"),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(alerts_resp.status(), Status::Ok);
    let alerts_body: Value = alerts_resp.into_json().await.expect("alerts json");
    assert!(
        alerts_body.is_array(),
        "alerts endpoint must return a JSON array"
    );
}

#[rocket::async_test]
async fn concurrent_duplicate_candidate_submission_conflict() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));

    let payload = json!({
        "candidate_id": "cand-concurrent",
        "date_of_birth": "03/27/2001",
        "national_id": "IDCONCURRENT",
        "scanned_barcode": "BAR-CONCURRENT",
        "metadata_json": "{\"name\":\"Cand Concurrent\",\"room_id\":\"room-x\"}"
    });

    let first = common::attach_auth(
        app.client.post("/api/v1/candidates").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    let second = common::attach_auth(
        app.client.post("/api/v1/candidates").json(&payload),
        &headers,
    )
    .dispatch()
    .await;

    let statuses = [first.status(), second.status()];
    assert!(statuses.contains(&Status::Created));
    assert!(statuses.contains(&Status::Conflict));
}

#[rocket::async_test]
async fn scan_asset_lookup_returns_asset_match() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));

    let coord_id: String = sqlx::query_scalar("SELECT id FROM users WHERE username = ?")
        .bind(COORD_USERNAME)
        .fetch_one(&app.pool)
        .await
        .expect("coord id");

    sqlx::query("INSERT INTO exam_sessions (id, template_name, duration_minutes, status, starts_at, ends_at, created_by) VALUES ('sess-scan-asset', 'base-template', 90, 'Scheduled', UTC_TIMESTAMP(), DATE_ADD(UTC_TIMESTAMP(), INTERVAL 90 MINUTE), ?)")
        .bind(&coord_id)
        .execute(&app.pool)
        .await
        .expect("insert session");

    sqlx::query("INSERT INTO assets (id, booklet_code, tracking_status, session_id, incident_count, created_by) VALUES ('asset-scan-001', 'BOOKLET-SCAN-001', 'Prepared', 'sess-scan-asset', 0, ?)")
        .bind(&coord_id)
        .execute(&app.pool)
        .await
        .expect("insert asset");

    let response = common::attach_auth(
        app.client
            .post("/api/v1/scans/lookup")
            .json(&json!({"code":"BOOKLET-SCAN-001","intent":"asset_lookup"})),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(response.status(), Status::Ok);
    let body: Value = response.into_json().await.expect("scan body");
    assert_eq!(body["found"], true);
    assert_eq!(body["asset_id"], "asset-scan-001");
}

#[rocket::async_test]
async fn scan_asset_lookup_non_owner_denied() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let owner_headers = auth_headers(&body.expect("json"));

    let owner_id: String = sqlx::query_scalar("SELECT id FROM users WHERE username = ?")
        .bind(COORD_USERNAME)
        .fetch_one(&app.pool)
        .await
        .expect("owner id");

    sqlx::query("INSERT INTO exam_sessions (id, template_name, duration_minutes, status, starts_at, ends_at, created_by) VALUES ('sess-scan-deny', 'base-template', 90, 'Scheduled', UTC_TIMESTAMP(), DATE_ADD(UTC_TIMESTAMP(), INTERVAL 90 MINUTE), ?)")
        .bind(&owner_id)
        .execute(&app.pool)
        .await
        .expect("insert session");

    sqlx::query("INSERT INTO assets (id, booklet_code, tracking_status, session_id, incident_count, created_by) VALUES ('asset-scan-deny-001', 'BOOKLET-SCAN-DENY-001', 'Prepared', 'sess-scan-deny', 0, ?)")
        .bind(&owner_id)
        .execute(&app.pool)
        .await
        .expect("insert asset");

    let second_user_id = uuid::Uuid::new_v4().to_string();
    let auth_service = app
        .client
        .rocket()
        .state::<app_services::auth_service::AuthService>()
        .expect("auth state");
    let second_hash = auth_service
        .hash_password("Another#Pass123")
        .expect("hash password");
    sqlx::query::<sqlx::MySql>(
        "INSERT INTO users (id, username, password_hash, role, failed_login_attempts) VALUES (?, ?, ?, 'Coordinator', 0)",
    )
    .bind(&second_user_id)
    .bind("coord_scan_2")
    .bind(second_hash)
    .execute(&app.pool)
    .await
    .expect("insert second user");

    let (status2, body2) = login(&app.client, "coord_scan_2", "Another#Pass123").await;
    assert_eq!(status2, Status::Ok);
    let second_headers = auth_headers(&body2.expect("json"));

    let denied_resp = common::attach_auth(
        app.client
            .post("/api/v1/scans/lookup")
            .json(&json!({"code":"BOOKLET-SCAN-DENY-001","intent":"asset_lookup"})),
        &second_headers,
    )
    .dispatch()
    .await;
    assert_eq!(denied_resp.status(), Status::Ok);
    let denied_body: Value = denied_resp.into_json().await.expect("denied scan body");
    assert_eq!(denied_body["found"], false);
    assert_eq!(denied_body["asset_id"], Value::Null);

    let owner_resp = common::attach_auth(
        app.client
            .post("/api/v1/scans/lookup")
            .json(&json!({"code":"BOOKLET-SCAN-DENY-001","intent":"asset_lookup"})),
        &owner_headers,
    )
    .dispatch()
    .await;
    assert_eq!(owner_resp.status(), Status::Ok);
    let owner_body: Value = owner_resp.into_json().await.expect("owner scan body");
    assert_eq!(owner_body["found"], true);
    assert_eq!(owner_body["asset_id"], "asset-scan-deny-001");
}

#[rocket::async_test]
async fn scan_candidate_lookup_non_owner_denied() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let owner_headers = auth_headers(&body.expect("json"));

    let create_candidate = json!({
        "candidate_id": "cand-scan-deny-001",
        "date_of_birth": "03/27/2001",
        "national_id": "ID-SCAN-DENY-001",
        "scanned_barcode": "BAR-SCAN-DENY-001",
        "metadata_json": "{\"name\":\"Scan Deny Candidate\",\"room_id\":\"room-x\"}"
    });
    let created = common::attach_auth(
        app.client
            .post("/api/v1/candidates")
            .json(&create_candidate),
        &owner_headers,
    )
    .dispatch()
    .await;
    assert_eq!(created.status(), Status::Created);

    let second_user_id = uuid::Uuid::new_v4().to_string();
    let auth_service = app
        .client
        .rocket()
        .state::<app_services::auth_service::AuthService>()
        .expect("auth state");
    let second_hash = auth_service
        .hash_password("Another#Pass123")
        .expect("hash password");
    sqlx::query::<sqlx::MySql>(
        "INSERT INTO users (id, username, password_hash, role, failed_login_attempts) VALUES (?, ?, ?, 'Coordinator', 0)",
    )
    .bind(&second_user_id)
    .bind("coord_scan_candidate_2")
    .bind(second_hash)
    .execute(&app.pool)
    .await
    .expect("insert second user");

    let (status2, body2) = login(&app.client, "coord_scan_candidate_2", "Another#Pass123").await;
    assert_eq!(status2, Status::Ok);
    let second_headers = auth_headers(&body2.expect("json"));

    let denied_resp = common::attach_auth(
        app.client
            .post("/api/v1/scans/lookup")
            .json(&json!({"code":"BAR-SCAN-DENY-001","intent":"candidate_lookup"})),
        &second_headers,
    )
    .dispatch()
    .await;
    assert_eq!(denied_resp.status(), Status::Ok);
    let denied_body: Value = denied_resp.into_json().await.expect("denied scan body");
    assert_eq!(denied_body["found"], false);
    assert_eq!(denied_body["candidate_id"], Value::Null);

    let owner_resp = common::attach_auth(
        app.client
            .post("/api/v1/scans/lookup")
            .json(&json!({"code":"BAR-SCAN-DENY-001","intent":"candidate_lookup"})),
        &owner_headers,
    )
    .dispatch()
    .await;
    assert_eq!(owner_resp.status(), Status::Ok);
    let owner_body: Value = owner_resp.into_json().await.expect("owner scan body");
    assert_eq!(owner_body["found"], true);
    assert_eq!(owner_body["candidate_id"], "cand-scan-deny-001");
}

#[rocket::async_test]
async fn attachment_upload_and_retrieval_roundtrip() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));

    let create_candidate = json!({
        "candidate_id": "cand-attach-roundtrip",
        "date_of_birth": "03/27/2001",
        "national_id": "IDATTACH001",
        "scanned_barcode": "BAR-ATTACH-001",
        "metadata_json": "{\"name\":\"Attach Roundtrip\"}"
    });
    let c_resp = common::attach_auth(
        app.client
            .post("/api/v1/candidates")
            .json(&create_candidate),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(c_resp.status(), Status::Created);

    let raw = b"attachment-bytes";
    let upload_payload = json!({
        "record_type":"candidate",
        "record_id":"cand-attach-roundtrip",
        "file_name":"proof.pdf",
        "extension":"pdf",
        "bytes_base64": base64::engine::general_purpose::STANDARD.encode(raw),
        "operator_label":"op-1",
        "device_label":"scanner-1"
    });
    let upload_resp = common::attach_auth(
        app.client.post("/api/v1/attachments").json(&upload_payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(upload_resp.status(), Status::Created);

    let list_resp = common::attach_auth(
        app.client
            .get("/api/v1/attachments?record_type=candidate&record_id=cand-attach-roundtrip"),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(list_resp.status(), Status::Ok);
    let rows: Vec<Value> = list_resp.into_json().await.expect("attachment rows");
    assert!(!rows.is_empty());
    let attachment_id = rows[0]["id"].as_str().expect("attachment id");

    let get_resp = common::attach_auth(
        app.client
            .get(format!("/api/v1/attachments/{attachment_id}")),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(get_resp.status(), Status::Ok);
    let file_body: Value = get_resp.into_json().await.expect("attachment body");
    assert_eq!(
        file_body["bytes_base64"],
        base64::engine::general_purpose::STANDARD.encode(raw)
    );
}
