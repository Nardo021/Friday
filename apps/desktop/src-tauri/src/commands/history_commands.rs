use std::sync::Arc;

use tauri::State;

use crate::core::AgentCore;
use crate::storage::{IdeasRepo, MessagesRepo};

#[tauri::command]
pub fn list_ideas(
    core: State<'_, Arc<AgentCore>>,
) -> Result<Vec<crate::storage::ideas_repo::Idea>, crate::errors::AppError> {
    IdeasRepo::new(&core.db).list()
}

#[tauri::command]
pub fn delete_idea(
    core: State<'_, Arc<AgentCore>>,
    id: String,
) -> Result<(), crate::errors::AppError> {
    IdeasRepo::new(&core.db).delete(&id)
}

#[tauri::command]
pub fn list_messages(
    core: State<'_, Arc<AgentCore>>,
    session_id: String,
) -> Result<Vec<crate::storage::StoredMessage>, crate::errors::AppError> {
    MessagesRepo::new(&core.db).list_for_session(&session_id)
}

#[tauri::command]
pub fn export_session_markdown(
    core: State<'_, Arc<AgentCore>>,
    session_id: String,
) -> Result<String, crate::errors::AppError> {
    core.export_session_markdown(&session_id)
}

#[tauri::command]
pub fn delete_session(
    core: State<'_, Arc<AgentCore>>,
    session_id: String,
) -> Result<(), crate::errors::AppError> {
    core.delete_session(&session_id)
}
