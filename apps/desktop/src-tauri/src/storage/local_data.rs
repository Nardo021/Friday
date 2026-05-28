use std::fs;
use std::path::{Path, PathBuf};

use crate::errors::{AppError, AppResult};
use crate::security::SecretStore;

/// Matches Tauri bundle identifier so the NSIS uninstaller removes this folder.
pub const APP_DATA_DIR_NAME: &str = "com.leo.friday";
const LEGACY_APP_DATA_DIR_NAME: &str = "Friday";
const WIPE_MARKER: &str = ".wipe-requested";
pub fn app_data_dir() -> AppResult<PathBuf> {
    dirs::data_dir()
        .map(|d| d.join(APP_DATA_DIR_NAME))
        .ok_or_else(|| AppError::Other("cannot resolve data dir".into()))
}

pub fn legacy_app_data_dir() -> AppResult<PathBuf> {
    dirs::data_dir()
        .map(|d| d.join(LEGACY_APP_DATA_DIR_NAME))
        .ok_or_else(|| AppError::Other("cannot resolve data dir".into()))
}

pub fn migrate_legacy_app_data_dir() -> AppResult<()> {
    let legacy = legacy_app_data_dir()?;
    let current = app_data_dir()?;

    if !legacy.exists() || legacy == current {
        return Ok(());
    }

    if !current.exists() {
        fs::create_dir_all(current.parent().ok_or_else(|| {
            AppError::Other("invalid app data path".into())
        })?)?;
        fs::rename(&legacy, &current).map_err(|e| AppError::Storage(e.to_string()))?;
        return Ok(());
    }

    merge_dir(&legacy, &current)?;
    let _ = fs::remove_dir_all(&legacy);
    Ok(())
}

fn merge_dir(from: &Path, to: &Path) -> AppResult<()> {
    for entry in fs::read_dir(from).map_err(|e| AppError::Storage(e.to_string()))? {
        let entry = entry.map_err(|e| AppError::Storage(e.to_string()))?;
        let dest = to.join(entry.file_name());
        if dest.exists() {
            continue;
        }
        fs::rename(entry.path(), dest).map_err(|e| AppError::Storage(e.to_string()))?;
    }
    Ok(())
}

pub fn schedule_wipe_on_restart() -> AppResult<()> {
    let dir = app_data_dir()?;
    fs::create_dir_all(&dir)?;
    fs::write(dir.join(WIPE_MARKER), b"1")?;
    Ok(())
}

/// Runs before the SQLite database is opened. Returns true if data was wiped.
pub fn consume_wipe_on_restart() -> AppResult<bool> {
    let dir = app_data_dir()?;
    if !dir.join(WIPE_MARKER).exists() {
        return Ok(false);
    }
    wipe_local_app_data()?;
    Ok(true)
}

pub fn wipe_local_app_data() -> AppResult<()> {    SecretStore::clear_all()?;
    let current = app_data_dir()?;
    if current.exists() {
        fs::remove_dir_all(&current).map_err(|e| AppError::Storage(e.to_string()))?;
    }
    let legacy = legacy_app_data_dir()?;
    if legacy.exists() {
        fs::remove_dir_all(&legacy).map_err(|e| AppError::Storage(e.to_string()))?;
    }
    Ok(())
}

pub fn local_data_dir_display() -> AppResult<String> {
    Ok(app_data_dir()?.display().to_string())
}
