use frontend::{toast_bg, trend_points, DashboardSummary, ToastKind};

#[test]
fn helper_exports_are_available_to_frontend_integration_tests() {
    assert_eq!(toast_bg(&ToastKind::Success), "bg-emerald-600");
    assert_eq!(toast_bg(&ToastKind::Error), "bg-rose-600");
    assert_eq!(toast_bg(&ToastKind::Info), "bg-slate-700");

    let summary = DashboardSummary {
        total_candidates: 1,
        total_rooms: 1,
        total_sessions_this_week: 1,
        seat_utilization_count: 2,
        near_expiry_count: 3,
        incident_rate_count: 4,
        upcoming_sessions: Vec::new(),
        recent_outputs: Vec::new(),
    };
    let points = trend_points(&summary);
    assert!(points.starts_with("0,"));
    assert_eq!(points.split(' ').count(), 6);
}
