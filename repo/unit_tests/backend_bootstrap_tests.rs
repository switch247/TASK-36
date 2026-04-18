mod backend_main {
    include!("../backend/src/main.rs");

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn build_cors_accepts_default_origin_configuration() {
            std::env::remove_var("ROCKET_CORS_ORIGINS");
            let cors = build_cors();
            assert!(cors.is_ok(), "default CORS construction should succeed");
        }

        #[test]
        fn build_cors_accepts_custom_origin_list() {
            std::env::set_var(
                "ROCKET_CORS_ORIGINS",
                "http://localhost:8080,http://127.0.0.1:8080,http://frontend:8080",
            );
            let cors = build_cors();
            std::env::remove_var("ROCKET_CORS_ORIGINS");
            assert!(cors.is_ok(), "custom CORS origin lists should be accepted");
        }

        #[test]
        fn init_tracing_is_safe_to_call_in_test_process() {
            init_tracing();
        }
    }
}
