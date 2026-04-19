use rocket::http::Status;
use rocket::request::Request;
use rocket::serde::json::Json;
use rocket::{catch, catchers, Catcher};

use crate::errors::{ApiErrorBody, StashedApiError};

fn body_from_request_or_status(req: &Request<'_>, status: Status) -> ApiErrorBody {
    // If the guard stashed a structured body for this request, serve it back.
    // Otherwise fall back to a generic body built from the status.
    let stash = req.local_cache(|| {
        StashedApiError(ApiErrorBody {
            code: status.code,
            message: default_message_for(status).to_string(),
            details: None,
        })
    });
    stash.0.clone()
}

fn default_message_for(status: Status) -> &'static str {
    match status.code {
        400 => "bad request",
        401 => "missing credentials: provide bearer token, session id, or both",
        403 => "forbidden",
        404 => "not found",
        409 => "conflict",
        422 => "invalid payload",
        _ => "request failed",
    }
}

#[catch(400)]
fn bad_request(req: &Request) -> Json<ApiErrorBody> {
    Json(body_from_request_or_status(req, Status::BadRequest))
}

#[catch(401)]
fn unauthorized(req: &Request) -> Json<ApiErrorBody> {
    Json(body_from_request_or_status(req, Status::Unauthorized))
}

#[catch(403)]
fn forbidden(req: &Request) -> Json<ApiErrorBody> {
    Json(body_from_request_or_status(req, Status::Forbidden))
}

#[catch(404)]
fn not_found(req: &Request) -> Json<ApiErrorBody> {
    Json(body_from_request_or_status(req, Status::NotFound))
}

#[catch(409)]
fn conflict(req: &Request) -> Json<ApiErrorBody> {
    Json(body_from_request_or_status(req, Status::Conflict))
}

#[catch(422)]
fn unprocessable(req: &Request) -> Json<ApiErrorBody> {
    Json(body_from_request_or_status(req, Status::UnprocessableEntity))
}

#[catch(500)]
fn internal(req: &Request) -> Json<ApiErrorBody> {
    Json(body_from_request_or_status(req, Status::InternalServerError))
}

#[catch(default)]
fn default_catcher(status: Status, req: &Request) -> Json<ApiErrorBody> {
    Json(body_from_request_or_status(req, status))
}

pub fn catchers_v1() -> Vec<Catcher> {
    catchers![
        bad_request,
        unauthorized,
        forbidden,
        not_found,
        conflict,
        unprocessable,
        internal,
        default_catcher
    ]
}
