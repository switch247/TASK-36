mod common;

use base64::Engine;
use rocket::http::Status;
use serde_json::json;

use common::{
    auth_headers, login, setup_app, COORD_PASSWORD, COORD_USERNAME, PROCTOR_PASSWORD,
    PROCTOR_USERNAME,
};

async fn seed_candidate_for_attachment(
    client: &rocket::local::asynchronous::Client,
    headers: &[rocket::http::Header<'static>],
    candidate_id: &str,
) {
    let payload = json!({
        "candidate_id": candidate_id,
        "date_of_birth": "03/27/2001",
        "national_id": format!("NID-{candidate_id}"),
        "scanned_barcode": format!("BAR-{candidate_id}"),
        "metadata_json": "{\"name\":\"Attach Candidate\"}"
    });
    let _ = common::attach_auth(client.post("/api/v1/candidates").json(&payload), headers)
        .dispatch()
        .await;
}

#[rocket::async_test]
async fn missing_auth_returns_401() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let response = app
        .client
        .get("/api/v1/operations/incident-rates")
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Unauthorized);
}

#[rocket::async_test]
async fn forbidden_role_returns_403() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, PROCTOR_USERNAME, PROCTOR_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));

    let response = common::attach_auth(app.client.get("/api/v1/reports/dashboard"), &headers)
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Forbidden);
    let body: serde_json::Value = response.into_json().await.expect("error body");
    assert_eq!(body["code"].as_u64(), Some(403));
    assert!(
        body["message"].as_str().unwrap_or("").len() > 0,
        "forbidden response must carry a message"
    );
}

#[rocket::async_test]
async fn not_found_resource_returns_404() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));

    let response = common::attach_auth(
        app.client.get("/api/v1/candidates/does-not-exist"),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(response.status(), Status::NotFound);
}

#[rocket::async_test]
async fn duplicate_attachment_fingerprint_returns_409() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));

    let raw = b"same-bytes";
    let b64 = base64::engine::general_purpose::STANDARD.encode(raw);
    let payload = json!({
        "record_type": "candidate",
        "record_id": "cand-attach-1",
        "file_name": "doc1.pdf",
        "extension": "pdf",
        "bytes_base64": b64,
        "operator_label": "op-1",
        "device_label": "scanner-1"
    });
    seed_candidate_for_attachment(&app.client, &headers, "cand-attach-1").await;

    let first = common::attach_auth(
        app.client.post("/api/v1/attachments").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(first.status(), Status::Created);

    let second = common::attach_auth(
        app.client.post("/api/v1/attachments").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(second.status(), Status::Conflict);
}

#[rocket::async_test]
async fn attachment_count_limit_returns_400() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));

    for i in 0..10 {
        seed_candidate_for_attachment(&app.client, &headers, "cand-attach-limit").await;
        let bytes = format!("bytes-{i}").into_bytes();
        let payload = json!({
            "record_type": "candidate",
            "record_id": "cand-attach-limit",
            "file_name": format!("file-{i}.pdf"),
            "extension": "pdf",
            "bytes_base64": base64::engine::general_purpose::STANDARD.encode(bytes),
            "operator_label": "op-1",
            "device_label": "scanner-1"
        });

        let response = common::attach_auth(
            app.client.post("/api/v1/attachments").json(&payload),
            &headers,
        )
        .dispatch()
        .await;
        assert_eq!(response.status(), Status::Created);
    }

    let payload = json!({
        "record_type": "candidate",
        "record_id": "cand-attach-limit",
        "file_name": "overflow.pdf",
        "extension": "pdf",
        "bytes_base64": base64::engine::general_purpose::STANDARD.encode(b"overflow"),
        "operator_label": "op-1",
        "device_label": "scanner-1"
    });

    let response = common::attach_auth(
        app.client.post("/api/v1/attachments").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(response.status(), Status::BadRequest);
}

#[rocket::async_test]
async fn attachment_invalid_extension_returns_400() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));

    let payload = json!({
        "record_type": "candidate",
        "record_id": "cand-attach-ext",
        "file_name": "virus.exe",
        "extension": "exe",
        "bytes_base64": base64::engine::general_purpose::STANDARD.encode(b"bad"),
        "operator_label": "op-1",
        "device_label": "scanner-1"
    });
    seed_candidate_for_attachment(&app.client, &headers, "cand-attach-ext").await;

    let response = common::attach_auth(
        app.client.post("/api/v1/attachments").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(response.status(), Status::BadRequest);
}

#[rocket::async_test]
async fn template_validation_missing_required_candidate_name_returns_400() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));

    let payload = json!({
        "candidate_id": "cand-template-missing-name",
        "date_of_birth": "03/27/2001",
        "national_id": "NID-TEMPLATE-MISSING-NAME",
        "scanned_barcode": "BAR-TEMPLATE-MISSING-NAME",
        "metadata_json": "{\"room_id\":\"room-x\"}"
    });

    let response = common::attach_auth(
        app.client.post("/api/v1/candidates").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(response.status(), Status::BadRequest);
}
