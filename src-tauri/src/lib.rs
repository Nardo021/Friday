mod adapters;
mod commands;
mod core;
mod errors;
mod process;
mod security;
mod storage;
mod system;

use std::sync::Arc;

use tauri::Manager;

use core::AgentCore;
use process::{ProcessSupervisor, create_process_registry};
use storage::{Database, db_path};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db = Database::new(db_path().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            db.migrate().map_err(|e| e.to_string())?;
            let db = Arc::new(db);

            let registry = create_process_registry();
            let supervisor = Arc::new(ProcessSupervisor::new(registry));
            let core = AgentCore::new(supervisor, db);

            tauri::async_runtime::block_on(async {
                core.init().await.map_err(|e| e.to_string())
            })?;

            app.manage(core);
            system::tray::setup_tray(app.handle()).map_err(|e| e.to_string())?;
            let _ = system::window_manager::show_window(app.handle(), "pet");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::agent_commands::start_agent_session,
            commands::agent_commands::send_agent_message,
            commands::agent_commands::stop_agent_session,
            commands::agent_commands::get_session_status,
            commands::agent_commands::approve_command,
            commands::agent_commands::reject_command,
            commands::session_commands::list_sessions,
            commands::session_commands::get_session_detail,
            commands::session_commands::get_session_events,
            commands::project_commands::add_project,
            commands::project_commands::list_projects,
            commands::project_commands::get_project,
            commands::settings_commands::get_settings,
            commands::settings_commands::save_settings,
            commands::window_commands::show_window,
            commands::window_commands::hide_window,
            commands::window_commands::open_chat,
            commands::window_commands::open_quick_bubble,
            commands::window_commands::open_command_center,
            commands::adapter_commands::list_adapters,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
