#[cfg(desktop)]
pub fn setup_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
    use tauri::Manager;

    let show_pet = MenuItem::with_id(app, "show_pet", "Show Pet", true, None::<&str>)?;
    let hide_pet = MenuItem::with_id(app, "hide_pet", "Hide Pet", true, None::<&str>)?;
    let open_bubble =
        MenuItem::with_id(app, "open_bubble", "Quick Bubble", true, None::<&str>)?;
    let new_task = MenuItem::with_id(app, "new_task", "New Cursor Task", true, None::<&str>)?;
    let stop_task = MenuItem::with_id(app, "stop_task", "Stop Current Task", true, None::<&str>)?;
    let open_panel = MenuItem::with_id(app, "open_panel", "Open Panel", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Friday", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &show_pet,
            &hide_pet,
            &open_bubble,
            &new_task,
            &stop_task,
            &open_panel,
            &quit,
        ],
    )?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show_pet" => {
                let _ = crate::system::window_manager::show_window(app, "pet");
            }
            "hide_pet" => {
                let _ = crate::system::window_manager::hide_window(app, "pet");
            }
            "open_bubble" => {
                let _ = crate::system::window_manager::open_quick_bubble(app);
            }
            "new_task" => {
                let _ = crate::system::window_manager::open_panel(app);
            }
            "stop_task" => {
                if let Some(core) = app.try_state::<std::sync::Arc<crate::core::AgentCore>>() {
                    let core = core.inner().clone();
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let target = {
                            let mgr = core.session_manager.lock().await;
                            mgr.active_session()
                                .filter(|s| crate::core::event::is_running_status(s.status))
                                .or_else(|| {
                                    mgr.list().into_iter().find(|s| {
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
            "open_panel" => {
                let _ = crate::system::window_manager::open_panel(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let _ = crate::system::window_manager::open_quick_bubble(app);
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg(not(desktop))]
pub fn setup_tray(_app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
