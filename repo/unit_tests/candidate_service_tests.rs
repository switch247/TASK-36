#[cfg(test)]
mod tests {
    use sqlx::mysql::MySqlPoolOptions;

    use app_services::candidate_service::CandidateService;

    fn test_service() -> CandidateService {
        let pool = MySqlPoolOptions::new()
            .connect_lazy("mysql://user:pass@localhost:3306/test_db")
            .expect("lazy pool");
        CandidateService::new(pool, [7u8; 32])
    }

    #[rocket::async_test]
    async fn normalize_dob_mmddyyyy_accepts_valid_input() {
        let service = test_service();
        let normalized = service
            .normalize_dob_mmddyyyy("03/27/2026")
            .expect("valid dob");
        assert_eq!(normalized, "2026-03-27");
    }

    #[rocket::async_test]
    async fn normalize_dob_mmddyyyy_rejects_invalid_calendar_date() {
        let service = test_service();
        let err = service
            .normalize_dob_mmddyyyy("02/30/2026")
            .expect_err("invalid date must fail");
        assert!(err.to_string().contains("normalization error"));
    }

    #[rocket::async_test]
    async fn candidate_service_preserves_configured_aes_key() {
        let service = test_service();
        assert_eq!(service.aes_key, [7u8; 32]);
    }
}
