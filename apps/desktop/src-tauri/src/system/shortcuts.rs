use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::core::AgentCore;

pub fn register_shortcuts(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let gs = app.global_shortcut();
    if let Err(e) = gs.unregister_all() {
        eprintln!("Friday: failed to clear previous shortcuts: {e}");
    }

    #[cfg(target_os = "windows")]
    {
        // Avoid Ctrl+Space — it conflicts with the Chinese IME switcher on Windows.
        bind(
            app,
            Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::Space),
            "quick_bubble",
        );
    }
    #[cfg(not(target_os = "windows"))]
    {
        bind(
            app,
            Shortcut::new(Some(Modifiers::CONTROL), Code::Space),
            "quick_bubble",
        );
    }
    bind(
        app,
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyF),
        "open_panel",
    );
    bind(
        app,
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV),
        "voice_input",
    );
    bind(
        app,
        Shortcut::new(Some(Modifiers::CONTROL), Code::Period),
        "stop_session",
    );

    Ok(())
}

fn bind(app: &tauri::AppHandle, shortcut: Shortcut, action: &'static str) {
    let label = shortcut.into_string();
    let gs = app.global_shortcut();
    if let Err(e) = gs.unregister(shortcut.clone()) {
        eprintln!("Friday: could not unregister {label} before re-bind: {e}");
    }

    if let Err(e) = gs.on_shortcut(shortcut, move |app, _, event| {
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
                        let target = {
                            let mgr = core.session_manager.lock().await;
                            mgr.active_session()
                                .filter(|s| {
                                    crate::core::event::is_running_status(s.status)
                                })
                                .or_else(|| {
                                    mgr.list()
                                        .into_iter()
                                        .find(|s| {
                                            crate::core::event::is_running_status(s.status)
                                        })
                                })
                        };
                        if let Some(s) = target {
                            let _ = core.close_session_safely(app, &s.id).await;
                        }
                    });
                }
            }
            _ => {}
        }
    }) {
        eprintln!(
            "Friday: shortcut {label} ({action}) not registered — likely in use by another app: {e}"
        );
    }
}
