#[cfg(test)]
mod tests {
    use base64::Engine;
    use serde_json::json;

    use app_services::output_service::OutputService;

    #[test]
    fn mask_sensitive_id_keeps_last_four_characters() {
        assert_eq!(OutputService::mask_sensitive_id("ABC123456"), "****3456");
        assert_eq!(OutputService::mask_sensitive_id("1234"), "****");
    }

    #[test]
    fn export_csv_whitelisted_emits_header_and_rows() {
        let rows = vec![
            json!({"session_id":"sess-1","count":2}),
            json!({"session_id":"sess-2","count":3}),
        ];

        let csv = OutputService::export_csv_whitelisted(&rows, &["session_id", "count"])
            .expect("csv export should succeed");

        assert!(csv.starts_with("session_id,count\n"));
        assert!(csv.contains("\"sess-1\",2"));
        assert!(csv.contains("\"sess-2\",3"));
    }

    #[test]
    fn export_excel_like_tsv_returns_excel_data_uri() {
        let rows = vec![json!({"session_id":"sess-1","title":"A&B <Report>"})];

        let data_uri = OutputService::export_excel_like_tsv(&rows, &["session_id", "title"])
            .expect("excel export should succeed");

        assert!(data_uri.starts_with("data:application/vnd.ms-excel;base64,"));
        let encoded = data_uri
            .strip_prefix("data:application/vnd.ms-excel;base64,")
            .expect("excel prefix");
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .expect("base64 decode");
        let xml = String::from_utf8(decoded).expect("utf8 xml");
        assert!(xml.contains("sess-1"));
        assert!(xml.contains("A&amp;B &lt;Report&gt;"));
    }

    #[test]
    fn export_pdf_placeholder_returns_pdf_data_uri() {
        let data_uri = OutputService::export_pdf_placeholder("Exam Summary", "Generated body");
        assert!(data_uri.starts_with("data:application/pdf;base64,"));
        let encoded = data_uri
            .strip_prefix("data:application/pdf;base64,")
            .expect("pdf prefix");
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .expect("base64 decode");
        assert!(decoded.starts_with(b"%PDF-1.4\n"));
    }
}
