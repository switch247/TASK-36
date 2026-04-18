#[cfg(test)]
mod tests {
    use app_core::auth::validate_password_policy;

    #[test]
    fn password_policy_rejects_short() {
        assert!(validate_password_policy("short").is_err());
    }

    #[test]
    fn password_policy_requires_uppercase() {
        assert!(validate_password_policy("alllowercase12!").is_err());
    }

    #[test]
    fn password_policy_requires_lowercase() {
        assert!(validate_password_policy("ALLUPPERCASE12!").is_err());
    }

    #[test]
    fn password_policy_requires_digit() {
        assert!(validate_password_policy("NoDigitsHere!!").is_err());
    }

    #[test]
    fn password_policy_requires_special() {
        assert!(validate_password_policy("NoSpecial1234").is_err());
    }

    #[test]
    fn password_policy_accepts_complex() {
        assert!(validate_password_policy("ComplexPass12!").is_ok());
    }
}
