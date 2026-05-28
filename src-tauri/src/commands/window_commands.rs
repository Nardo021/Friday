use tauri::AppHandle;

#[tauri::command]
pub fn show_window(app: AppHandle, label: String) -> Result<(), crate::errors::AppError> {
    crate::system::window_manager::show_window(&app, &label)
}

#[tauri::command]
pub fn hide_window(app: AppHandle, label: String) -> Result<(), crate::errors::AppError> {
    crate::system::window_manager::hide_window(&app, &label)
}

#[tauri::command]
pub fn open_chat(app: AppHandle) -> Result<(), crate::errors::AppError> {
    crate::system::window_manager::open_chat_from_pet(&app)
}

#[tauri::command]
pub fn open_quick_bubble(app: AppHandle) -> Result<(), crate::errors::AppError> {
    crate::system::window_manager::open_quick_bubble(&app)
}

#[tauri::command]
pub fn open_command_center(app: AppHandle) -> Result<(), crate::errors::AppError> {
    crate::system::window_manager::open_command_center(&app)
}
