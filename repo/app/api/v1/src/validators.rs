use rocket::http::Status;

use crate::errors::{ApiError, ApiResult};

pub fn validate_room_capacity(capacity: i32) -> ApiResult<()> {
    if !(1..=500).contains(&capacity) {
        return Err(ApiError::new(
            Status::BadRequest,
            "validation failed",
            Some(serde_json::json!({
                "field": "capacity",
                "message": "capacity must be between 1 and 500"
            })),
        ));
    }
    Ok(())
}

pub fn validate_session_duration(duration_minutes: i32) -> ApiResult<()> {
    if !(15..=360).contains(&duration_minutes) {
        return Err(ApiError::new(
            Status::BadRequest,
            "validation failed",
            Some(serde_json::json!({
                "field": "duration_minutes",
                "message": "duration_minutes must be between 15 and 360"
            })),
        ));
    }
    Ok(())
}
