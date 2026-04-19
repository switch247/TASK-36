mod common;

use rocket::http::Status;
use serde_json::json;

use common::{
    auth_headers, login, setup_app, COORD_PASSWORD, COORD_USERNAME, PROCTOR_PASSWORD,
    PROCTOR_USERNAME,
};

#[rocket::async_test]
async fn create_read_update_delete_candidate_with_auth() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));

    let create_payload = json!({
        "candidate_id": "cand-001",
        "date_of_birth": "03/27/2001",
        "national_id": "ID00112233",
        "scanned_barcode": "BAR-001",
        "metadata_json": "{\"name\":\"Cand One\",\"room_id\":\"room-a\"}"
    });

    let response = common::attach_auth(
        app.client.post("/api/v1/candidates").json(&create_payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(response.status(), Status::Created);

    let response = common::attach_auth(app.client.get("/api/v1/candidates/cand-001"), &headers)
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Ok);
    let read_body: serde_json::Value = response.into_json().await.expect("candidate body");
    assert_eq!(read_body["id"], "cand-001");
    assert_eq!(read_body["national_id"], "ID00112233");
    assert_eq!(read_body["scanned_barcode"], "BAR-001");
    assert_eq!(
        read_body["dob_masked"], "**/**/****",
        "DOB must be masked in responses"
    );

    let update_payload = json!({
        "scanned_barcode": "BAR-UPDATED",
        "metadata_json": "{\"room_id\":\"room-b\"}"
    });

    let response = common::attach_auth(
        app.client
            .put("/api/v1/candidates/cand-001")
            .json(&update_payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(response.status(), Status::Ok);

    // Verify the update actually mutated the row.
    let updated_barcode: String =
        sqlx::query_scalar("SELECT scanned_barcode FROM candidates WHERE id = ?")
            .bind("cand-001")
            .fetch_one(&app.pool)
            .await
            .expect("candidate row after update");
    assert_eq!(updated_barcode, "BAR-UPDATED");

    let response = common::attach_auth(app.client.delete("/api/v1/candidates/cand-001"), &headers)
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::NoContent);

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM candidates WHERE id = ?")
        .bind("cand-001")
        .fetch_one(&app.pool)
        .await
        .expect("count after delete");
    assert_eq!(
        remaining, 0,
        "delete must actually remove the candidate row"
    );

    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM entity_change_history WHERE entity_name = 'candidates'",
    )
    .fetch_one(&app.pool)
    .await
    .expect("history query");
    assert!(count >= 3);
}

#[rocket::async_test]
async fn duplicate_candidate_returns_409() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));

    let payload = json!({
        "candidate_id": "cand-dup",
        "date_of_birth": "03/27/2001",
        "national_id": "ID009999",
        "scanned_barcode": "BAR-DUP",
        "metadata_json": "{\"name\":\"Cand Dup\",\"room_id\":\"room-a\"}"
    });

    let first = common::attach_auth(
        app.client.post("/api/v1/candidates").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(first.status(), Status::Created);

    let second = common::attach_auth(
        app.client.post("/api/v1/candidates").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(second.status(), Status::Conflict);
}

#[rocket::async_test]
async fn duplicate_candidate_by_barcode_returns_409() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));

    let first = json!({
        "candidate_id": "cand-dup-bar-1",
        "date_of_birth": "03/27/2001",
        "national_id": "IDBAR001",
        "scanned_barcode": "BAR-SHARED-001",
        "metadata_json": "{\"name\":\"Alpha One\",\"room_id\":\"room-a\"}"
    });
    let second = json!({
        "candidate_id": "cand-dup-bar-2",
        "date_of_birth": "03/27/2001",
        "national_id": "IDBAR002",
        "scanned_barcode": "BAR-SHARED-001",
        "metadata_json": "{\"name\":\"Beta Two\",\"room_id\":\"room-a\"}"
    });

    let first_resp =
        common::attach_auth(app.client.post("/api/v1/candidates").json(&first), &headers)
            .dispatch()
            .await;
    assert_eq!(first_resp.status(), Status::Created);

    let second_resp = common::attach_auth(
        app.client.post("/api/v1/candidates").json(&second),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(second_resp.status(), Status::Conflict);
}

#[rocket::async_test]
async fn guided_merge_duplicate_by_name_and_dob_returns_409() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));

    let first = json!({
        "candidate_id": "cand-merge-1",
        "date_of_birth": "03/27/2001",
        "national_id": "IDMERGE001",
        "scanned_barcode": "BAR-MERGE-001",
        "metadata_json": "{\"name\":\"Jane Alexandra Doe\",\"room_id\":\"room-a\"}"
    });
    let second = json!({
        "candidate_id": "cand-merge-2",
        "date_of_birth": "03/27/2001",
        "national_id": "IDMERGE002",
        "scanned_barcode": "BAR-MERGE-002",
        "metadata_json": "{\"name\":\"Jane Alexndra Doe\",\"room_id\":\"room-a\"}"
    });

    let first_resp =
        common::attach_auth(app.client.post("/api/v1/candidates").json(&first), &headers)
            .dispatch()
            .await;
    assert_eq!(first_resp.status(), Status::Created);

    let second_resp = common::attach_auth(
        app.client.post("/api/v1/candidates").json(&second),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(second_resp.status(), Status::Conflict);
}

#[rocket::async_test]
async fn insufficient_role_on_inventory_route_returns_403() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (status, body) = login(&app.client, PROCTOR_USERNAME, PROCTOR_PASSWORD).await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));

    let payload = json!({
        "candidate_id": "cand-403",
        "date_of_birth": "03/27/2001",
        "national_id": "ID003333",
        "scanned_barcode": "BAR-403",
        "metadata_json": "{\"room_id\":\"room-a\"}"
    });

    let response = common::attach_auth(
        app.client.post("/api/v1/candidates").json(&payload),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(response.status(), Status::Forbidden);
}

#[rocket::async_test]
async fn cross_user_access_returns_404() {
    let app = setup_app().await.expect("Failed to initialize test app");

    let (coord_status, coord_body) = login(&app.client, COORD_USERNAME, COORD_PASSWORD).await;
    assert_eq!(coord_status, Status::Ok);
    let coord_headers = auth_headers(&coord_body.expect("json"));

    let create_payload = json!({
        "candidate_id": "cand-owned",
        "date_of_birth": "03/27/2001",
        "national_id": "ID004444",
        "scanned_barcode": "BAR-OWNED",
        "metadata_json": "{\"name\":\"Cand Owned\",\"room_id\":\"room-a\"}"
    });

    let created = common::attach_auth(
        app.client.post("/api/v1/candidates").json(&create_payload),
        &coord_headers,
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
    .bind("coord_2")
    .bind(second_hash)
    .execute(&app.pool)
    .await
    .expect("insert second user");

    let (status, body) = login(&app.client, "coord_2", "Another#Pass123").await;
    assert_eq!(status, Status::Ok);
    let headers = auth_headers(&body.expect("json"));

    let response = common::attach_auth(app.client.get("/api/v1/candidates/cand-owned"), &headers)
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::NotFound);
}
