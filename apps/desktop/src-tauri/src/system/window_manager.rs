use tauri::{AppHandle, Manager};

use crate::errors::{AppError, AppResult};
use crate::system::screen_manager::{
    WindowPosition, default_pet_position, get_window_outer_size, get_window_position,
    set_window_outer_position,
};

pub const PANEL_WINDOW_LABEL: &str = "panel";
pub const PET_WINDOW_LABEL: &str = "pet";
pub const STATUS_BUBBLE_WINDOW_LABEL: &str = "status-bubble";
pub const QUICK_BUBBLE_WINDOW_LABEL: &str = "quick-bubble";
pub const ONBOARDING_WINDOW_LABEL: &str = "onboarding";
const LEGACY_CHAT_WINDOW_LABEL: &str = "chat";

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

pub fn get_window_position_by_label(app: &AppHandle, label: &str) -> AppResult<WindowPosition> {
    let window = app
        .get_webview_window(label)
        .ok_or_else(|| AppError::Other(format!("window not found: {label}")))?;
    get_window_position(&window)
}

pub fn set_window_position(app: &AppHandle, label: &str, x: f64, y: f64) -> AppResult<WindowPosition> {
    let window = app
        .get_webview_window(label)
        .ok_or_else(|| AppError::Other(format!("window not found: {label}")))?;
    set_window_outer_position(&window, app, x, y)
}

pub fn get_pet_position(app: &AppHandle) -> AppResult<WindowPosition> {
    get_window_position_by_label(app, PET_WINDOW_LABEL)
}

pub fn set_pet_position(app: &AppHandle, x: f64, y: f64) -> AppResult<WindowPosition> {
    set_window_position(app, PET_WINDOW_LABEL, x, y)
}

pub fn anchor_window_near_pet(
    app: &AppHandle,
    target_label: &str,
    offset_x: f64,
    offset_y: f64,
) -> AppResult<WindowPosition> {
    let pet = get_pet_position(app)?;
    set_window_position(app, target_label, pet.x + offset_x, pet.y + offset_y)
}

pub fn set_click_through(app: &AppHandle, label: &str, enabled: bool) -> AppResult<()> {
    let window = app
        .get_webview_window(label)
        .ok_or_else(|| AppError::Other(format!("window not found: {label}")))?;
    window
        .set_ignore_cursor_events(enabled)
        .map_err(|e| AppError::Other(e.to_string()))
}

pub fn initialize_pet_position(app: &AppHandle, x: Option<f64>, y: Option<f64>) -> AppResult<WindowPosition> {
    let pet = app
        .get_webview_window(PET_WINDOW_LABEL)
        .ok_or_else(|| AppError::Other("pet window not found".into()))?;

    let (w, h) = get_window_outer_size(&pet)?;
    let position = match (x, y) {
        (Some(px), Some(py)) => set_window_outer_position(&pet, app, px, py)?,
        _ => {
            let default = default_pet_position(app, w, h)?;
            set_window_outer_position(&pet, app, default.x, default.y)?
        }
    };
    Ok(position)
}

pub fn open_panel(app: &AppHandle) -> AppResult<()> {
    if app.get_webview_window(PANEL_WINDOW_LABEL).is_some() {
        show_window(app, PANEL_WINDOW_LABEL)
    } else {
        show_window(app, LEGACY_CHAT_WINDOW_LABEL)
    }
}

pub fn toggle_panel(app: &AppHandle) -> AppResult<()> {
    let label = if app.get_webview_window(PANEL_WINDOW_LABEL).is_some() {
        PANEL_WINDOW_LABEL
    } else {
        LEGACY_CHAT_WINDOW_LABEL
    };
    toggle_window(app, label)
}

pub fn open_chat_from_pet(app: &AppHandle) -> AppResult<()> {
    open_panel(app)
}

pub fn open_quick_bubble(app: &AppHandle) -> AppResult<()> {
    let _ = anchor_window_near_pet(app, QUICK_BUBBLE_WINDOW_LABEL, 12.0, -300.0);
    show_window(app, QUICK_BUBBLE_WINDOW_LABEL)
}

pub fn hide_quick_bubble(app: &AppHandle) -> AppResult<()> {
    hide_window(app, QUICK_BUBBLE_WINDOW_LABEL)
}

pub fn finish_onboarding(app: &AppHandle, last_x: Option<f64>, last_y: Option<f64>) -> AppResult<()> {
    hide_window(app, ONBOARDING_WINDOW_LABEL)?;
    let _ = initialize_pet_position(app, last_x, last_y);
    show_window(app, PET_WINDOW_LABEL)?;
    Ok(())
}

pub fn show_onboarding(app: &AppHandle) -> AppResult<()> {
    let _ = hide_window(app, PANEL_WINDOW_LABEL);
    let _ = hide_window(app, PET_WINDOW_LABEL);
    show_window(app, ONBOARDING_WINDOW_LABEL)
}

pub fn show_status_bubble(app: &AppHandle) -> AppResult<()> {
    let _ = anchor_window_near_pet(app, STATUS_BUBBLE_WINDOW_LABEL, 0.0, -88.0);
    show_window(app, STATUS_BUBBLE_WINDOW_LABEL)
}

pub fn hide_status_bubble(app: &AppHandle) -> AppResult<()> {
    hide_window(app, STATUS_BUBBLE_WINDOW_LABEL)
}

pub fn open_command_center(app: &AppHandle) -> AppResult<()> {
    show_window(app, "command-center")
}
