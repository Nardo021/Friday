use std::sync::Arc;

use tauri::State;

use crate::bridge::{bridge_url, start_bridge, stop_bridge, BridgeBroadcast};
use crate::core::AgentCore;
use crate::storage::settings_repo::{
    ensure_mobile_bridge_token, MobileBridgeSettings, SettingsRepo,
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileBridgeSettingsView {
    pub enabled: bool,
    pub port: u16,
    pub auth_token: String,
    pub local_url: String,
}

#[tauri::command]
pub fn get_mobile_bridge_settings(
    core: State<'_, Arc<AgentCore>>,
) -> Result<MobileBridgeSettingsView, crate::errors::AppError> {
    let mut settings = SettingsRepo::new(&core.db).get()?;
    ensure_mobile_bridge_token(&mut settings);
    SettingsRepo::new(&core.db).save(&settings)?;
    core.reload_settings_cache()?;
    Ok(to_view(&settings.mobile_bridge))
}

#[tauri::command]
pub fn update_mobile_bridge_settings(
    core: State<'_, Arc<AgentCore>>,
    app: tauri::AppHandle,
    bridge: State<'_, BridgeBroadcast>,
    input: MobileBridgeSettings,
) -> Result<MobileBridgeSettingsView, crate::errors::AppError> {
    let mut settings = SettingsRepo::new(&core.db).get()?;
    settings.mobile_bridge.enabled = input.enabled;
    settings.mobile_bridge.port = input.port;
    ensure_mobile_bridge_token(&mut settings);
    SettingsRepo::new(&core.db).save(&settings)?;
    core.reload_settings_cache()?;

    stop_bridge();
    if settings.mobile_bridge.enabled {
        start_bridge(
            app,
            core.inner().clone(),
            bridge.inner().clone(),
            settings.mobile_bridge.port,
            settings.mobile_bridge.auth_token.clone(),
        );
    }

    Ok(to_view(&settings.mobile_bridge))
}

#[tauri::command]
pub fn regenerate_mobile_bridge_token(
    core: State<'_, Arc<AgentCore>>,
    app: tauri::AppHandle,
    bridge: State<'_, BridgeBroadcast>,
) -> Result<MobileBridgeSettingsView, crate::errors::AppError> {
    let mut settings = SettingsRepo::new(&core.db).get()?;
    settings.mobile_bridge.auth_token = uuid::Uuid::new_v4().to_string();
    SettingsRepo::new(&core.db).save(&settings)?;

    if settings.mobile_bridge.enabled {
        stop_bridge();
        start_bridge(
            app,
            core.inner().clone(),
            bridge.inner().clone(),
            settings.mobile_bridge.port,
            settings.mobile_bridge.auth_token.clone(),
        );
    }

    Ok(to_view(&settings.mobile_bridge))
}

#[tauri::command]
pub fn get_local_bridge_url(
    core: State<'_, Arc<AgentCore>>,
) -> Result<String, crate::errors::AppError> {
    let settings = SettingsRepo::new(&core.db).get()?;
    Ok(bridge_url(settings.mobile_bridge.port))
}

fn to_view(mobile: &MobileBridgeSettings) -> MobileBridgeSettingsView {
    MobileBridgeSettingsView {
        enabled: mobile.enabled,
        port: mobile.port,
        auth_token: mobile.auth_token.clone(),
        local_url: bridge_url(mobile.port),
    }
}
