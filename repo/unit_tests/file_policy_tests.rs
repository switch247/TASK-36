#[cfg(test)]
mod tests {
    use app_core::file_policy::{
        sha256_fingerprint, validate_extension, validate_file_count, validate_file_size,
        MAX_FILE_SIZE_BYTES, MAX_FILES_PER_RECORD,
    };

    #[test]
    fn file_policy_accepts_boundary_values() {
        assert!(validate_file_count(MAX_FILES_PER_RECORD).is_ok());
        assert!(validate_file_size(MAX_FILE_SIZE_BYTES).is_ok());
        assert!(validate_extension("pdf").is_ok());
        assert!(validate_extension("JPEG").is_ok());
    }

    #[test]
    fn file_policy_rejects_values_beyond_limits() {
        assert!(validate_file_count(MAX_FILES_PER_RECORD + 1).is_err());
        assert!(validate_file_size(MAX_FILE_SIZE_BYTES + 1).is_err());
        assert!(validate_extension("exe").is_err());
    }

    #[test]
    fn sha256_fingerprint_is_stable() {
        let a = sha256_fingerprint(b"same-bytes");
        let b = sha256_fingerprint(b"same-bytes");
        let c = sha256_fingerprint(b"different-bytes");

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 64);
    }
}
