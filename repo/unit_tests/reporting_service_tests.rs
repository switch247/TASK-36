#[path = "../API_tests/common.rs"]
mod common;

#[cfg(test)]
mod tests {
    use super::common;
    use app_services::reporting_service::ReportingService;
    use chrono::{Duration, Utc};

    #[rocket::async_test]
    async fn reporting_service_returns_structured_report_rows() {
        let app = common::setup_app().await.expect("setup");
        let service = ReportingService::new(app.pool.clone());
        let coord_id = common::user_id_for(&app.pool, common::COORD_USERNAME).await;

        common::factory_room(&app.pool, "room-report-unit", 40, "Unit Hall", &coord_id).await;
        common::factory_session(&app.pool, "sess-report-unit", "Template A", 60, &coord_id).await;
        common::factory_asset(
            &app.pool,
            "asset-report-unit",
            "BOOK-REPORT-UNIT",
            "sess-report-unit",
            &coord_id,
        )
        .await;
        common::factory_candidate_http(
            &app.client,
            &common::login_as(&app.client, common::Role::Coordinator).await,
            "cand-report-unit",
            "ID-REPORT-UNIT",
            "BAR-REPORT-UNIT",
        )
        .await;
        sqlx::query("UPDATE assets SET incident_count = 3, tracking_status = 'Collected', expires_on = ? WHERE id = 'asset-report-unit'")
            .bind((Utc::now() + Duration::days(7)).date_naive())
            .execute(&app.pool)
            .await
            .expect("update asset");

        let seat = service.seat_utilization().await.expect("seat utilization");
        assert!(seat.iter().any(|row| row.room_id == "room-report-unit"));

        let near_expiry = service.near_expiry_assets(30).await.expect("near expiry");
        assert!(near_expiry.iter().any(|row| row.id == "asset-report-unit"));

        let incidents = service.incident_rates().await.expect("incidents");
        assert!(incidents
            .iter()
            .any(|row| row.session_id == "sess-report-unit"));

        let returns = service.return_rates().await.expect("returns");
        assert!(returns
            .iter()
            .any(|row| row.session_id == "sess-report-unit"));

        let inventory = service.materials_inventory().await.expect("inventory");
        assert!(inventory
            .iter()
            .any(|row| row.asset_id == "asset-report-unit"));
    }

    #[rocket::async_test]
    async fn reporting_service_operations_alerts_surface_all_alert_types() {
        let app = common::setup_app().await.expect("setup");
        let service = ReportingService::new(app.pool.clone());
        let coord_id = common::user_id_for(&app.pool, common::COORD_USERNAME).await;

        common::factory_session(&app.pool, "sess-alert-hi", "Template A", 60, &coord_id).await;
        common::factory_asset(
            &app.pool,
            "asset-alert-hi",
            "BOOK-ALERT-HI",
            "sess-alert-hi",
            &coord_id,
        )
        .await;
        sqlx::query("UPDATE assets SET incident_count = 3, tracking_status = 'Prepared', expires_on = ? WHERE id = 'asset-alert-hi'")
            .bind((Utc::now() + Duration::days(3)).date_naive())
            .execute(&app.pool)
            .await
            .expect("update high incident asset");

        common::factory_session(
            &app.pool,
            "sess-alert-low-return",
            "Template A",
            60,
            &coord_id,
        )
        .await;
        common::factory_asset(
            &app.pool,
            "asset-alert-low-return",
            "BOOK-ALERT-LOW",
            "sess-alert-low-return",
            &coord_id,
        )
        .await;

        let alerts = service.operations_alerts(30).await.expect("alerts");
        assert!(alerts.iter().any(|a| a.alert_type == "NearExpiry"));
        assert!(alerts.iter().any(|a| a.alert_type == "HighIncident"));
        assert!(alerts.iter().any(|a| a.alert_type == "LowReturnRate"));
    }

    #[rocket::async_test]
    async fn reporting_service_near_expiry_clamps_zero_days_to_a_one_day_window() {
        let app = common::setup_app().await.expect("setup");
        let service = ReportingService::new(app.pool.clone());
        let coord_id = common::user_id_for(&app.pool, common::COORD_USERNAME).await;

        common::factory_session(&app.pool, "sess-expiry-window", "Template A", 60, &coord_id).await;
        common::factory_asset(
            &app.pool,
            "asset-expiry-window",
            "BOOK-EXPIRY-WINDOW",
            "sess-expiry-window",
            &coord_id,
        )
        .await;
        sqlx::query("UPDATE assets SET expires_on = ? WHERE id = 'asset-expiry-window'")
            .bind((Utc::now() + Duration::days(1)).date_naive())
            .execute(&app.pool)
            .await
            .expect("update expiry");

        let rows = service.near_expiry_assets(0).await.expect("near expiry");
        assert!(
            rows.iter().any(|row| row.id == "asset-expiry-window"),
            "within_days=0 should still include assets expiring within one day"
        );
    }
}
