use tauri::State;

use crate::core::event::AgentSession;
use crate::core::AgentCore;
use crate::errors::AppResult;

#[tauri::command]
pub async fn start_agent_session(
    app: tauri::AppHandle,
    core: State<'_, AgentCore>,
    project_id: String,
    prompt: String,
) -> Result<AgentSession, crate::errors::AppError> {
    core.start_session(app, project_id, prompt).await
}

#[tauri::command]
pub async fn send_agent_message(
    app: tauri::AppHandle,
    core: State<'_, AgentCore>,
    session_id: String,
    message: String,
) -> Result<(), crate::errors::AppError> {
    core.send_message(app, &session_id, message).await
}

#[tauri::command]
pub async fn stop_agent_session(
    app: tauri::AppHandle,
    core: State<'_, AgentCore>,
    session_id: String,
) -> Result<(), crate::errors::AppError> {
    core.stop_session(app, &session_id).await
}

#[tauri::command]
pub async fn get_session_status(
    core: State<'_, AgentCore>,
    session_id: String,
) -> Result<AgentSession, crate::errors::AppError> {
    core.get_session(&session_id).await
}

#[tauri::command]
pub async fn approve_command(
    app: tauri::AppHandle,
    core: State<'_, AgentCore>,
    approval_id: String,
) -> Result<(), crate::errors::AppError> {
    core.approve_command(app, &approval_id).await
}

#[tauri::command]
pub async fn reject_command(
    app: tauri::AppHandle,
    core: State<'_, AgentCore>,
    approval_id: String,
) -> Result<(), crate::errors::AppError> {
    core.reject_command(app, &approval_id).await
}
