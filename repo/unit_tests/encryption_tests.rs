#[cfg(test)]
mod tests {
    use app_core::crypto::{decrypt_dob, encrypt_dob};

    #[test]
    fn encrypt_then_decrypt_roundtrip() {
        let key = [7u8; 32];
        let original = "03/26/2026";
        let encrypted = encrypt_dob(original, &key).expect("encryption should succeed");
        let decrypted = decrypt_dob(&encrypted, &key).expect("decryption should succeed");
        assert_eq!(original, decrypted);
    }

    #[test]
    fn rejects_decryption_with_wrong_key() {
        let key_a = [9u8; 32];
        let key_b = [1u8; 32];
        let encrypted = encrypt_dob("01/15/2001", &key_a).expect("encryption should succeed");
        assert!(decrypt_dob(&encrypted, &key_b).is_err());
    }
}
