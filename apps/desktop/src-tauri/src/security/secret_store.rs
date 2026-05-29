use base64::Engine;
use keyring::Entry;
use rand::RngCore;

use crate::errors::{AppError, AppResult};
use crate::storage::secret_sqlite;

const SERVICE: &str = "Friday";
pub const CURSOR_API_KEY_ACCOUNT: &str = "cursor_api_key";
const DATA_KEY_ACCOUNT: &str = "friday_data_key";
const STT_API_KEY_ACCOUNT: &str = "stt_api_key";

pub struct SecretStore;
impl SecretStore {
    /// Reject OpenAI-style keys (`sk-…`) so onboarding stores Cursor Cloud Agent keys only.
    pub fn validate_cursor_api_key(key: &str) -> AppResult<()> {
        let trimmed = key.trim();
        if trimmed.starts_with("sk-") {
            return Err(AppError::Other(
                "This looks like an OpenAI API key. Create a Cursor API key in \
                 cursor.com/dashboard → Integrations → API Keys (not an OpenAI key)."
                    .into(),
            ));
        }
        Ok(())
    }

    fn keyring_entry(account: &str) -> AppResult<Entry> {
        Entry::new(SERVICE, account)
            .map_err(|e| AppError::Other(format!("secure storage unavailable: {e}")))
    }

    pub fn try_keyring_read(account: &str) -> AppResult<Option<String>> {
        let entry = Self::keyring_entry(account)?;
        match entry.get_password() {
            Ok(value) if !value.trim().is_empty() => Ok(Some(value.trim().to_string())),
            Ok(_) => Ok(None),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Other(format!("failed to read secret: {e}"))),
        }
    }

    /// Returns true when keyring write + read-back verification succeeds.
    pub fn try_keyring_write(account: &str, value: &str) -> AppResult<bool> {
        let trimmed = value.trim();
        let entry = Self::keyring_entry(account)?;
        if entry.set_password(trimmed).is_err() {
            return Ok(false);
        }
        Ok(Self::try_keyring_read(account)?
            .map(|v| v == trimmed)
            .unwrap_or(false))
    }

    pub fn get_cursor_api_key() -> AppResult<Option<String>> {
        if let Some(value) = Self::try_keyring_read(CURSOR_API_KEY_ACCOUNT)? {
            return Ok(Some(value));
        }
        secret_sqlite::read_plain_standalone(CURSOR_API_KEY_ACCOUNT)
    }

    pub fn has_cursor_api_key() -> AppResult<bool> {
        Ok(Self::get_cursor_api_key()?.is_some())
    }

    pub fn clear_cursor_api_key() -> AppResult<()> {
        if let Ok(entry) = Self::keyring_entry(CURSOR_API_KEY_ACCOUNT) {
            let _ = entry.delete_credential();
        }
        if let Ok(db) = crate::storage::sqlite::Database::new(crate::storage::sqlite::db_path()?) {
            let _ = secret_sqlite::delete_plain(&db, CURSOR_API_KEY_ACCOUNT);
        }
        Ok(())
    }

    pub fn save_stt_api_key(key: &str) -> AppResult<()> {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            return Err(AppError::Other("STT API key cannot be empty".into()));
        }
        if Self::try_keyring_write(STT_API_KEY_ACCOUNT, trimmed)? {
            if let Ok(db) = crate::storage::sqlite::Database::new(crate::storage::sqlite::db_path()?)
            {
                let _ = secret_sqlite::delete_plain(&db, STT_API_KEY_ACCOUNT);
            }
            return Ok(());
        }
        let db = crate::storage::sqlite::Database::new(crate::storage::sqlite::db_path()?)?;
        secret_sqlite::write_plain(&db, STT_API_KEY_ACCOUNT, trimmed)?;
        let stored = secret_sqlite::read_plain(&db, STT_API_KEY_ACCOUNT)?;
        if stored.as_deref() != Some(trimmed) {
            return Err(AppError::Other("STT API key could not be persisted.".into()));
        }
        Ok(())
    }

    pub fn get_stt_api_key() -> AppResult<Option<String>> {
        if let Some(value) = Self::try_keyring_read(STT_API_KEY_ACCOUNT)? {
            return Ok(Some(value));
        }
        secret_sqlite::read_plain_standalone(STT_API_KEY_ACCOUNT)
    }

    pub fn clear_stt_api_key() -> AppResult<()> {
        if let Ok(entry) = Self::keyring_entry(STT_API_KEY_ACCOUNT) {
            let _ = entry.delete_credential();
        }
        if let Ok(db) = crate::storage::sqlite::Database::new(crate::storage::sqlite::db_path()?) {
            let _ = secret_sqlite::delete_plain(&db, STT_API_KEY_ACCOUNT);
        }
        Ok(())
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
        let entry = Self::keyring_entry(DATA_KEY_ACCOUNT)?;
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
        let entry = Self::keyring_entry(DATA_KEY_ACCOUNT)?;
        entry
            .set_password(&encoded)
            .map_err(|e| AppError::Other(format!("failed to store data key: {e}")))?;
        Ok(())
    }

    fn read_data_key() -> AppResult<Option<[u8; 32]>> {
        let entry = Self::keyring_entry(DATA_KEY_ACCOUNT)?;
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

#[cfg(test)]
mod tests {
    use super::SecretStore;

    #[test]
    fn rejects_openai_style_key_for_cursor_slot() {
        let err = SecretStore::validate_cursor_api_key("sk-proj-test12345678901234567890")
            .expect_err("expected rejection");
        assert!(err.to_string().contains("OpenAI"));
    }
}
