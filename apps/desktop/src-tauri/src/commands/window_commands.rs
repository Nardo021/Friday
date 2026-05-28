use tauri::AppHandle;

use crate::system::screen_manager::{MonitorInfo, WindowPosition};
use crate::system::window_manager;

#[tauri::command]
pub fn show_window(app: AppHandle, label: String) -> Result<(), crate::errors::AppError> {
    window_manager::show_window(&app, &label)
}

#[tauri::command]
pub fn hide_window(app: AppHandle, label: String) -> Result<(), crate::errors::AppError> {
    window_manager::hide_window(&app, &label)
}

#[tauri::command]
pub fn open_panel(app: AppHandle) -> Result<(), crate::errors::AppError> {
    window_manager::open_panel(&app)
}

#[tauri::command]
pub fn open_chat(app: AppHandle) -> Result<(), crate::errors::AppError> {
    window_manager::open_chat_from_pet(&app)
}

#[tauri::command]
pub fn open_quick_bubble(app: AppHandle) -> Result<(), crate::errors::AppError> {
    window_manager::open_quick_bubble(&app)
}

#[tauri::command]
pub fn open_command_center(app: AppHandle) -> Result<(), crate::errors::AppError> {
    window_manager::open_command_center(&app)
}

#[tauri::command]
pub fn get_pet_position(app: AppHandle) -> Result<WindowPosition, crate::errors::AppError> {
    window_manager::get_pet_position(&app)
}

#[tauri::command]
pub fn set_pet_position(
    app: AppHandle,
    x: f64,
    y: f64,
) -> Result<WindowPosition, crate::errors::AppError> {
    window_manager::set_pet_position(&app, x, y)
}

#[tauri::command]
pub fn get_window_position(
    app: AppHandle,
    label: String,
) -> Result<WindowPosition, crate::errors::AppError> {
    window_manager::get_window_position_by_label(&app, &label)
}

#[tauri::command]
pub fn set_window_position(
    app: AppHandle,
    label: String,
    x: f64,
    y: f64,
) -> Result<WindowPosition, crate::errors::AppError> {
    window_manager::set_window_position(&app, &label, x, y)
}

#[tauri::command]
pub fn anchor_window(
    app: AppHandle,
    label: String,
    offset_x: f64,
    offset_y: f64,
) -> Result<WindowPosition, crate::errors::AppError> {
    window_manager::anchor_window_near_pet(&app, &label, offset_x, offset_y)
}

#[tauri::command]
pub fn set_window_click_through(
    app: AppHandle,
    label: String,
    enabled: bool,
) -> Result<(), crate::errors::AppError> {
    window_manager::set_click_through(&app, &label, enabled)
}

#[tauri::command]
pub fn get_monitor_info(app: AppHandle) -> Result<MonitorInfo, crate::errors::AppError> {
    crate::system::screen_manager::get_primary_monitor_info(&app)
}

#[tauri::command]
pub fn get_all_monitors(app: AppHandle) -> Result<Vec<MonitorInfo>, crate::errors::AppError> {
    crate::system::screen_manager::get_all_monitors_info(&app)
}

#[tauri::command]
pub fn show_status_bubble(app: AppHandle) -> Result<(), crate::errors::AppError> {
    window_manager::show_status_bubble(&app)
}

#[tauri::command]
pub fn hide_status_bubble(app: AppHandle) -> Result<(), crate::errors::AppError> {
    window_manager::hide_status_bubble(&app)
}

#[tauri::command]
pub fn hide_quick_bubble(app: AppHandle) -> Result<(), crate::errors::AppError> {
    window_manager::hide_quick_bubble(&app)
}

#[tauri::command]
pub fn finish_onboarding(app: AppHandle) -> Result<(), crate::errors::AppError> {
    window_manager::finish_onboarding(&app, None, None)
}

#[tauri::command]
pub fn open_onboarding(app: AppHandle) -> Result<(), crate::errors::AppError> {
    window_manager::show_onboarding(&app)
}
