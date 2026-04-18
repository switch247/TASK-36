use super::*;

#[test]
fn dashboard_view_renders_summary_module() {
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

    let rendered = dashboard_view(summary);
    assert!(rendered.is_ok(), "dashboard summary view should render");
}

#[test]
fn layout_helpers_render_without_router_state() {
    assert!(metric("Candidates", "12".into()).is_ok());
    assert!(spinner().is_ok());
}

#[test]
fn table_components_render_real_frontend_rows() {
    let users = vec![UserRow {
        username: "coord_local".into(),
        role: "Coordinator".into(),
    }];
    let outputs = vec![OutputRow {
        id: "out-1".into(),
        session_id: "sess-1".into(),
        output_type: "SummaryReport".into(),
        mode: "FinalPrint".into(),
        created_at: "2026-03-27T10:00:00".into(),
    }];
    let attachments = vec![AttachmentRow {
        id: "att-1".into(),
        record_type: "candidate".into(),
        record_id: "cand-1".into(),
        file_name: "proof".into(),
        extension: "pdf".into(),
        size_bytes: 128,
        captured_at: "2026-03-27T10:00:00".into(),
    }];

    assert!(table_users(users).is_ok(), "users table should render");
    assert!(table_outputs(outputs).is_ok(), "outputs table should render");
    assert!(table_attachments(attachments).is_ok(), "attachments table should render");
}

#[test]
fn reports_tables_render_real_report_modules() {
    let incidents = vec![IncidentRow {
        session_id: "sess-1".into(),
        avg_incidents: 1.5,
    }];
    let return_rates = vec![ReturnRateRow {
        session_id: "sess-1".into(),
        total_assets: 10,
        returned_assets: 8,
        return_rate_pct: 80.0,
    }];
    let alerts = vec![AlertRow {
        alert_type: "near_expiry".into(),
        severity: "medium".into(),
        session_id: Some("sess-1".into()),
        asset_id: Some("asset-1".into()),
        message: "Asset expires soon".into(),
    }];
    let materials = vec![MaterialInventoryRow {
        asset_id: "asset-1".into(),
        booklet_code: "BOOK-1".into(),
        tracking_status: "Prepared".into(),
        session_id: "sess-1".into(),
        expires_on: Some("2026-04-01".into()),
        incident_count: 0,
    }];

    assert!(table_incidents(incidents).is_ok(), "incident table should render");
    assert!(
        table_return_rates(return_rates).is_ok(),
        "return rate table should render"
    );
    assert!(table_alerts(alerts).is_ok(), "alerts table should render");
    assert!(table_materials_inventory(materials).is_ok(), "inventory table should render");
}

#[test]
fn list_tables_render_domain_rows() {
    let candidates = vec![CandidateRow {
        id: "cand-1".into(),
        scanned_barcode: "BAR-1".into(),
        national_id: "ID-1".into(),
    }];
    let rooms = vec![RoomRow {
        id: "room-1".into(),
        capacity: 40,
        location: "Hall A".into(),
    }];
    let sessions = vec![SessionRow {
        id: "sess-1".into(),
        template_name: "base-template".into(),
        status: "Scheduled".into(),
        duration_minutes: 90,
        starts_at: Some("2026-03-27T09:00:00".into()),
    }];
    let assets = vec![AssetRow {
        id: "asset-1".into(),
        booklet_code: "BOOK-1".into(),
        tracking_status: "Prepared".into(),
        incident_count: 0,
    }];

    assert!(table_candidates(candidates).is_ok());
    assert!(table_rooms(rooms).is_ok());
    assert!(table_sessions(sessions).is_ok());
    assert!(table_assets(assets).is_ok());
}

#[test]
fn formatting_helpers_cover_dashboard_display_logic() {
    assert_eq!(toast_bg(&ToastKind::Success), "bg-emerald-600");

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

#[test]
fn template_table_renders_template_versions() {
    let templates = vec![TemplateRow {
        template_id: "base-template".into(),
        version_no: 2,
        locked_for_final_print: true,
        created_at: "2026-03-27T10:00:00".into(),
    }];

    assert!(table_templates(templates).is_ok(), "template table should render");
}
