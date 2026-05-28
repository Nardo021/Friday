use tauri::State;

use crate::adapters::adapter_trait::AdapterInfo;
use crate::core::AgentCore;

#[tauri::command]
pub fn list_adapters(core: State<'_, AgentCore>) -> Result<Vec<AdapterInfo>, crate::errors::AppError> {
    Ok(core.adapter_registry.list())
}
