use tauri::State;

use crate::core::event::AgentSession;
use crate::core::AgentCore;
use crate::storage::{EventsRepo, SessionsRepo};

#[tauri::command]
pub fn list_sessions(core: State<'_, AgentCore>) -> Result<Vec<AgentSession>, crate::errors::AppError> {
    core.list_sessions()
}

#[tauri::command]
pub fn get_session_detail(
    core: State<'_, AgentCore>,
    session_id: String,
) -> Result<AgentSession, crate::errors::AppError> {
    SessionsRepo::new(&core.db).get(&session_id)
}

#[tauri::command]
pub fn get_session_events(
    core: State<'_, AgentCore>,
    session_id: String,
) -> Result<Vec<crate::core::event::AgentEvent>, crate::errors::AppError> {
    EventsRepo::new(&core.db).list_for_session(&session_id)
}
