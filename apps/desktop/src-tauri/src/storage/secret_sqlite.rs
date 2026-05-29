use rusqlite::params;

use crate::errors::{AppError, AppResult};
use crate::security::DataCrypto;
use crate::storage::sqlite::{Database, db_path};

/// Plaintext API-key rows in the shared `settings` table (fallback when keyring fails).
pub fn read_plain(db: &Database, key: &str) -> AppResult<Option<String>> {
    let raw = read_raw(db, key)?;
    let Some(raw) = raw else {
        return Ok(None);
    };
    if raw.starts_with("enc:v1:") {
        if let Ok(plain) = DataCrypto::decrypt(&raw) {
            return Ok(Some(plain));
        }
    }
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

pub fn write_plain(db: &Database, key: &str, plaintext: &str) -> AppResult<()> {
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO settings (key, value_json) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
            params![key, plaintext],
        )
        .map_err(|e| AppError::Storage(format!("failed to write secret: {e}")))?;
        Ok(())
    })
}

pub fn delete_plain(db: &Database, key: &str) -> AppResult<()> {
    db.with_conn(|conn| {
        conn.execute("DELETE FROM settings WHERE key = ?1", params![key])
            .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(())
    })
}

/// Standalone read for code paths without `Arc<Database>` (e.g. cloud client).
pub fn read_plain_standalone(key: &str) -> AppResult<Option<String>> {
    let db = Database::new(db_path()?)?;
    read_plain(&db, key)
}

fn read_raw(db: &Database, key: &str) -> AppResult<Option<String>> {
    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value_json FROM settings WHERE key = ?1")
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query(params![key])
            .map_err(|e| AppError::Storage(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| AppError::Storage(e.to_string()))? {
            row.get(0).map_err(|e| AppError::Storage(e.to_string()))
        } else {
            Ok(None)
        }
    })
}
