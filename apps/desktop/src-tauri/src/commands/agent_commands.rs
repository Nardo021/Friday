use std::sync::Arc;

use tauri::State;

use crate::core::event::{AgentSessionType, FridaySession};
use crate::core::AgentCore;

#[tauri::command]
pub async fn create_session(
    app: tauri::AppHandle,
    core: State<'_, Arc<AgentCore>>,
    session_type: String,
    project_id: String,
    prompt: String,
) -> Result<FridaySession, crate::errors::AppError> {
    let session_type = parse_session_type(&session_type)?;
    match core
        .create_session(app.clone(), session_type, project_id, prompt)
        .await
    {
        Ok(session) => Ok(session),
        Err(e) => {
            let _ = core
                .emit_visible_user_error(&app, None, &e.to_string())
                .await;
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn close_session(
    app: tauri::AppHandle,
    core: State<'_, Arc<AgentCore>>,
    session_id: String,
) -> Result<(), crate::errors::AppError> {
    core.close_session_safely(app, &session_id).await
}

#[tauri::command]
pub async fn select_active_session(
    core: State<'_, Arc<AgentCore>>,
    session_id: String,
) -> Result<(), crate::errors::AppError> {
    core.select_active_session(&session_id).await
}

#[tauri::command]
pub async fn resize_terminal(
    core: State<'_, Arc<AgentCore>>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), crate::errors::AppError> {
    core.resize_terminal(&session_id, cols, rows).await
}

#[tauri::command]
pub async fn list_active_sessions(
    core: State<'_, Arc<AgentCore>>,
) -> Result<Vec<FridaySession>, crate::errors::AppError> {
    core.list_active_sessions().await
}

#[tauri::command]
pub async fn follow_up(
    app: tauri::AppHandle,
    core: State<'_, Arc<AgentCore>>,
    session_id: String,
    message: String,
) -> Result<(), crate::errors::AppError> {
    core.follow_up(app, &session_id, message).await
}

#[tauri::command]
pub async fn start_agent_session(
    app: tauri::AppHandle,
    core: State<'_, Arc<AgentCore>>,
    project_id: String,
    prompt: String,
) -> Result<FridaySession, crate::errors::AppError> {
    core.start_session(app, project_id, prompt).await
}

#[tauri::command]
pub async fn send_agent_message(
    app: tauri::AppHandle,
    core: State<'_, Arc<AgentCore>>,
    session_id: String,
    message: String,
) -> Result<(), crate::errors::AppError> {
    core.send_message(app, &session_id, message).await
}

#[tauri::command]
pub async fn stop_agent_session(
    app: tauri::AppHandle,
    core: State<'_, Arc<AgentCore>>,
    session_id: String,
) -> Result<(), crate::errors::AppError> {
    core.stop_session(app, &session_id).await
}

#[tauri::command]
pub async fn get_session_status(
    core: State<'_, Arc<AgentCore>>,
    session_id: String,
) -> Result<FridaySession, crate::errors::AppError> {
    core.get_session(&session_id).await
}

#[tauri::command]
pub async fn approve_command(
    app: tauri::AppHandle,
    core: State<'_, Arc<AgentCore>>,
    approval_id: String,
) -> Result<(), crate::errors::AppError> {
    core.approve_command(app, &approval_id).await
}

#[tauri::command]
pub async fn reject_command(
    app: tauri::AppHandle,
    core: State<'_, Arc<AgentCore>>,
    approval_id: String,
) -> Result<(), crate::errors::AppError> {
    core.reject_command(app, &approval_id).await
}

fn parse_session_type(s: &str) -> Result<AgentSessionType, crate::errors::AppError> {
    match s {
        "external_cli" => Ok(AgentSessionType::ExternalCli),
        "friday_owned_cli" => Ok(AgentSessionType::FridayOwnedCli),
        "cursor_sdk_local" => Ok(AgentSessionType::CursorSdkLocal),
        "cursor_cloud" => Ok(AgentSessionType::CursorCloud),
        _ => Err(crate::errors::AppError::Other(format!(
            "unknown session type: {s}"
        ))),
    }
}
