use tauri::{AppHandle, Manager, WebviewWindow};

use crate::errors::{AppError, AppResult};

pub fn show_window(app: &AppHandle, label: &str) -> AppResult<()> {
    if let Some(window) = app.get_webview_window(label) {
        window.show().map_err(|e| AppError::Other(e.to_string()))?;
        window.set_focus().map_err(|e| AppError::Other(e.to_string()))?;
    } else {
        return Err(AppError::Other(format!("window not found: {label}")));
    }
    Ok(())
}

pub fn hide_window(app: &AppHandle, label: &str) -> AppResult<()> {
    if let Some(window) = app.get_webview_window(label) {
        window.hide().map_err(|e| AppError::Other(e.to_string()))?;
    }
    Ok(())
}

pub fn toggle_window(app: &AppHandle, label: &str) -> AppResult<()> {
    if let Some(window) = app.get_webview_window(label) {
        if window.is_visible().unwrap_or(false) {
            hide_window(app, label)?;
        } else {
            show_window(app, label)?;
        }
    }
    Ok(())
}

pub fn open_chat_from_pet(app: &AppHandle) -> AppResult<()> {
    show_window(app, "chat")
}

pub fn open_quick_bubble(app: &AppHandle) -> AppResult<()> {
    show_window(app, "quick-bubble")
}

pub fn open_command_center(app: &AppHandle) -> AppResult<()> {
    show_window(app, "command-center")
}
