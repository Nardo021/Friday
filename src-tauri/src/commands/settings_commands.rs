use tauri::State;

use crate::core::AgentCore;
use crate::storage::settings_repo::FridaySettings;
use crate::storage::SettingsRepo;

#[tauri::command]
pub fn get_settings(core: State<'_, AgentCore>) -> Result<FridaySettings, crate::errors::AppError> {
    SettingsRepo::new(&core.db).get()
}

#[tauri::command]
pub fn save_settings(
    core: State<'_, AgentCore>,
    settings: FridaySettings,
) -> Result<(), crate::errors::AppError> {
    SettingsRepo::new(&core.db).save(&settings)
}
