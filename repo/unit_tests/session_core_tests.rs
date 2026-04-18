#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};

    use app_core::session::{calculate_session_expiry, is_session_active, lockout_until, LOCKOUT_MINUTES, SESSION_IDLE_MINUTES};

    #[test]
    fn calculate_session_expiry_uses_idle_window() {
        let now = Utc.with_ymd_and_hms(2026, 4, 18, 9, 0, 0).unwrap();
        let expiry = calculate_session_expiry(now);
        assert_eq!(expiry, now + Duration::minutes(SESSION_IDLE_MINUTES));
    }

    #[test]
    fn is_session_active_flips_after_idle_boundary() {
        let now = Utc.with_ymd_and_hms(2026, 4, 18, 9, 0, 0).unwrap();
        assert!(is_session_active(now - Duration::minutes(SESSION_IDLE_MINUTES - 1), now));
        assert!(!is_session_active(now - Duration::minutes(SESSION_IDLE_MINUTES), now));
    }

    #[test]
    fn lockout_until_uses_lockout_window() {
        let now = Utc.with_ymd_and_hms(2026, 4, 18, 9, 0, 0).unwrap();
        assert_eq!(lockout_until(now), now + Duration::minutes(LOCKOUT_MINUTES));
    }
}
