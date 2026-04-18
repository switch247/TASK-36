#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use app_api_v1::{
        parse_prompt_datetime, validate_room_capacity, validate_session_duration, ApiError,
    };
    use app_core::types::{ApiActor, UserRole};

    #[test]
    fn room_capacity_validator_returns_structured_details() {
        let err = validate_room_capacity(999).expect_err("capacity must fail");
        assert_eq!(err.status.code, 400);
        assert_eq!(err.body.message, "validation failed");
        assert_eq!(err.body.details.as_ref().unwrap()["field"], "capacity");
    }

    #[test]
    fn session_duration_validator_returns_structured_details() {
        let err = validate_session_duration(5).expect_err("duration must fail");
        assert_eq!(err.status.code, 400);
        assert_eq!(err.body.message, "validation failed");
        assert_eq!(err.body.details.as_ref().unwrap()["field"], "duration_minutes");
    }

    #[test]
    fn parse_prompt_datetime_accepts_prompt_format_and_rejects_iso() {
        let parsed = parse_prompt_datetime("03/27/2026 09:15 AM").expect("prompt datetime");
        assert_eq!(parsed.date(), NaiveDate::from_ymd_opt(2026, 3, 27).unwrap());

        let err = parse_prompt_datetime("2026-03-27T09:15:00Z").expect_err("iso must fail");
        assert_eq!(err.status.code, 400);
        assert_eq!(err.body.message, "datetime must be MM/DD/YYYY hh:mm AM/PM");
    }

    #[test]
    fn api_error_constructors_use_expected_status_codes() {
        let unauthorized = ApiError::unauthorized("missing creds");
        let forbidden = ApiError::forbidden("blocked");
        let not_found = ApiError::not_found("missing");

        assert_eq!(unauthorized.status.code, 401);
        assert_eq!(forbidden.status.code, 403);
        assert_eq!(not_found.status.code, 404);
    }

    #[test]
    fn api_actor_shape_covers_admin_and_non_admin_roles() {
        let admin = ApiActor {
            user_id: Some("user-1".into()),
            role: UserRole::Admin,
            username: Some("admin_local".into()),
        };
        let coord = ApiActor {
            user_id: Some("user-2".into()),
            role: UserRole::Coordinator,
            username: Some("coord_local".into()),
        };

        assert!(matches!(admin.role, UserRole::Admin));
        assert!(matches!(coord.role, UserRole::Coordinator));
    }
}
