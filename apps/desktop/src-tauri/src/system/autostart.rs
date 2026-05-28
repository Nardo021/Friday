pub fn set_launch_at_startup(app: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let autostart = app.autolaunch();
    if enabled {
        autostart.enable().map_err(|e| e.to_string())
    } else {
        autostart.disable().map_err(|e| e.to_string())
    }
}

pub fn autostart_builder() -> tauri_plugin_autostart::Builder {
    tauri_plugin_autostart::Builder::new().app_name("Friday")
}
