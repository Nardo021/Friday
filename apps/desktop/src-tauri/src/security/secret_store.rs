use base64::Engine;
use keyring::Entry;
use rand::RngCore;

use crate::errors::{AppError, AppResult};

const SERVICE: &str = "Friday";
const CURSOR_API_KEY_ACCOUNT: &str = "cursor_api_key";
const DATA_KEY_ACCOUNT: &str = "friday_data_key";
const STT_API_KEY_ACCOUNT: &str = "stt_api_key";

pub struct SecretStore;
impl SecretStore {
    pub fn save_cursor_api_key(key: &str) -> AppResult<()> {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            return Err(AppError::Other("API key cannot be empty".into()));
        }

        let entry = Entry::new(SERVICE, CURSOR_API_KEY_ACCOUNT)
            .map_err(|e| AppError::Other(format!("secure storage unavailable: {e}")))?;
        entry
            .set_password(trimmed)
            .map_err(|e| AppError::Other(format!("failed to store API key securely: {e}")))?;
        Ok(())
    }

    pub fn get_cursor_api_key() -> AppResult<Option<String>> {
        let entry = Entry::new(SERVICE, CURSOR_API_KEY_ACCOUNT)
            .map_err(|e| AppError::Other(format!("secure storage unavailable: {e}")))?;
        match entry.get_password() {
            Ok(value) if !value.trim().is_empty() => Ok(Some(value)),
            Ok(_) => Ok(None),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Other(format!("failed to read API key: {e}"))),
        }
    }

    pub fn has_cursor_api_key() -> AppResult<bool> {
        Ok(Self::get_cursor_api_key()?.is_some())
    }

    pub fn clear_cursor_api_key() -> AppResult<()> {
        let entry = Entry::new(SERVICE, CURSOR_API_KEY_ACCOUNT)
            .map_err(|e| AppError::Other(format!("secure storage unavailable: {e}")))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::Other(format!("failed to remove API key: {e}"))),
        }
    }

    pub fn save_stt_api_key(key: &str) -> AppResult<()> {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            return Err(AppError::Other("STT API key cannot be empty".into()));
        }
        let entry = Entry::new(SERVICE, STT_API_KEY_ACCOUNT)
            .map_err(|e| AppError::Other(format!("secure storage unavailable: {e}")))?;
        entry
            .set_password(trimmed)
            .map_err(|e| AppError::Other(format!("failed to store STT API key: {e}")))?;
        Ok(())
    }

    pub fn get_stt_api_key() -> AppResult<Option<String>> {
        let entry = Entry::new(SERVICE, STT_API_KEY_ACCOUNT)
            .map_err(|e| AppError::Other(format!("secure storage unavailable: {e}")))?;
        match entry.get_password() {
            Ok(value) if !value.trim().is_empty() => Ok(Some(value)),
            Ok(_) => Ok(None),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Other(format!("failed to read STT API key: {e}"))),
        }
    }

    pub fn clear_stt_api_key() -> AppResult<()> {
        let entry = Entry::new(SERVICE, STT_API_KEY_ACCOUNT)
            .map_err(|e| AppError::Other(format!("secure storage unavailable: {e}")))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::Other(format!("failed to remove STT API key: {e}"))),
        }
    }

    pub fn has_stt_api_key() -> AppResult<bool> {
        Ok(Self::get_stt_api_key()?.is_some())
    }

    pub fn get_or_create_data_key() -> AppResult<[u8; 32]> {
        if let Some(key) = Self::read_data_key()? {
            return Ok(key);
        }

        let mut key = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);
        Self::save_data_key(&key)?;
        Ok(key)
    }

    pub fn clear_data_key() -> AppResult<()> {
        let entry = Entry::new(SERVICE, DATA_KEY_ACCOUNT)
            .map_err(|e| AppError::Other(format!("secure storage unavailable: {e}")))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::Other(format!("failed to remove data key: {e}"))),
        }
    }

    pub fn clear_all() -> AppResult<()> {
        Self::clear_cursor_api_key()?;
        Self::clear_stt_api_key()?;
        Self::clear_data_key()?;
        Ok(())
    }

    fn save_data_key(key: &[u8; 32]) -> AppResult<()> {
        let encoded = base64::engine::general_purpose::STANDARD.encode(key);
        let entry = Entry::new(SERVICE, DATA_KEY_ACCOUNT)
            .map_err(|e| AppError::Other(format!("secure storage unavailable: {e}")))?;
        entry
            .set_password(&encoded)
            .map_err(|e| AppError::Other(format!("failed to store data key: {e}")))?;
        Ok(())
    }

    fn read_data_key() -> AppResult<Option<[u8; 32]>> {
        let entry = Entry::new(SERVICE, DATA_KEY_ACCOUNT)
            .map_err(|e| AppError::Other(format!("secure storage unavailable: {e}")))?;
        match entry.get_password() {
            Ok(value) => {
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(value.trim())
                    .map_err(|e| AppError::Other(format!("invalid data key encoding: {e}")))?;
                if bytes.len() != 32 {
                    return Err(AppError::Other("invalid data key length".into()));
                }
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes);
                Ok(Some(key))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Other(format!("failed to read data key: {e}"))),
        }
    }
}