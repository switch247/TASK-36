#[path = "../API_tests/common.rs"]
mod common;

#[cfg(test)]
mod tests {
    use app_services::cleansing_service::CleansingService;

    #[rocket::async_test]
    async fn validate_zip_city_uses_seeded_reference_data() {
        let app = common::setup_app().await.expect("setup");
        let service = CleansingService::new(app.pool.clone());

        assert!(service.validate_zip_city("00100", "Nairobi").await.expect("match"));
        assert!(!service.validate_zip_city("00100", "Kisumu").await.expect("mismatch"));
        assert!(!service.validate_zip_city("99999", "Unknown").await.expect("missing"));
    }

    #[test]
    fn normalize_record_combines_unit_currency_and_date_normalization() {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .connect_lazy("mysql://user:pass@localhost:3306/test_db")
            .expect("lazy pool");
        let service = CleansingService::new(pool);

        let normalized = service
            .normalize_record(2.0, "km", 100.0, "KES", 0.0078, "03/26/2026")
            .expect("normalize record");

        assert_eq!(normalized.normalized_value, 2000.0);
        assert_eq!(normalized.normalized_unit, "m");
        assert_eq!(normalized.normalized_amount_usd, "USD 0.78");
        assert_eq!(normalized.normalized_date, "2026-03-26");
    }

    #[test]
    fn cleansing_service_edge_cases_are_enforced() {
        assert!(CleansingService::is_room_capacity_outlier(500, &[50, 60, 40]).expect("outlier"));
        assert!(CleansingService::is_room_capacity_outlier(100, &[]).is_err());
        assert_eq!(
            CleansingService::parse_dob("03/27/2026").expect("dob").to_string(),
            "2026-03-27"
        );
        assert!(CleansingService::parse_dob("not-a-date").is_err());
    }

    #[test]
    fn normalize_record_rejects_invalid_units_currency_and_dates() {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .connect_lazy("mysql://user:pass@localhost:3306/test_db")
            .expect("lazy pool");
        let service = CleansingService::new(pool);

        assert!(service
            .normalize_record(2.0, "yards?", 100.0, "KES", 0.0078, "03/26/2026")
            .is_err());
        assert!(service
            .normalize_record(2.0, "km", 100.0, "??", 0.0078, "03/26/2026")
            .is_err());
        assert!(service
            .normalize_record(2.0, "km", 100.0, "KES", 0.0078, "2026-03-26")
            .is_err());
    }
}
