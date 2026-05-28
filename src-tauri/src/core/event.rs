use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Starting,
    Thinking,
    Reading,
    Editing,
    RunningCommand,
    WaitingApproval,
    Testing,
    Paused,
    Completed,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    AgentStatus {
        session_id: String,
        status: AgentStatus,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        timestamp: String,
    },
    AgentMessage {
        session_id: String,
        role: MessageRole,
        text: String,
        timestamp: String,
    },
    ToolStarted {
        session_id: String,
        tool: String,
        title: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<serde_json::Value>,
        timestamp: String,
    },
    ToolCompleted {
        session_id: String,
        tool: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<String>,
        timestamp: String,
    },
    FileChanged {
        session_id: String,
        path: String,
        action: FileAction,
        timestamp: String,
    },
    CommandStarted {
        session_id: String,
        command: String,
        cwd: String,
        risk: RiskLevel,
        timestamp: String,
    },
    CommandCompleted {
        session_id: String,
        command: String,
        exit_code: i32,
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<String>,
        timestamp: String,
    },
    ApprovalRequired {
        session_id: String,
        approval_id: String,
        title: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        command: Option<String>,
        risk: RiskLevel,
        timestamp: String,
    },
    SessionStarted {
        session_id: String,
        adapter_id: String,
        project_id: String,
        timestamp: String,
    },
    SessionCompleted {
        session_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        summary: Option<String>,
        timestamp: String,
    },
    SessionError {
        session_id: String,
        message: String,
        timestamp: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileAction {
    Created,
    Edited,
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSession {
    pub id: String,
    pub title: String,
    pub adapter_id: String,
    pub project_id: String,
    pub cwd: String,
    pub status: AgentStatus,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_type: Option<String>,
    pub trusted: bool,
    pub default_adapter_id: String,
    pub created_at: String,
    pub last_used_at: String,
}

impl AgentEvent {
    pub fn session_id(&self) -> &str {
        match self {
            AgentEvent::AgentStatus { session_id, .. }
            | AgentEvent::AgentMessage { session_id, .. }
            | AgentEvent::ToolStarted { session_id, .. }
            | AgentEvent::ToolCompleted { session_id, .. }
            | AgentEvent::FileChanged { session_id, .. }
            | AgentEvent::CommandStarted { session_id, .. }
            | AgentEvent::CommandCompleted { session_id, .. }
            | AgentEvent::ApprovalRequired { session_id, .. }
            | AgentEvent::SessionStarted { session_id, .. }
            | AgentEvent::SessionCompleted { session_id, .. }
            | AgentEvent::SessionError { session_id, .. } => session_id,
        }
    }
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}
