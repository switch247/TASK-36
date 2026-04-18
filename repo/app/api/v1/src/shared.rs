use chrono::NaiveDateTime;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};

use app_core::types::ApiActor;
use app_services::audit_service::AuditService;
use app_services::auth_service::AuthService;

use crate::errors::{ApiError, ApiResult};

pub struct ApiContext {
    pub actor: ApiActor,
    pub ip_address: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiContext {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let authz = req.headers().get_one("Authorization");
        let session_id = req.headers().get_one("x-session-id");

        let Some(auth_service) = req.rocket().state::<AuthService>() else {
            return Outcome::Error((Status::InternalServerError, ApiError::internal("auth service unavailable")));
        };

        let actor = match (authz, session_id) {
            (Some(authz), Some(session_id)) => {
                let Some(token) = authz.strip_prefix("Bearer ").map(str::trim) else {
                    return Outcome::Error((Status::Unauthorized, ApiError::unauthorized("invalid authorization scheme")));
                };
                match auth_service.validate_actor(token, session_id).await {
                    Ok(actor) => actor,
                    Err(_) => return Outcome::Error((Status::Unauthorized, ApiError::unauthorized("invalid token or session"))),
                }
            }
            (Some(authz), None) => {
                let Some(token) = authz.strip_prefix("Bearer ").map(str::trim) else {
                    return Outcome::Error((Status::Unauthorized, ApiError::unauthorized("invalid authorization scheme")));
                };
                match auth_service.validate_actor_jwt_only(token) {
                    Ok(actor) => actor,
                    Err(_) => return Outcome::Error((Status::Unauthorized, ApiError::unauthorized("invalid token"))),
                }
            }
            (None, Some(session_id)) => match auth_service.validate_actor_session_only(session_id).await {
                Ok(actor) => actor,
                Err(_) => return Outcome::Error((Status::Unauthorized, ApiError::unauthorized("invalid session"))),
            },
            (None, None) => {
                return Outcome::Error((
                    Status::Unauthorized,
                    ApiError::unauthorized("missing credentials: provide bearer token, session id, or both"),
                ));
            }
        };

        let ip_address = req
            .client_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Outcome::Success(ApiContext { actor, ip_address })
    }
}

pub async fn audit(audit_service: &State<AuditService>, ctx: &ApiContext, action: &str, resource: &str) {
    let _ = audit_service
        .record_api_call(
            ctx.actor.user_id.as_deref(),
            action,
            resource,
            &ctx.ip_address,
        )
        .await;
}

pub fn actor_user_id(ctx: &ApiContext) -> ApiResult<&str> {
    ctx.actor
        .user_id
        .as_deref()
        .ok_or_else(|| ApiError::unauthorized("missing actor user id in token claims"))
}

pub fn parse_prompt_datetime(value: &str) -> ApiResult<NaiveDateTime> {
    NaiveDateTime::parse_from_str(value, "%m/%d/%Y %I:%M %p")
        .map_err(|_| ApiError::bad_request("datetime must be MM/DD/YYYY hh:mm AM/PM"))
}

pub fn is_admin(ctx: &ApiContext) -> bool {
    matches!(ctx.actor.role, app_core::types::UserRole::Admin)
}
