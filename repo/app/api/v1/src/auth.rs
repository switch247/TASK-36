use rocket::serde::json::Json;
use rocket::{post, State};

use app_core::errors::CoreError;
use app_services::auth_service::AuthService;

use crate::errors::{ApiError, ApiResult};

#[derive(Debug, serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, serde::Serialize)]
pub struct LoginResponse {
    pub session_id: String,
    pub session_expires_at: String,
    pub jwt: String,
    pub jwt_expires_at: String,
}

#[post("/auth/login", data = "<payload>")]
pub async fn login(
    payload: Json<LoginRequest>,
    auth_service: &State<AuthService>,
) -> ApiResult<Json<LoginResponse>> {
    let result = auth_service
        .authenticate(&payload.username, &payload.password)
        .await
        .map_err(|err| {
            tracing::error!(error = %err, "login authentication failed");
            match err.downcast_ref::<CoreError>() {
                Some(CoreError::AccountLocked(until)) => {
                    ApiError::unauthorized(format!("account locked until {until}"))
                }
                _ => ApiError::unauthorized("authentication failed"),
            }
        })?;

    Ok(Json(LoginResponse {
        session_id: result.session_id.to_string(),
        session_expires_at: result.session_expires_at.to_rfc3339(),
        jwt: result.jwt,
        jwt_expires_at: result.jwt_expires_at.to_rfc3339(),
    }))
}
