use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::core::AgentCore;

pub fn register_shortcuts(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();

    #[cfg(target_os = "windows")]
    {
        // Avoid Ctrl+Space — it conflicts with the Chinese IME switcher on Windows.
        bind(
            app,
            Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::Space),
            "quick_bubble",
        )?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        bind(app, Shortcut::new(Some(Modifiers::CONTROL), Code::Space), "quick_bubble")?;
    }
    bind(
        app,
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyF),
        "open_panel",
    )?;
    bind(
        app,
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV),
        "voice_input",
    )?;
    bind(
        app,
        Shortcut::new(Some(Modifiers::CONTROL), Code::Period),
        "stop_session",
    )?;

    Ok(())
}

fn bind(
    app: &tauri::AppHandle,
    shortcut: Shortcut,
    action: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = app.clone();
    app.global_shortcut().on_shortcut(shortcut, move |app, _, event| {
        if event.state != ShortcutState::Pressed {
            return;
        }
        match action {
            "quick_bubble" | "voice_input" => {
                let _ = crate::system::window_manager::open_quick_bubble(app);
            }
            "open_panel" => {
                let _ = crate::system::window_manager::open_panel(app);
            }
            "stop_session" => {
                if let Some(core) = app.try_state::<std::sync::Arc<AgentCore>>() {
                    let core = core.inner().clone();
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Ok(sessions) = core.list_active_sessions().await {
                            if let Some(s) = sessions.first() {
                                let _ = core.close_session_safely(app, &s.id).await;
                            }
                        }
                    });
                }
            }
            _ => {}
        }
    })?;
    Ok(())
}
