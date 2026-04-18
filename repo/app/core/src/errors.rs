use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("authentication failed")]
    AuthenticationFailed,
    #[error("password policy violation")]
    PasswordPolicyViolation,
    #[error("account locked until {0}")]
    AccountLocked(String),
    #[error("token generation failed")]
    TokenGenerationFailed,
    #[error("invalid token")]
    InvalidToken,
    #[error("session invalid or expired")]
    SessionInvalid,
    #[error("encryption failure")]
    EncryptionFailure,
    #[error("decryption failure")]
    DecryptionFailure,
    #[error("normalization error: {0}")]
    NormalizationError(String),
    #[error("file policy violation: {0}")]
    FilePolicyViolation(String),
    #[error("template validation error: {0}")]
    TemplateValidationError(String),
    #[error("authorization denied")]
    AuthorizationDenied,
}
