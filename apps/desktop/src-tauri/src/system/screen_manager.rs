use serde::Serialize;
use tauri::{AppHandle, Manager, Monitor, PhysicalPosition, WebviewWindow};

use crate::errors::{AppError, AppResult};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    pub name: Option<String>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
    pub work_area_x: i32,
    pub work_area_y: i32,
    pub work_area_width: u32,
    pub work_area_height: u32,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowPosition {
    pub x: f64,
    pub y: f64,
}

pub fn get_primary_monitor_info(app: &AppHandle) -> AppResult<MonitorInfo> {
    let monitor = app
        .primary_monitor()
        .map_err(|e| AppError::Other(e.to_string()))?
        .ok_or_else(|| AppError::Other("no primary monitor".into()))?;
    Ok(monitor_to_info(&monitor))
}

pub fn get_all_monitors_info(app: &AppHandle) -> AppResult<Vec<MonitorInfo>> {
    let monitors = app
        .available_monitors()
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok(monitors.iter().map(monitor_to_info).collect())
}

fn monitor_to_info(monitor: &Monitor) -> MonitorInfo {
    let pos = monitor.position();
    let size = monitor.size();
    let scale = monitor.scale_factor();
    let work = monitor.work_area();

    MonitorInfo {
        name: monitor.name().map(|s| s.to_string()),
        x: pos.x,
        y: pos.y,
        width: size.width,
        height: size.height,
        scale_factor: scale,
        work_area_x: work.position.x,
        work_area_y: work.position.y,
        work_area_width: work.size.width,
        work_area_height: work.size.height,
    }
}

pub fn get_window_outer_size(window: &WebviewWindow) -> AppResult<(u32, u32)> {
    let size = window
        .outer_size()
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok((size.width, size.height))
}

pub fn get_window_position(window: &WebviewWindow) -> AppResult<WindowPosition> {
    let pos = window
        .outer_position()
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok(WindowPosition {
        x: pos.x as f64,
        y: pos.y as f64,
    })
}

pub fn clamp_position_for_monitor(
    monitor: &MonitorInfo,
    x: f64,
    y: f64,
    window_w: u32,
    window_h: u32,
) -> WindowPosition {
    let min_x = monitor.work_area_x as f64;
    let min_y = monitor.work_area_y as f64;
    let max_x = min_x + monitor.work_area_width as f64 - window_w as f64;
    let max_y = min_y + monitor.work_area_height as f64 - window_h as f64;

    WindowPosition {
        x: x.clamp(min_x, max_x.max(min_x)),
        y: y.clamp(min_y, max_y.max(min_y)),
    }
}

pub fn clamp_position(app: &AppHandle, x: f64, y: f64, window_w: u32, window_h: u32) -> AppResult<WindowPosition> {
    let monitor = get_primary_monitor_info(app)?;
    Ok(clamp_position_for_monitor(&monitor, x, y, window_w, window_h))
}

pub fn default_pet_position(app: &AppHandle, window_w: u32, window_h: u32) -> AppResult<WindowPosition> {
    let monitor = get_primary_monitor_info(app)?;
    let x = monitor.work_area_x as f64 + monitor.work_area_width as f64 - window_w as f64 - 24.0;
    let y = monitor.work_area_y as f64 + monitor.work_area_height as f64 - window_h as f64 - 16.0;
    Ok(clamp_position_for_monitor(&monitor, x, y, window_w, window_h))
}

pub fn set_window_outer_position(
    window: &WebviewWindow,
    app: &AppHandle,
    x: f64,
    y: f64,
) -> AppResult<WindowPosition> {
    let (w, h) = get_window_outer_size(window)?;
    let clamped = clamp_position(app, x, y, w, h)?;
    window
        .set_position(PhysicalPosition::new(clamped.x as i32, clamped.y as i32))
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok(clamped)
}

pub fn monitor_containing_point(app: &AppHandle, x: f64, y: f64) -> AppResult<MonitorInfo> {
    let monitors = get_all_monitors_info(app)?;
    for monitor in monitors {
        let in_x = x >= monitor.x as f64 && x < monitor.x as f64 + monitor.width as f64;
        let in_y = y >= monitor.y as f64 && y < monitor.y as f64 + monitor.height as f64;
        if in_x && in_y {
            return Ok(monitor);
        }
    }
    get_primary_monitor_info(app)
}

pub fn clamp_position_at_point(
    app: &AppHandle,
    x: f64,
    y: f64,
    window_w: u32,
    window_h: u32,
) -> AppResult<WindowPosition> {
    let monitor = monitor_containing_point(app, x + window_w as f64 / 2.0, y + window_h as f64 / 2.0)?;
    Ok(clamp_position_for_monitor(&monitor, x, y, window_w, window_h))
}
