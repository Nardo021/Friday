mod adapters;
mod bridge;
mod commands;
mod core;
mod discovery;
mod errors;
mod pty;
mod process;
mod security;
mod storage;
mod system;
mod voice;

use std::sync::Arc;

use tauri::Manager;

use core::agent_core::AgentCore;
use discovery::start_discovery_loop;
use bridge::BridgeBroadcast;
use process::{ProcessSupervisor, create_process_registry};
use storage::{
    consume_wipe_on_restart, migrate_legacy_app_data_dir, Database, SettingsRepo, db_path,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(system::autostart::autostart_builder().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            migrate_legacy_app_data_dir().map_err(|e| e.to_string())?;
            consume_wipe_on_restart().map_err(|e| e.to_string())?;

            let db = Database::new(db_path().map_err(|e| e.to_string())?)                .map_err(|e| e.to_string())?;
            db.migrate().map_err(|e| e.to_string())?;
            let db = Arc::new(db);

            SettingsRepo::new(&db)
                .migrate_secrets()
                .map_err(|e| e.to_string())?;

            let settings = SettingsRepo::new(&db).get().map_err(|e| e.to_string())?;

            let registry = create_process_registry();
            let supervisor = Arc::new(ProcessSupervisor::new(registry));
            let core = Arc::new(AgentCore::new(supervisor, db));
            core.bind_weak(Arc::downgrade(&core));

            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async {
                core.init(app_handle).await.map_err(|e| e.to_string())
            })?;

            start_discovery_loop(core.clone());

            let bridge_broadcast = BridgeBroadcast::new(256);
            app.manage(bridge_broadcast.clone());

            if settings.mobile_bridge.enabled {
                bridge::start_bridge(
                    app.handle().clone(),
                    core.clone(),
                    bridge_broadcast,
                    settings.mobile_bridge.port,
                    settings.mobile_bridge.auth_token.clone(),
                );
            }

            app.manage(core);
            system::tray::setup_tray(app.handle()).map_err(|e| e.to_string())?;
            system::shortcuts::register_shortcuts(app.handle()).map_err(|e| e.to_string())?;

            if settings.behavior.launch_at_startup {
                let _ = system::autostart::set_launch_at_startup(app.handle(), true);
            }

            let _ = system::window_manager::initialize_pet_position(
                app.handle(),
                settings.pet.last_x,
                settings.pet.last_y,
            );

            if settings.onboarding.completed {
                let _ = system::window_manager::show_window(app.handle(), "pet");
            } else {
                let _ = system::window_manager::show_onboarding(app.handle());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::agent_commands::create_session,
            commands::agent_commands::close_session,
            commands::agent_commands::select_active_session,
            commands::agent_commands::resize_terminal,
            commands::agent_commands::list_active_sessions,
            commands::agent_commands::follow_up,
            commands::agent_commands::start_agent_session,
            commands::agent_commands::send_agent_message,
            commands::agent_commands::stop_agent_session,
            commands::agent_commands::get_session_status,
            commands::agent_commands::approve_command,
            commands::agent_commands::reject_command,
            commands::intent_commands::route_quick_input,
            commands::intent_commands::submit_quick_input,
            commands::intent_commands::execute_quick_intent,
            commands::session_commands::list_sessions,
            commands::session_commands::get_session_detail,
            commands::session_commands::get_session_events,
            commands::voice_commands::transcribe_audio,
            commands::history_commands::list_ideas,
            commands::history_commands::delete_idea,
            commands::history_commands::list_messages,
            commands::history_commands::export_session_markdown,
            commands::history_commands::delete_session,
            commands::voice_commands::save_stt_api_key,
            commands::voice_commands::clear_stt_api_key,
            commands::project_commands::add_project,
            commands::project_commands::list_projects,
            commands::project_commands::get_project,
            commands::settings_commands::get_settings,
            commands::settings_commands::save_settings,
            commands::settings_commands::save_cursor_api_key,
            commands::settings_commands::clear_cursor_api_key,
            commands::settings_commands::get_local_data_path,            commands::settings_commands::clear_local_data,
            commands::bridge_commands::get_mobile_bridge_settings,
            commands::bridge_commands::update_mobile_bridge_settings,
            commands::bridge_commands::regenerate_mobile_bridge_token,
            commands::bridge_commands::get_local_bridge_url,
            commands::window_commands::show_window,
            commands::window_commands::hide_window,
            commands::window_commands::open_panel,
            commands::window_commands::open_chat,
            commands::window_commands::open_quick_bubble,
            commands::window_commands::open_command_center,
            commands::window_commands::get_pet_position,
            commands::window_commands::set_pet_position,
            commands::window_commands::get_window_position,
            commands::window_commands::set_window_position,
            commands::window_commands::anchor_window,
            commands::window_commands::set_window_click_through,
            commands::window_commands::get_monitor_info,
            commands::window_commands::get_all_monitors,
            commands::window_commands::show_status_bubble,
            commands::window_commands::hide_status_bubble,
            commands::window_commands::hide_quick_bubble,
            commands::window_commands::finish_onboarding,
            commands::window_commands::open_onboarding,
            commands::adapter_commands::list_adapters,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
