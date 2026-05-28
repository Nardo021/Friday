use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilities {
    pub supports_streaming: bool,
    pub supports_interactive_input: bool,
    pub supports_approvals: bool,
    pub supports_file_change_events: bool,
    pub supports_command_events: bool,
    pub supports_session_resume: bool,
    pub supports_stop: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdapterInfo {
    pub id: String,
    pub name: String,
    pub available: bool,
    pub capabilities: AgentCapabilities,
}

#[derive(Debug, Clone)]
pub struct StartSessionInput {
    pub session_id: String,
    pub prompt: String,
    pub cwd: String,
    pub model: Option<String>,
}
