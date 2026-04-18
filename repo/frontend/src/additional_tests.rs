use super::*;

#[test]
fn display_formatters_cover_datetime_and_optional_date_paths() {
    assert_eq!(
        format_display_datetime("2026-03-27T13:45:00"),
        "03/27/2026 01:45 PM"
    );
    assert_eq!(format_display_datetime("not-a-datetime"), "not-a-datetime");
    assert_eq!(format_display_date_opt(Some("2026-03-27")), "03/27/2026");
    assert_eq!(format_display_date_opt(None), "N/A");
}

#[test]
fn session_storage_helpers_are_safe_without_browser_storage() {
    let session = LoginResponse {
        session_id: "11111111-1111-1111-1111-111111111111".into(),
        jwt: "header.payload.signature".into(),
        jwt_expires_at: "2030-01-01T00:00:00+00:00".into(),
        session_expires_at: "2030-01-01T00:00:00+00:00".into(),
    };

    save_session(&session);
    assert!(
        load_session().is_none(),
        "native unit tests should not require browser localStorage"
    );
    clear_session();
    assert!(load_session().is_none());
}

#[test]
fn route_enum_covers_remaining_navigation_targets() {
    let _sessions: Route = Route::Sessions {};
    let _assets: Route = Route::Assets {};
    let _templates: Route = Route::Templates {};
    let _admin: Route = Route::Admin {};
    let _not_found: Route = Route::NotFound {
        _route: vec!["missing".into()],
    };
}

#[test]
fn request_models_serialize_expected_frontend_api_contracts() {
    let candidate = CreateCandidateRequest {
        candidate_id: "cand-42".into(),
        date_of_birth: "03/27/2001".into(),
        national_id: "ID-42".into(),
        scanned_barcode: "BAR-42".into(),
        metadata_json: "{\"name\":\"Casey\",\"room_id\":\"room-a\"}".into(),
        template_id: Some("candidate-registration".into()),
    };
    let candidate_json = serde_json::to_value(&candidate).expect("candidate json");
    assert_eq!(candidate_json["candidate_id"], "cand-42");
    assert_eq!(candidate_json["template_id"], "candidate-registration");

    let room = CreateRoomRequest {
        id: "room-42".into(),
        capacity: 50,
        location: "Hall A".into(),
        template_id: Some("room-config".into()),
    };
    let room_json = serde_json::to_value(&room).expect("room json");
    assert_eq!(room_json["capacity"], 50);
    assert_eq!(room_json["location"], "Hall A");

    let user = CreateUserRequest {
        username: "coord_new".into(),
        password: "StrongPass#2026!".into(),
        role: "Coordinator".into(),
        template_id: Some("proctor-profile".into()),
    };
    let user_json = serde_json::to_value(&user).expect("user json");
    assert_eq!(user_json["username"], "coord_new");
    assert_eq!(user_json["role"], "Coordinator");

    let session = CreateSessionRequest {
        id: "sess-42".into(),
        template_name: "Template A".into(),
        duration_minutes: 90,
        status: "Scheduled".into(),
        starts_at: "03/27/2026 09:00 AM".into(),
        ends_at: "03/27/2026 10:30 AM".into(),
    };
    let session_json = serde_json::to_value(&session).expect("session json");
    assert_eq!(session_json["template_name"], "Template A");
    assert_eq!(session_json["duration_minutes"], 90);

    let output = OutputReq {
        session_id: "sess-42".into(),
        mode: "Draft".into(),
        output_type: "AdmitCard".into(),
    };
    let output_json = serde_json::to_value(&output).expect("output json");
    assert_eq!(output_json["mode"], "Draft");
    assert_eq!(output_json["output_type"], "AdmitCard");

    let template = TemplateReq {
        template_id: "base-template".into(),
        version_no: 2,
        snapshot: serde_json::json!({"rules":{"id":["Required"]}}),
        lock_for_final_print: true,
    };
    let template_json = serde_json::to_value(&template).expect("template json");
    assert_eq!(template_json["template_id"], "base-template");
    assert_eq!(template_json["lock_for_final_print"], true);

    let attachment = AttachmentUploadReq {
        record_type: "candidate".into(),
        record_id: "cand-42".into(),
        file_name: "proof".into(),
        extension: "pdf".into(),
        bytes_base64: "UERG".into(),
        operator_label: "scanner-1".into(),
        device_label: "desk-a".into(),
    };
    let attachment_json = serde_json::to_value(&attachment).expect("attachment json");
    assert_eq!(attachment_json["record_type"], "candidate");
    assert_eq!(attachment_json["extension"], "pdf");

    let export = ExportReportRequest {
        report: "incident_rates".into(),
        within_days: Some(30),
        filter: Some("room-a".into()),
        limit: Some(100),
    };
    let export_json = serde_json::to_value(&export).expect("export json");
    assert_eq!(export_json["report"], "incident_rates");
    assert_eq!(export_json["within_days"], 30);

    let message = MessageDraftReq {
        channel: "Email".into(),
        recipient: "candidate@example.test".into(),
        subject: Some("Exam confirmation".into()),
        body: "Your exam is scheduled.".into(),
    };
    let message_json = serde_json::to_value(&message).expect("message json");
    assert_eq!(message_json["channel"], "Email");
    assert_eq!(message_json["recipient"], "candidate@example.test");

    let scan = ScanReq {
        code: "BAR-42".into(),
        intent: "candidate_lookup".into(),
    };
    let scan_json = serde_json::to_value(&scan).expect("scan json");
    assert_eq!(scan_json["code"], "BAR-42");
    assert_eq!(scan_json["intent"], "candidate_lookup");
}

#[test]
fn response_models_deserialize_expected_optional_fields() {
    let scan: ScanResp = serde_json::from_value(serde_json::json!({
        "code": "BAR-42",
        "found": true,
        "candidate_id": "cand-42",
        "asset_id": null,
        "asset_status": null,
        "message": "Candidate located"
    }))
    .expect("scan response");
    assert!(scan.found);
    assert_eq!(scan.candidate_id.as_deref(), Some("cand-42"));
    assert_eq!(scan.asset_id, None);

    let file: AttachmentFileResp = serde_json::from_value(serde_json::json!({
        "file_name": "proof.pdf",
        "bytes_base64": "UERG"
    }))
    .expect("attachment file response");
    assert_eq!(file.file_name, "proof.pdf");
    assert_eq!(file.bytes_base64, "UERG");

    let export: ExportResponse = serde_json::from_value(serde_json::json!({
        "content": "session_id\tavg_incidents"
    }))
    .expect("export response");
    assert!(export.content.contains("session_id"));
}

#[test]
fn display_formatters_cover_more_frontend_edge_cases() {
    assert_eq!(
        format_display_datetime("2026-03-27 13:45:00 UTC"),
        "03/27/2026 01:45 PM"
    );
    assert_eq!(format_display_datetime("N/A"), "N/A");
    assert_eq!(format_display_date_opt(Some("")), "N/A");
    assert_eq!(format_display_date_opt(Some("not-a-date")), "not-a-date");
}
