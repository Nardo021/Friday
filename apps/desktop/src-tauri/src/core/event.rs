use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentSessionType {
    ExternalCli,
    FridayOwnedCli,
    CursorSdkLocal,
    CursorCloud,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionOwnership {
    External,
    Friday,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlLevel {
    None,
    Observe,
    Partial,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FridaySessionStatus {
    Discovered,
    Idle,
    Starting,
    Thinking,
    Reading,
    Editing,
    RunningCommand,
    WaitingPermission,
    Testing,
    Done,
    Error,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRepo {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionProcess {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pty_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCloud {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FridaySession {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub session_type: AgentSessionType,
    pub ownership: SessionOwnership,
    pub adapter_id: String,
    pub status: FridaySessionStatus,
    pub control_level: ControlLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<SessionRepo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process: Option<SessionProcess>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud: Option<SessionCloud>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoverySource {
    ProcessScan,
    Api,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AgentEvent {
    #[serde(rename = "session.discovered")]
    SessionDiscovered {
        session_id: String,
        source: DiscoverySource,
        timestamp: String,
    },
    #[serde(rename = "session.started")]
    SessionStarted {
        session_id: String,
        timestamp: String,
    },
    #[serde(rename = "agent.status")]
    AgentStatus {
        session_id: String,
        status: FridaySessionStatus,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        timestamp: String,
    },
    #[serde(rename = "agent.message")]
    AgentMessage {
        session_id: String,
        role: MessageRole,
        content: String,
        timestamp: String,
    },
    #[serde(rename = "tool.call")]
    ToolCall {
        session_id: String,
        tool_name: String,
        title: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<serde_json::Value>,
        timestamp: String,
    },
    #[serde(rename = "file.changed")]
    FileChanged {
        session_id: String,
        path: String,
        action: FileAction,
        timestamp: String,
    },
    #[serde(rename = "command.started")]
    CommandStarted {
        session_id: String,
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cwd: Option<String>,
        risk: RiskLevel,
        timestamp: String,
    },
    #[serde(rename = "command.completed")]
    CommandCompleted {
        session_id: String,
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        exit_code: Option<i32>,
        timestamp: String,
    },
    #[serde(rename = "approval.required")]
    ApprovalRequired {
        session_id: String,
        approval_id: String,
        title: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        command: Option<String>,
        risk: RiskLevel,
        timestamp: String,
    },
    #[serde(rename = "artifact.created")]
    ArtifactCreated {
        session_id: String,
        artifact_id: String,
        title: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        timestamp: String,
    },
    #[serde(rename = "pr.created")]
    PrCreated {
        session_id: String,
        pr_url: String,
        timestamp: String,
    },
    #[serde(rename = "session.completed")]
    SessionCompleted {
        session_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        summary: Option<String>,
        timestamp: String,
    },
    #[serde(rename = "session.error")]
    SessionError {
        session_id: String,
        error: String,
        timestamp: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilities {
    pub can_create: bool,
    pub can_attach: bool,
    pub can_observe: bool,
    pub can_send_follow_up: bool,
    pub can_stop: bool,
    pub can_resume: bool,
    pub can_stream_events: bool,
    pub can_read_artifacts: bool,
    pub can_open_pr: bool,
    pub control_level: ControlLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdapterInfo {
    pub id: String,
    pub name: String,
    pub available: bool,
    pub session_type: AgentSessionType,
    pub capabilities: AgentCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_url: Option<String>,
    pub trusted: bool,
    pub default_adapter_id: String,
    pub created_at: String,
    pub last_used_at: String,
}

impl AgentEvent {
    pub fn session_id(&self) -> &str {
        match self {
            AgentEvent::SessionDiscovered { session_id, .. }
            | AgentEvent::SessionStarted { session_id, .. }
            | AgentEvent::AgentStatus { session_id, .. }
            | AgentEvent::AgentMessage { session_id, .. }
            | AgentEvent::ToolCall { session_id, .. }
            | AgentEvent::FileChanged { session_id, .. }
            | AgentEvent::CommandStarted { session_id, .. }
            | AgentEvent::CommandCompleted { session_id, .. }
            | AgentEvent::ApprovalRequired { session_id, .. }
            | AgentEvent::ArtifactCreated { session_id, .. }
            | AgentEvent::PrCreated { session_id, .. }
            | AgentEvent::SessionCompleted { session_id, .. }
            | AgentEvent::SessionError { session_id, .. } => session_id,
        }
    }
}

pub fn is_running_status(status: FridaySessionStatus) -> bool {
    matches!(
        status,
        FridaySessionStatus::Starting
            | FridaySessionStatus::Thinking
            | FridaySessionStatus::Reading
            | FridaySessionStatus::Editing
            | FridaySessionStatus::RunningCommand
            | FridaySessionStatus::WaitingPermission
            | FridaySessionStatus::Testing
    )
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}
