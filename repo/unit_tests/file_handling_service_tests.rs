#[cfg(test)]
mod tests {
    use app_services::file_handling_service::{FileHandlingService, IncomingFile};

    #[test]
    fn validate_and_manifest_returns_expected_metadata() {
        let files = vec![IncomingFile {
            file_name: "evidence",
            extension: "PDF",
            bytes: b"proof-bytes",
        }];

        let manifests =
            FileHandlingService::validate_and_manifest(&files, "operator-1", "scanner-a")
                .expect("manifest generation should succeed");

        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].file_name, "evidence");
        assert_eq!(manifests[0].mime_ext, "PDF");
        assert_eq!(manifests[0].size_bytes, b"proof-bytes".len());
        assert_eq!(manifests[0].metadata.operator, "operator-1");
        assert_eq!(manifests[0].metadata.device_label, "scanner-a");
        assert_eq!(manifests[0].fingerprint_sha256.len(), 64);
    }

    #[test]
    fn validate_and_manifest_rejects_invalid_extension() {
        let files = vec![IncomingFile {
            file_name: "script",
            extension: "exe",
            bytes: b"binary",
        }];

        let err = FileHandlingService::validate_and_manifest(&files, "operator-1", "scanner-a")
            .expect_err("invalid extension must fail");
        let msg = err.to_string();
        assert!(msg.contains("extension"));
        assert!(msg.contains("not allowed"));
    }

    #[test]
    fn validate_and_manifest_rejects_too_many_files() {
        let files: Vec<_> = (0..11)
            .map(|_| IncomingFile {
                file_name: "proof",
                extension: "pdf",
                bytes: b"x",
            })
            .collect();

        let err = FileHandlingService::validate_and_manifest(&files, "operator-1", "scanner-a")
            .expect_err("file count policy must fail");
        assert!(err.to_string().contains("file count"));
    }
}
