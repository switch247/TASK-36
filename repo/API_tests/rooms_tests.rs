mod common;

use rocket::http::Status;
use serde_json::{json, Value};

use common::{attach_auth, factory_room, login_as, setup_app, user_id_for, Role, COORD_USERNAME};

fn room_payload(id: &str, capacity: i32, location: &str) -> Value {
    json!({
        "id": id,
        "capacity": capacity,
        "location": location
    })
}

#[rocket::async_test]
async fn create_room_as_coordinator_persists_and_audits() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let resp = attach_auth(
        app.client
            .post("/api/v1/rooms")
            .json(&room_payload("room-new-1", 50, "Hall A")),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Created);

    let row: (i32, String) = sqlx::query_as("SELECT capacity, location FROM rooms WHERE id = ?")
        .bind("room-new-1")
        .fetch_one(&app.pool)
        .await
        .expect("room row");
    assert_eq!(row, (50, "Hall A".to_string()));

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM entity_change_history WHERE entity_name = 'rooms' AND entity_id = ?",
    )
    .bind("room-new-1")
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert!(audit_count >= 1, "entity_change_history must record create");
}

#[rocket::async_test]
async fn create_room_capacity_out_of_range_returns_400() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let resp = attach_auth(
        app.client
            .post("/api/v1/rooms")
            .json(&room_payload("room-too-big", 9999, "Hall A")),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::BadRequest);
    let body: Value = resp.into_json().await.expect("error body");
    assert_eq!(body["code"], 400);
    assert_eq!(body["message"], "validation failed");
    assert_eq!(body["details"]["field"], "capacity");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rooms WHERE id = ?")
        .bind("room-too-big")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[rocket::async_test]
async fn create_room_duplicate_id_returns_409() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let first = attach_auth(
        app.client
            .post("/api/v1/rooms")
            .json(&room_payload("room-dup", 80, "Hall B")),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(first.status(), Status::Created);

    let second = attach_auth(
        app.client
            .post("/api/v1/rooms")
            .json(&room_payload("room-dup", 80, "Hall B")),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(second.status(), Status::Conflict);
    let body: Value = second.into_json().await.expect("error body");
    assert_eq!(body["code"], 409);
    assert!(body["message"]
        .as_str()
        .unwrap_or_default()
        .contains("room"));
}

#[rocket::async_test]
async fn create_room_forbidden_for_proctor() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Proctor).await;

    let resp = attach_auth(
        app.client
            .post("/api/v1/rooms")
            .json(&room_payload("room-proc", 40, "Hall P")),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Forbidden);
    let body: Value = resp.into_json().await.expect("error body");
    assert_eq!(body["code"], 403);
    assert!(body["message"]
        .as_str()
        .unwrap_or_default()
        .contains("role"));
}

#[rocket::async_test]
async fn list_rooms_unauthenticated_returns_401() {
    let app = setup_app().await.expect("setup");
    let resp = app.client.get("/api/v1/rooms").dispatch().await;
    assert_eq!(resp.status(), Status::Unauthorized);
}

#[rocket::async_test]
async fn list_rooms_scoped_to_coordinator() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;

    // seeded by this coordinator
    factory_room(&app.pool, "room-mine", 30, "Mine", &coord_id).await;

    // owned by a different coordinator — should NOT show up for COORD_USERNAME
    let other_id = common::factory_user(
        &app.client,
        &app.pool,
        "coord_other_user",
        "StrongPass#2026!",
        "Coordinator",
    )
    .await;
    factory_room(&app.pool, "room-other", 40, "Other", &other_id).await;

    let resp = attach_auth(app.client.get("/api/v1/rooms?limit=100"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("rows");
    let ids: Vec<&str> = rows.iter().filter_map(|r| r["id"].as_str()).collect();
    assert!(ids.contains(&"room-mine"));
    assert!(
        !ids.contains(&"room-other"),
        "coordinator must not see another coordinator's rooms"
    );
}

#[rocket::async_test]
async fn list_rooms_admin_sees_all() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Admin).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_room(&app.pool, "room-seed", 25, "Seed", &coord_id).await;

    let resp = attach_auth(app.client.get("/api/v1/rooms?limit=100"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("rows");
    assert!(rows.iter().any(|r| r["id"] == "room-seed"));
}

#[rocket::async_test]
async fn list_rooms_with_filter_restricts_results() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_room(&app.pool, "filter-aa", 10, "Alpha", &coord_id).await;
    factory_room(&app.pool, "filter-bb", 10, "Beta", &coord_id).await;

    let resp = attach_auth(
        app.client.get("/api/v1/rooms?filter=alpha&limit=50"),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);
    let rows: Vec<Value> = resp.into_json().await.expect("rows");
    let matched: Vec<&str> = rows.iter().filter_map(|r| r["location"].as_str()).collect();
    assert!(matched
        .iter()
        .all(|loc| loc.to_ascii_lowercase().contains("alpha")));
}

#[rocket::async_test]
async fn update_room_by_owner_persists_and_returns_200() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_room(&app.pool, "room-upd", 20, "Old", &coord_id).await;

    let resp = attach_auth(
        app.client
            .put("/api/v1/rooms/room-upd")
            .json(&room_payload("room-upd", 75, "Renovated")),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Ok);

    let (cap, loc): (i32, String) =
        sqlx::query_as("SELECT capacity, location FROM rooms WHERE id = ?")
            .bind("room-upd")
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(cap, 75);
    assert_eq!(loc, "Renovated");
}

#[rocket::async_test]
async fn update_room_not_found_returns_404() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let resp = attach_auth(
        app.client
            .put("/api/v1/rooms/ghost")
            .json(&room_payload("ghost", 30, "Nowhere")),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::NotFound);
    let body: Value = resp.into_json().await.expect("error body");
    assert_eq!(body["code"], 404);
    assert!(body["message"]
        .as_str()
        .unwrap_or_default()
        .contains("room"));
}

#[rocket::async_test]
async fn update_room_owned_by_another_user_returns_404() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let other_id = common::factory_user(
        &app.client,
        &app.pool,
        "coord_update_other",
        "StrongPass#2026!",
        "Coordinator",
    )
    .await;
    factory_room(&app.pool, "room-foreign", 20, "Foreign", &other_id).await;

    let resp = attach_auth(
        app.client
            .put("/api/v1/rooms/room-foreign")
            .json(&room_payload("room-foreign", 50, "Tampered")),
        &headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::NotFound);

    let (cap, loc): (i32, String) =
        sqlx::query_as("SELECT capacity, location FROM rooms WHERE id = ?")
            .bind("room-foreign")
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(
        cap, 20,
        "foreign coordinator must not mutate another's room"
    );
    assert_eq!(loc, "Foreign");
}

#[rocket::async_test]
async fn delete_room_by_owner_returns_204_and_removes_row() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_room(&app.pool, "room-del", 20, "Del", &coord_id).await;

    let resp = attach_auth(app.client.delete("/api/v1/rooms/room-del"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::NoContent);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rooms WHERE id = ?")
        .bind("room-del")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[rocket::async_test]
async fn delete_room_forbidden_for_proctor() {
    let app = setup_app().await.expect("setup");
    let proctor_headers = login_as(&app.client, Role::Proctor).await;
    let coord_id = user_id_for(&app.pool, COORD_USERNAME).await;
    factory_room(&app.pool, "room-keep", 20, "Keep", &coord_id).await;

    let resp = attach_auth(
        app.client.delete("/api/v1/rooms/room-keep"),
        &proctor_headers,
    )
    .dispatch()
    .await;
    assert_eq!(resp.status(), Status::Forbidden);
}

#[rocket::async_test]
async fn delete_room_not_found_returns_404() {
    let app = setup_app().await.expect("setup");
    let headers = login_as(&app.client, Role::Coordinator).await;

    let resp = attach_auth(app.client.delete("/api/v1/rooms/nothing-here"), &headers)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::NotFound);
    let body: Value = resp.into_json().await.expect("error body");
    assert_eq!(body["code"], 404);
    assert!(body["message"]
        .as_str()
        .unwrap_or_default()
        .contains("room"));
}
