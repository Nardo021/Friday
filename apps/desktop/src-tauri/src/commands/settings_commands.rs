use tauri::State;

use crate::core::AgentCore;use crate::storage::local_data::{
    local_data_dir_display, schedule_wipe_on_restart,
};use crate::storage::settings_repo::FridaySettings;
use crate::storage::SettingsRepo;

#[tauri::command]
pub fn get_settings(core: State<'_, AgentCore>) -> Result<FridaySettings, crate::errors::AppError> {
    SettingsRepo::new(&core.db).get()
}

#[tauri::command]
pub fn save_settings(
    core: State<'_, AgentCore>,
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
pub fn save_cursor_api_key(
    core: State<'_, AgentCore>,
    api_key: String,
) -> Result<(), crate::errors::AppError> {
    SettingsRepo::new(&core.db).save_cursor_api_key(&api_key)
}

#[tauri::command]
pub fn get_local_data_path() -> Result<String, crate::errors::AppError> {
    local_data_dir_display()
}

#[tauri::command]
pub fn clear_cursor_api_key(core: State<'_, AgentCore>) -> Result<(), crate::errors::AppError> {
    SettingsRepo::new(&core.db).clear_cursor_api_key()
}

#[tauri::command]
pub fn clear_local_data(app: tauri::AppHandle) -> Result<(), crate::errors::AppError> {
    schedule_wipe_on_restart()?;
    let _ = crate::security::SecretStore::clear_all();
    app.restart();
    #[allow(unreachable_code)]
    Ok(())
}