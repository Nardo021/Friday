use tauri::{AppHandle, Emitter};

use crate::core::event::AgentEvent;
use crate::errors::AppResult;

pub const AGENT_EVENT_CHANNEL: &str = "agent-event";

pub fn emit_agent_event(app: &AppHandle, event: &AgentEvent) -> AppResult<()> {
    app.emit(AGENT_EVENT_CHANNEL, event)
        .map_err(|e| crate::errors::AppError::Other(e.to_string()))
}
