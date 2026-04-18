use aes_gcm::{
    aead::{rand_core::RngCore, Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;

use crate::errors::CoreError;

pub fn encrypt_dob(plain: &str, key_bytes: &[u8; 32]) -> Result<String, CoreError> {
    let cipher = Aes256Gcm::new_from_slice(key_bytes).map_err(|_| CoreError::EncryptionFailure)?;
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let nonce_obj = Nonce::from_slice(&nonce);

    let ciphertext = cipher
        .encrypt(nonce_obj, plain.as_bytes())
        .map_err(|_| CoreError::EncryptionFailure)?;

    let mut payload = nonce.to_vec();
    payload.extend(ciphertext);
    Ok(B64.encode(payload))
}

pub fn decrypt_dob(enc: &str, key_bytes: &[u8; 32]) -> Result<String, CoreError> {
    let decoded = B64.decode(enc).map_err(|_| CoreError::DecryptionFailure)?;
    if decoded.len() < 13 {
        return Err(CoreError::DecryptionFailure);
    }
    let (nonce, ciphertext) = decoded.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(key_bytes).map_err(|_| CoreError::DecryptionFailure)?;
    let plain = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| CoreError::DecryptionFailure)?;
    String::from_utf8(plain).map_err(|_| CoreError::DecryptionFailure)
}
