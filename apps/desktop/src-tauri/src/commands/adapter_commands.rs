use std::sync::Arc;

use tauri::State;

use crate::core::event::AdapterInfo;
use crate::core::AgentCore;

#[tauri::command]
pub fn list_adapters(core: State<'_, Arc<AgentCore>>) -> Result<Vec<AdapterInfo>, crate::errors::AppError> {
    Ok(core.adapter_registry.list())
}
