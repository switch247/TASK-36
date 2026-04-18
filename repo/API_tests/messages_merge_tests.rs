mod common;

use rocket::http::Status;
use serde_json::json;

use common::{attach_auth, factory_candidate_http, login_as, setup_app, Role};

#[rocket::async_test]
async fn create_message_draft_persists_for_coordinator() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let payload = json!({
        "channel": "Email",
        "recipient": "candidate@example.test",
        "subject": "Exam confirmation",
        "body": "Your exam is scheduled."
    });
    let resp = attach_auth(
        app.client.post("/api/v1/messages/drafts").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Created);

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM message_drafts WHERE recipient = 'candidate@example.test'",
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert_eq!(count, 1);
}

#[rocket::async_test]
async fn create_message_draft_forbidden_for_proctor() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Proctor).await;

    let payload = json!({
        "channel": "Email",
        "recipient": "nope@example.test",
        "body": "denied"
    });
    let resp = attach_auth(
        app.client.post("/api/v1/messages/drafts").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Forbidden);
}

#[rocket::async_test]
async fn create_message_draft_unauthenticated_returns_401() {
    let app = setup_app().await.expect("setup");
    let resp = app
        .client
        .post("/api/v1/messages/drafts")
        .json(&json!({"channel":"Email","recipient":"x@y.z","body":"hi"}))
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Unauthorized);
}

#[rocket::async_test]
async fn create_merge_candidate_persists_row_and_audits() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    factory_candidate_http(&app.client, &headers, "cand-m-left", "NID-M-L", "BAR-M-L").await;
    factory_candidate_http(&app.client, &headers, "cand-m-right", "NID-M-R", "BAR-M-R").await;

    let payload = json!({
        "left_candidate_id": "cand-m-left",
        "right_candidate_id": "cand-m-right",
        "similarity_score": 0.91
    });
    let resp = attach_auth(
        app.client.post("/api/v1/candidates/merge").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Created);

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM merge_candidates WHERE left_candidate_id = 'cand-m-left' AND right_candidate_id = 'cand-m-right'",
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert_eq!(count, 1);

    let similarity: f64 = sqlx::query_scalar(
        "SELECT similarity_score FROM merge_candidates WHERE left_candidate_id = 'cand-m-left'",
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert!((similarity - 0.91).abs() < f64::EPSILON);
}

#[rocket::async_test]
async fn create_merge_candidate_forbidden_for_proctor() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Proctor).await;

    let payload = json!({
        "left_candidate_id": "ignored-left",
        "right_candidate_id": "ignored-right",
        "similarity_score": 0.5
    });
    let resp = attach_auth(
        app.client.post("/api/v1/candidates/merge").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Forbidden);
}
