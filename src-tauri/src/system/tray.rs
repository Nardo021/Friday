#[cfg(desktop)]
pub fn setup_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
    use tauri::Manager;

    let open_chat = MenuItem::with_id(app, "open_chat", "Open Chat", true, None::<&str>)?;
    let open_cc = MenuItem::with_id(app, "open_cc", "Command Center", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Friday", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_chat, &open_cc, &quit])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open_chat" => {
                let _ = crate::system::window_manager::show_window(app, "chat");
            }
            "open_cc" => {
                let _ = crate::system::window_manager::open_command_center(app);
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
                let _ = crate::system::window_manager::toggle_window(app, "chat");
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg(not(desktop))]
pub fn setup_tray(_app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
