mod common;

use rocket::http::Status;
use serde_json::{json, Value};

use common::{
    attach_auth, factory_session, login_as, setup_app, user_id_for, Role, COORD_USERNAME,
};

async fn setup_session_for_print(app: &common::TestApp) -> String {
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    let session_id = "sess-output-typed";
    factory_session(&app.pool, session_id, "Template A", 60, &coord_id).await;
    session_id.to_string()
}

#[rocket::async_test]
async fn print_admit_cards_endpoint_produces_admit_card_output() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let session_id = setup_session_for_print(&app).await;

    let resp = attach_auth(
        app.client.post("/api/v1/outputs/admit-cards").json(&json!({
            "session_id": session_id,
            "output_type": "Ignored", // server forces AdmitCard
            "mode": "Draft"
        })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.expect("json");
    assert_eq!(body["output_type"], "AdmitCard");
    assert_eq!(body["mode"], "Draft");
}

#[rocket::async_test]
async fn print_seating_charts_endpoint_returns_seating_chart() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let session_id = setup_session_for_print(&app).await;

    let resp = attach_auth(
        app.client
            .post("/api/v1/outputs/seating-charts")
            .json(&json!({
                "session_id": session_id,
                "output_type": "x",
                "mode": "Draft"
            })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.expect("json");
    assert_eq!(body["output_type"], "SeatingChart");
}

#[rocket::async_test]
async fn print_door_signs_endpoint_returns_door_sign() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let session_id = setup_session_for_print(&app).await;

    let resp = attach_auth(
        app.client.post("/api/v1/outputs/door-signs").json(&json!({
            "session_id": session_id,
            "output_type": "x",
            "mode": "Draft"
        })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.expect("json");
    assert_eq!(body["output_type"], "DoorSign");
}

#[rocket::async_test]
async fn print_proctor_packet_endpoint_returns_proctor_packet() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let session_id = setup_session_for_print(&app).await;

    let resp = attach_auth(
        app.client
            .post("/api/v1/outputs/proctor-packet")
            .json(&json!({
                "session_id": session_id,
                "output_type": "x",
                "mode": "Draft"
            })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.expect("json");
    assert_eq!(body["output_type"], "ProctorPacket");
}

#[rocket::async_test]
async fn print_summary_report_endpoint_returns_summary_report() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let session_id = setup_session_for_print(&app).await;

    let resp = attach_auth(
        app.client
            .post("/api/v1/outputs/summary-report")
            .json(&json!({
                "session_id": session_id,
                "output_type": "x",
                "mode": "Draft"
            })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.expect("json");
    assert_eq!(body["output_type"], "SummaryReport");
}

#[rocket::async_test]
async fn print_admit_cards_forbidden_for_auditor() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Auditor).await;

    let resp = attach_auth(
        app.client.post("/api/v1/outputs/admit-cards").json(&json!({
            "session_id": "any",
            "output_type": "AdmitCard",
            "mode": "Draft"
        })),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Forbidden);
    let body: Value = resp.into_json().await.expect("json");
    assert_eq!(body["code"], 403);
}

#[rocket::async_test]
async fn list_outputs_returns_recent_outputs() {
    let app = setup_app().await.expect("setup");
    let coord_headers = login_as(&app.client, Role::Coordinator).await;
    let session_id = setup_session_for_print(&app).await;

    // Generate a draft output so there's at least one row.
    attach_auth(
        app.client.post("/api/v1/outputs").json(&json!({
            "session_id": session_id,
            "output_type": "AdmitCard",
            "mode": "Draft"
        })),
        &coord_headers,
    )
    .dispatch()
    .await;

    let resp = attach_auth(app.client.get("/api/v1/outputs?limit=10"), &coord_headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("rows");
    assert!(rows
        .iter()
        .any(|r| r["session_id"] == session_id && r["output_type"] == "AdmitCard"));
}

#[rocket::async_test]
async fn list_outputs_fallback_without_query_still_returns_rows() {
    let app = setup_app().await.expect("setup");
    let coord_headers = login_as(&app.client, Role::Coordinator).await;
    let first_session_id = setup_session_for_print(&app).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    let second_session_id = "sess-output-fallback-second";
    factory_session(&app.pool, second_session_id, "Template A", 60, &coord_id).await;

    attach_auth(
        app.client.post("/api/v1/outputs").json(&json!({
            "session_id": first_session_id,
            "output_type": "AdmitCard",
            "mode": "Draft"
        })),
        &coord_headers,
    )
    .dispatch()
    .await;
    attach_auth(
        app.client.post("/api/v1/outputs").json(&json!({
            "session_id": second_session_id,
            "output_type": "SummaryReport",
            "mode": "FinalPrint"
        })),
        &coord_headers,
    )
    .dispatch()
    .await;

    let resp = attach_auth(app.client.get("/api/v1/outputs"), &coord_headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("rows");
    assert!(
        rows.len() >= 2,
        "fallback should return the generated outputs, got {} rows",
        rows.len()
    );
    assert_eq!(rows[0]["session_id"], second_session_id);
    assert_eq!(rows[0]["output_type"], "SummaryReport");
    assert_eq!(rows[0]["mode"], "FinalPrint");
    assert!(
        rows.iter().any(|row| row["session_id"] == first_session_id),
        "fallback should still include the earlier generated output"
    );
}

#[rocket::async_test]
async fn list_outputs_forbidden_for_proctor() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Proctor).await;
    let resp = attach_auth(app.client.get("/api/v1/outputs?limit=10"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Forbidden);
    let body: Value = resp.into_json().await.expect("json");
    assert_eq!(body["code"], 403);
}
