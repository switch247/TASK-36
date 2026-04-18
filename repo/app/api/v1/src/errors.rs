use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};
use rocket::serde::{json::Json, Serialize};

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub code: u16,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct ApiError {
    pub status: Status,
    pub body: ApiErrorBody,
}

impl ApiError {
    pub fn new(status: Status, message: impl Into<String>, details: Option<serde_json::Value>) -> Self {
        Self {
            status,
            body: ApiErrorBody {
                code: status.code,
                message: message.into(),
                details,
            },
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(Status::Unauthorized, message, None)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(Status::Forbidden, message, None)
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(Status::BadRequest, message, None)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(Status::NotFound, message, None)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(Status::Conflict, message, None)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(Status::InternalServerError, message, None)
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        Response::build_from(Json(self.body).respond_to(req)?)
            .status(self.status)
            .ok()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
