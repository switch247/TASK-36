use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::errors::CoreError;

pub const MAX_FILES_PER_RECORD: usize = 10;
pub const MAX_FILE_SIZE_BYTES: usize = 25 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureMetadata {
    pub operator: String,
    pub captured_at: DateTime<Utc>,
    pub device_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManifest {
    pub file_name: String,
    pub mime_ext: String,
    pub size_bytes: usize,
    pub fingerprint_sha256: String,
    pub metadata: CaptureMetadata,
}

pub fn validate_file_count(count: usize) -> Result<(), CoreError> {
    if count > MAX_FILES_PER_RECORD {
        return Err(CoreError::FilePolicyViolation(format!(
            "file count {} exceeds {}",
            count, MAX_FILES_PER_RECORD
        )));
    }
    Ok(())
}

pub fn validate_file_size(size_bytes: usize) -> Result<(), CoreError> {
    if size_bytes > MAX_FILE_SIZE_BYTES {
        return Err(CoreError::FilePolicyViolation(format!(
            "file size {} exceeds {} bytes",
            size_bytes, MAX_FILE_SIZE_BYTES
        )));
    }
    Ok(())
}

pub fn validate_extension(ext: &str) -> Result<(), CoreError> {
    const ALLOWED: [&str; 7] = ["jpg", "png", "mp4", "mp3", "pdf", "docx", "jpeg"];
    let normalized = ext.trim().to_lowercase();
    if !ALLOWED.contains(&normalized.as_str()) {
        return Err(CoreError::FilePolicyViolation(format!(
            "extension {} is not allowed",
            ext
        )));
    }
    Ok(())
}

pub fn sha256_fingerprint(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{:x}", digest)
}
