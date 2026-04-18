use regex::Regex;

use crate::errors::CoreError;

#[derive(Debug, Clone)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digit: bool,
    pub require_special: bool,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: true,
        }
    }
}

pub fn validate_password_policy(password: &str) -> Result<(), CoreError> {
    let policy = PasswordPolicy::default();
    if password.len() < policy.min_length {
        return Err(CoreError::PasswordPolicyViolation);
    }

    if policy.require_uppercase && !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(CoreError::PasswordPolicyViolation);
    }

    if policy.require_lowercase && !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err(CoreError::PasswordPolicyViolation);
    }

    if policy.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(CoreError::PasswordPolicyViolation);
    }

    if policy.require_special {
        let re = Regex::new(r#"[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>/?]"#)
            .map_err(|_| CoreError::PasswordPolicyViolation)?;
        if !re.is_match(password) {
            return Err(CoreError::PasswordPolicyViolation);
        }
    }

    Ok(())
}
