mod common;

use base64::Engine;
use rocket::http::Status;
use serde_json::{json, Value};

use common::{attach_auth, login_as, setup_app, user_id_for, Role, COORD_USERNAME};

#[rocket::async_test]
async fn frontend_models_deserialize_from_live_backend_http_responses() {
    let app = setup_app().await.expect("setup");

    let login_resp = app
        .client
        .post("/api/v1/auth/login")
        .json(&json!({ "username": COORD_USERNAME, "password": common::COORD_PASSWORD }))
        .dispatch()
        .await;
    assert_eq!(login_resp.status(), Status::Ok);
    let login_body = login_resp.into_string().await.expect("login json");
    let login_model: frontend::LoginResponse =
        serde_json::from_str(&login_body).expect("frontend login model");
    assert!(!login_model.jwt.is_empty());
    assert!(!login_model.session_id.is_empty());

    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;

    common::factory_room(&app.pool, "room-fe", 42, "Frontend Hall", &coord_id).await;
    common::factory_session(&app.pool, "sess-fe", "base-template", 90, &coord_id).await;
    common::factory_asset(&app.pool, "asset-fe", "BOOK-FE", "sess-fe", &coord_id).await;
    common::factory_candidate_http(&app.client, &headers, "cand-fe", "ID-FE-001", "BAR-FE-001")
        .await;

    let output_resp = attach_auth(
        app.client.post("/api/v1/outputs").json(&json!({
            "session_id": "sess-fe",
            "mode": "TestPrint",
            "output_type": "AdmitCard"
        })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(output_resp.status(), Status::Ok);

    let attachment_resp = attach_auth(
        app.client.post("/api/v1/attachments").json(&json!({
            "record_type":"candidate",
            "record_id":"cand-fe",
            "file_name":"proof",
            "extension":"pdf",
            "bytes_base64": base64::engine::general_purpose::STANDARD.encode(b"frontend-proof"),
            "operator_label":"op-fe",
            "device_label":"scanner-fe"
        })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(attachment_resp.status(), Status::Created);

    let candidates_resp = attach_auth(
        app.client.get("/api/v1/candidates?page=1&limit=50"),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(candidates_resp.status(), Status::Ok);
    let candidates: Vec<frontend::CandidateRow> =
        candidates_resp.into_json().await.expect("candidate rows");
    assert!(candidates.iter().any(|row| row.id == "cand-fe"));

    let rooms_resp = attach_auth(app.client.get("/api/v1/rooms?page=1&limit=50"), &headers)
        .dispatch()
        .await;
    assert_eq!(rooms_resp.status(), Status::Ok);
    let rooms: Vec<frontend::RoomRow> = rooms_resp.into_json().await.expect("room rows");
    assert!(rooms.iter().any(|row| row.id == "room-fe"));

    let sessions_resp = attach_auth(app.client.get("/api/v1/sessions?page=1&limit=50"), &headers)
        .dispatch()
        .await;
    assert_eq!(sessions_resp.status(), Status::Ok);
    let sessions: Vec<frontend::SessionRow> =
        sessions_resp.into_json().await.expect("session rows");
    assert!(sessions.iter().any(|row| row.id == "sess-fe"));

    let assets_resp = attach_auth(app.client.get("/api/v1/assets?page=1&limit=50"), &headers)
        .dispatch()
        .await;
    assert_eq!(assets_resp.status(), Status::Ok);
    let assets: Vec<frontend::AssetRow> = assets_resp.into_json().await.expect("asset rows");
    assert!(assets.iter().any(|row| row.id == "asset-fe"));

    let outputs_resp = attach_auth(app.client.get("/api/v1/outputs"), &headers)
        .dispatch()
        .await;
    assert_eq!(outputs_resp.status(), Status::Ok);
    let outputs: Vec<frontend::OutputRow> = outputs_resp.into_json().await.expect("output rows");
    assert!(outputs.iter().any(|row| row.session_id == "sess-fe"));

    let attachments_resp = attach_auth(
        app.client
            .get("/api/v1/attachments?record_type=candidate&record_id=cand-fe"),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(attachments_resp.status(), Status::Ok);
    let attachments: Vec<frontend::AttachmentRow> =
        attachments_resp.into_json().await.expect("attachment rows");
    assert!(attachments.iter().any(|row| row.record_id == "cand-fe"));

    let templates_resp = attach_auth(app.client.get("/api/v1/templates"), &headers)
        .dispatch()
        .await;
    assert_eq!(templates_resp.status(), Status::Ok);
    let templates: Vec<frontend::TemplateRow> =
        templates_resp.into_json().await.expect("template rows");
    assert!(templates
        .iter()
        .any(|row| row.template_id == "base-template"));
}

#[rocket::async_test]
async fn frontend_report_models_and_scan_model_match_backend_payloads() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;

    common::factory_room(&app.pool, "room-report-fe", 35, "Report Hall", &coord_id).await;
    common::factory_session(&app.pool, "sess-report-fe", "Template A", 60, &coord_id).await;
    common::factory_asset(
        &app.pool,
        "asset-report-fe",
        "BOOK-REPORT-FE",
        "sess-report-fe",
        &coord_id,
    )
    .await;
    sqlx::query("UPDATE assets SET tracking_status = 'Collected' WHERE id = 'asset-report-fe'")
        .execute(&app.pool)
        .await
        .expect("update asset status");

    common::factory_candidate_http(
        &app.client,
        &headers,
        "cand-report-fe",
        "ID-REPORT-FE-001",
        "BAR-REPORT-FE-001",
    )
    .await;

    let incident_rows: Vec<frontend::IncidentRow> = attach_auth(
        app.client.get("/api/v1/operations/incident-rates"),
        &headers,
    )
    .dispatch()
    .await
    .into_json()
    .await
    .expect("incident rows");
    assert!(incident_rows.iter().all(|row| !row.session_id.is_empty()));

    let return_rows: Vec<frontend::ReturnRateRow> =
        attach_auth(app.client.get("/api/v1/operations/return-rates"), &headers)
            .dispatch()
            .await
            .into_json()
            .await
            .expect("return rows");
    assert!(return_rows
        .iter()
        .any(|row| row.session_id == "sess-report-fe"));

    let inventory_rows: Vec<frontend::MaterialInventoryRow> = attach_auth(
        app.client
            .get("/api/v1/operations/materials-inventory?page=1&limit=100"),
        &headers,
    )
    .dispatch()
    .await
    .into_json()
    .await
    .expect("inventory rows");
    assert!(inventory_rows
        .iter()
        .any(|row| row.asset_id == "asset-report-fe"));

    let alert_rows: Vec<frontend::AlertRow> = attach_auth(
        app.client
            .get("/api/v1/operations/alerts?within_days=30&page=1&limit=100"),
        &headers,
    )
    .dispatch()
    .await
    .into_json()
    .await
    .expect("alert rows");
    assert!(alert_rows.iter().all(|row| !row.alert_type.is_empty()));

    let export_resp = attach_auth(
        app.client.post("/api/v1/exports/csv").json(&json!({
            "report": "incident_rates",
            "limit": 100
        })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(export_resp.status(), Status::Ok);
    let export_body = export_resp.into_string().await.expect("export json");
    let export_model: frontend::ExportResponse =
        serde_json::from_str(&export_body).expect("frontend export model");
    assert!(export_model.content.contains("session_id"));

    let scan_resp = attach_auth(
        app.client
            .post("/api/v1/scans/lookup")
            .json(&json!({"code":"BAR-REPORT-FE-001","intent":"candidate_lookup"})),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(scan_resp.status(), Status::Ok);
    let scan_body = scan_resp.into_string().await.expect("scan json");
    let scan_model: frontend::ScanResp =
        serde_json::from_str(&scan_body).expect("frontend scan model");
    assert!(scan_model.found);
    assert_eq!(scan_model.candidate_id.as_deref(), Some("cand-report-fe"));
}

#[rocket::async_test]
async fn frontend_dashboard_summary_model_can_be_composed_from_live_endpoints() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;

    common::factory_room(&app.pool, "room-dash-fe", 30, "Dash Hall", &coord_id).await;
    common::factory_session(&app.pool, "sess-dash-fe", "Template A", 60, &coord_id).await;
    common::factory_asset(
        &app.pool,
        "asset-dash-fe",
        "BOOK-DASH-FE",
        "sess-dash-fe",
        &coord_id,
    )
    .await;
    common::factory_candidate_http(
        &app.client,
        &headers,
        "cand-dash-fe",
        "ID-DASH-FE-001",
        "BAR-DASH-FE-001",
    )
    .await;
    attach_auth(
        app.client.post("/api/v1/outputs").json(&json!({
            "session_id": "sess-dash-fe",
            "output_type": "AdmitCard",
            "mode": "Draft"
        })),
        &headers,
    )
    .dispatch()
    .await;

    let legacy: Value = attach_auth(app.client.get("/api/v1/reports/dashboard"), &headers)
        .dispatch()
        .await
        .into_json()
        .await
        .expect("legacy dashboard");
    let candidates: Vec<frontend::CandidateRow> = attach_auth(
        app.client.get("/api/v1/candidates?page=1&limit=200"),
        &headers,
    )
    .dispatch()
    .await
    .into_json()
    .await
    .expect("candidate rows");
    let rooms: Vec<frontend::RoomRow> =
        attach_auth(app.client.get("/api/v1/rooms?page=1&limit=200"), &headers)
            .dispatch()
            .await
            .into_json()
            .await
            .expect("room rows");
    let sessions: Vec<frontend::SessionRow> = attach_auth(
        app.client.get("/api/v1/sessions?page=1&limit=200"),
        &headers,
    )
    .dispatch()
    .await
    .into_json()
    .await
    .expect("session rows");
    let outputs: Vec<frontend::OutputRow> =
        attach_auth(app.client.get("/api/v1/outputs?page=1&limit=200"), &headers)
            .dispatch()
            .await
            .into_json()
            .await
            .expect("output rows");

    let summary = frontend::DashboardSummary {
        total_candidates: candidates.len() as i64,
        total_rooms: rooms.len() as i64,
        total_sessions_this_week: sessions.len() as i64,
        seat_utilization_count: legacy["seat_utilization_count"]
            .as_u64()
            .unwrap_or_default() as usize,
        near_expiry_count: legacy["near_expiry_count"].as_u64().unwrap_or_default() as usize,
        incident_rate_count: legacy["incident_rate_count"].as_u64().unwrap_or_default() as usize,
        upcoming_sessions: sessions
            .iter()
            .take(5)
            .map(|row| frontend::UpcomingSession {
                id: row.id.clone(),
                template_name: row.template_name.clone(),
                status: row.status.clone(),
                starts_at: row.starts_at.clone().unwrap_or_default(),
            })
            .collect(),
        recent_outputs: outputs
            .iter()
            .take(5)
            .map(|row| frontend::RecentOutput {
                id: row.id.clone(),
                output_type: row.output_type.clone(),
                mode: row.mode.clone(),
                created_at: row.created_at.clone(),
            })
            .collect(),
    };

    assert!(summary.total_candidates >= 1);
    assert!(summary.total_rooms >= 1);
    assert!(summary.total_sessions_this_week >= 1);
    assert!(frontend::dashboard_view(summary).is_ok());
}
