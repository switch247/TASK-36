use chrono::{DateTime, Duration, Utc};

pub const SESSION_IDLE_MINUTES: i64 = 30;
pub const LOCKOUT_ATTEMPTS: i32 = 5;
pub const LOCKOUT_MINUTES: i64 = 15;

pub fn calculate_session_expiry(now: DateTime<Utc>) -> DateTime<Utc> {
    now + Duration::minutes(SESSION_IDLE_MINUTES)
}

pub fn is_session_active(last_activity: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    now.signed_duration_since(last_activity) < Duration::minutes(SESSION_IDLE_MINUTES)
}

pub fn lockout_until(now: DateTime<Utc>) -> DateTime<Utc> {
    now + Duration::minutes(LOCKOUT_MINUTES)
}
