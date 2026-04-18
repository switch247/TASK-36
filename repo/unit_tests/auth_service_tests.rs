#[cfg(test)]
mod tests {
    use bcrypt::verify;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use sqlx::mysql::MySqlPoolOptions;

    use app_core::errors::CoreError;
    use app_core::types::{Claims, UserRole};
    use app_services::auth_service::AuthService;

    fn test_service(secret: &str) -> AuthService {
        let pool = MySqlPoolOptions::new()
            .connect_lazy("mysql://user:pass@localhost:3306/test_db")
            .expect("lazy pool");
        AuthService::new(pool, secret.to_string())
    }

    #[rocket::async_test]
    async fn hash_password_accepts_strong_password_and_rejects_weak_password() {
        let service = test_service("unit-test-secret");

        let hashed = service
            .hash_password("StrongPass#2026!")
            .expect("strong password should hash");
        assert!(verify("StrongPass#2026!", &hashed).expect("bcrypt verify"));

        let err = service
            .hash_password("weak")
            .expect_err("weak password must fail");
        let msg = err.to_string();
        assert!(msg.contains(&CoreError::PasswordPolicyViolation.to_string()));
    }

    #[rocket::async_test]
    async fn validate_actor_jwt_only_returns_claims_actor() {
        let service = test_service("jwt-secret-123");
        let claims = Claims {
            sub: "user-123".into(),
            role: UserRole::Coordinator,
            iat: 1_700_000_000,
            exp: 4_100_000_000,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("jwt-secret-123".as_bytes()),
        )
        .expect("encode token");

        let actor = service
            .validate_actor_jwt_only(&token)
            .expect("token should decode");

        assert_eq!(actor.user_id.as_deref(), Some("user-123"));
        assert!(matches!(actor.role, UserRole::Coordinator));
    }

    #[rocket::async_test]
    async fn validate_actor_jwt_only_rejects_invalid_token() {
        let service = test_service("jwt-secret-123");
        let err = service
            .validate_actor_jwt_only("not-a-real-jwt")
            .expect_err("invalid token must fail");
        assert!(err
            .to_string()
            .contains(&CoreError::InvalidToken.to_string()));
    }

    #[test]
    fn generate_data_key_returns_random_32_byte_keys() {
        let key_a = AuthService::generate_data_key();
        let key_b = AuthService::generate_data_key();

        assert_eq!(key_a.len(), 32);
        assert_eq!(key_b.len(), 32);
        assert_ne!(
            key_a, key_b,
            "generated keys should not repeat deterministically"
        );
    }
}
