use frontend::{
    dashboard_view, metric, spinner, table_alerts, table_assets, table_attachments,
    table_candidates, table_materials_inventory, table_outputs, table_rooms, table_sessions,
    table_users, AlertRow, AssetRow, AttachmentRow, CandidateRow, DashboardSummary,
    MaterialInventoryRow, OutputRow, RecentOutput, RoomRow, SessionRow, UpcomingSession, UserRow,
};

#[test]
fn dashboard_components_render_from_library_exports() {
    let summary = DashboardSummary {
        total_candidates: 12,
        total_rooms: 4,
        total_sessions_this_week: 3,
        seat_utilization_count: 2,
        near_expiry_count: 1,
        incident_rate_count: 1,
        upcoming_sessions: vec![UpcomingSession {
            id: "sess-1".into(),
            template_name: "base-template".into(),
            status: "Scheduled".into(),
            starts_at: "2026-03-27T09:00:00".into(),
        }],
        recent_outputs: vec![RecentOutput {
            id: "out-1".into(),
            output_type: "AdmitCard".into(),
            mode: "FinalPrint".into(),
            created_at: "2026-03-27T10:00:00".into(),
        }],
    };

    assert!(dashboard_view(summary).is_ok());
    assert!(metric("Candidates", "12".into()).is_ok());
    assert!(spinner().is_ok());
}

#[test]
fn frontend_domain_tables_render_from_integration_test() {
    assert!(table_candidates(vec![CandidateRow {
        id: "cand-1".into(),
        scanned_barcode: "BAR-1".into(),
        national_id: "ID-1".into(),
    }])
    .is_ok());
    assert!(table_rooms(vec![RoomRow {
        id: "room-1".into(),
        capacity: 40,
        location: "Hall A".into(),
    }])
    .is_ok());
    assert!(table_sessions(vec![SessionRow {
        id: "sess-1".into(),
        template_name: "base-template".into(),
        status: "Scheduled".into(),
        duration_minutes: 90,
        starts_at: Some("2026-03-27T09:00:00".into()),
    }])
    .is_ok());
    assert!(table_assets(vec![AssetRow {
        id: "asset-1".into(),
        booklet_code: "BOOK-1".into(),
        tracking_status: "Prepared".into(),
        incident_count: 0,
    }])
    .is_ok());
}

#[test]
fn frontend_reporting_and_admin_tables_render_from_integration_test() {
    assert!(table_users(vec![UserRow {
        username: "coord_local".into(),
        role: "Coordinator".into(),
    }])
    .is_ok());
    assert!(table_outputs(vec![OutputRow {
        id: "out-1".into(),
        session_id: "sess-1".into(),
        output_type: "SummaryReport".into(),
        mode: "FinalPrint".into(),
        created_at: "2026-03-27T10:00:00".into(),
    }])
    .is_ok());
    assert!(table_attachments(vec![AttachmentRow {
        id: "att-1".into(),
        record_type: "candidate".into(),
        record_id: "cand-1".into(),
        file_name: "proof".into(),
        extension: "pdf".into(),
        size_bytes: 128,
        captured_at: "2026-03-27T10:00:00".into(),
    }])
    .is_ok());
    assert!(table_alerts(vec![AlertRow {
        alert_type: "near_expiry".into(),
        severity: "medium".into(),
        session_id: Some("sess-1".into()),
        asset_id: Some("asset-1".into()),
        message: "Asset expires soon".into(),
    }])
    .is_ok());
    assert!(table_materials_inventory(vec![MaterialInventoryRow {
        asset_id: "asset-1".into(),
        booklet_code: "BOOK-1".into(),
        tracking_status: "Prepared".into(),
        session_id: "sess-1".into(),
        expires_on: Some("2026-04-01".into()),
        incident_count: 0,
    }])
    .is_ok());
}
