use base64::Engine;
use frontend::{jwt_role, LoginResponse};

#[test]
fn jwt_role_extracts_role_from_frontend_login_model() {
    let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(br#"{"alg":"none"}"#);
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(br#"{"sub":"user-1","role":"Coordinator"}"#);
    let jwt = format!("{header}.{payload}.signature");
    let session = LoginResponse {
        session_id: "11111111-1111-1111-1111-111111111111".into(),
        jwt,
        session_expires_at: "2030-01-01T00:00:00+00:00".into(),
        jwt_expires_at: "2030-01-01T00:00:00+00:00".into(),
    };

    assert_eq!(jwt_role(&session), "Coordinator");
}

#[test]
fn jwt_role_falls_back_to_auditor_for_invalid_token_shape() {
    let session = LoginResponse {
        session_id: "11111111-1111-1111-1111-111111111111".into(),
        jwt: "not-a-jwt".into(),
        session_expires_at: "2030-01-01T00:00:00+00:00".into(),
        jwt_expires_at: "2030-01-01T00:00:00+00:00".into(),
    };

    assert_eq!(jwt_role(&session), "Auditor");
}
