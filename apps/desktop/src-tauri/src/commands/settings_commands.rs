use std::sync::Arc;

use tauri::State;

use crate::core::AgentCore;
use crate::storage::local_data::{local_data_dir_display, schedule_wipe_on_restart};
use crate::storage::settings_repo::FridaySettings;
use crate::storage::SettingsRepo;

#[tauri::command]
pub fn get_settings(
    core: State<'_, Arc<AgentCore>>,
) -> Result<FridaySettings, crate::errors::AppError> {
    SettingsRepo::new(&core.db).get()
}

#[tauri::command]
pub fn save_settings(
    core: State<'_, Arc<AgentCore>>,
    app: tauri::AppHandle,
    settings: FridaySettings,
) -> Result<(), crate::errors::AppError> {
    SettingsRepo::new(&core.db).save(&settings)?;
    core.reload_settings_cache()?;
    let _ = crate::system::autostart::set_launch_at_startup(
        &app,
        settings.behavior.launch_at_startup,
    );
    Ok(())
}

#[tauri::command]
pub async fn save_cursor_api_key(
    core: State<'_, Arc<AgentCore>>,
    api_key: String,
) -> Result<(), crate::errors::AppError> {
    let trimmed = api_key.trim();
    crate::security::SecretStore::validate_cursor_api_key(trimmed)?;
    crate::adapters::cursor_cloud_agent::client::CursorCloudClient::verify_api_key(trimmed).await?;
    SettingsRepo::new(&core.db).save_cursor_api_key(trimmed)?;
    core.reload_settings_cache()?;
    Ok(())
}

#[tauri::command]
pub async fn verify_cursor_api_key(api_key: String) -> Result<(), crate::errors::AppError> {
    let trimmed = api_key.trim();
    crate::security::SecretStore::validate_cursor_api_key(trimmed)?;
    crate::adapters::cursor_cloud_agent::client::CursorCloudClient::verify_api_key(trimmed).await
}

#[tauri::command]
pub fn get_local_data_path() -> Result<String, crate::errors::AppError> {
    local_data_dir_display()
}

#[tauri::command]
pub fn clear_cursor_api_key(
    core: State<'_, Arc<AgentCore>>,
) -> Result<(), crate::errors::AppError> {
    SettingsRepo::new(&core.db).clear_cursor_api_key()?;
    core.reload_settings_cache()?;
    Ok(())
}

#[tauri::command]
pub fn clear_local_data(app: tauri::AppHandle) -> Result<(), crate::errors::AppError> {
    schedule_wipe_on_restart()?;
    let _ = crate::security::SecretStore::clear_all();
    app.restart();
    #[allow(unreachable_code)]
    Ok(())
}
