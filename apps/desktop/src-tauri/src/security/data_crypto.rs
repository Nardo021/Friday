use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::Engine;
use rand::RngCore;

use crate::errors::{AppError, AppResult};
use crate::security::SecretStore;

const PREFIX: &str = "enc:v1:";
const NONCE_LEN: usize = 12;

pub struct DataCrypto;

impl DataCrypto {
    /// Encrypts text for SQLite storage. Falls back to plaintext if the data key is unavailable.
    pub fn encrypt(plaintext: &str) -> AppResult<String> {
        let key = match SecretStore::get_or_create_data_key() {
            Ok(key) => key,
            Err(_) => return Ok(plaintext.to_string()),
        };

        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| AppError::Other(format!("encryption init failed: {e}")))?;

        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| AppError::Other(format!("encryption failed: {e}")))?;

        let mut payload = nonce_bytes.to_vec();
        payload.extend(ciphertext);
        Ok(format!(
            "{PREFIX}{}",
            base64::engine::general_purpose::STANDARD.encode(payload)
        ))
    }

    /// Decrypts stored text. Plaintext rows (legacy) pass through unchanged.
    pub fn decrypt(stored: &str) -> AppResult<String> {
        if !stored.starts_with(PREFIX) {
            return Ok(stored.to_string());
        }

        let key = SecretStore::get_or_create_data_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| AppError::Other(format!("decryption init failed: {e}")))?;

        let payload = base64::engine::general_purpose::STANDARD
            .decode(stored.trim_start_matches(PREFIX))
            .map_err(|e| AppError::Other(format!("invalid encrypted payload: {e}")))?;

        if payload.len() <= NONCE_LEN {
            return Err(AppError::Other("encrypted payload too short".into()));
        }

        let (nonce_bytes, ciphertext) = payload.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| AppError::Other(format!("decryption failed: {e}")))?;

        String::from_utf8(plaintext)
            .map_err(|e| AppError::Other(format!("invalid decrypted utf-8: {e}")))
    }
}
