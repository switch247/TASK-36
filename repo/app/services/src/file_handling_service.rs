use anyhow::Result;
use chrono::Utc;

use app_core::file_policy::{
    sha256_fingerprint, validate_extension, validate_file_count, validate_file_size, CaptureMetadata,
    FileManifest,
};

pub struct IncomingFile<'a> {
    pub file_name: &'a str,
    pub extension: &'a str,
    pub bytes: &'a [u8],
}

pub struct FileHandlingService;

impl FileHandlingService {
    pub fn validate_and_manifest(
        files: &[IncomingFile<'_>],
        operator: &str,
        device_label: &str,
    ) -> Result<Vec<FileManifest>> {
        validate_file_count(files.len())?;

        let now = Utc::now();
        let mut manifests = Vec::with_capacity(files.len());

        for file in files {
            validate_file_size(file.bytes.len())?;
            validate_extension(file.extension)?;

            manifests.push(FileManifest {
                file_name: file.file_name.to_string(),
                mime_ext: file.extension.to_string(),
                size_bytes: file.bytes.len(),
                fingerprint_sha256: sha256_fingerprint(file.bytes),
                metadata: CaptureMetadata {
                    operator: operator.to_string(),
                    captured_at: now,
                    device_label: device_label.to_string(),
                },
            });
        }

        Ok(manifests)
    }
}
