pub mod adapter_trait;
pub mod cursor;

use std::sync::Arc;

use adapter_trait::StartSessionInput;
use cursor::{CursorCommandBuilder, CursorEventMapper, CursorParser};
use crate::core::event::{AgentEvent, AgentStatus, MessageRole, now_iso};
use crate::errors::AppResult;
use crate::process::{ProcessSupervisor, SpawnConfig};
use crate::storage::settings_repo::CursorSettings;

pub struct AdapterRegistry {
    adapters: std::collections::HashMap<String, adapter_trait::AdapterInfo>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        let mut adapters = std::collections::HashMap::new();
        adapters.insert(
            "cursor-cli".into(),
            adapter_trait::AdapterInfo {
                id: "cursor-cli".into(),
                name: "Cursor CLI".into(),
                available: true,
                capabilities: cursor_capabilities(),
            },
        );
        adapters.insert(
            "claude-code".into(),
            adapter_trait::AdapterInfo {
                id: "claude-code".into(),
                name: "Claude Code".into(),
                available: false,
                capabilities: stub_capabilities(),
            },
        );
        adapters.insert(
            "codex-cli".into(),
            adapter_trait::AdapterInfo {
                id: "codex-cli".into(),
                name: "Codex CLI".into(),
                available: false,
                capabilities: stub_capabilities(),
            },
        );
        adapters.insert(
            "gemini-cli".into(),
            adapter_trait::AdapterInfo {
                id: "gemini-cli".into(),
                name: "Gemini CLI".into(),
                available: false,
                capabilities: stub_capabilities(),
            },
        );
        Self { adapters }
    }

    pub fn list(&self) -> Vec<adapter_trait::AdapterInfo> {
        self.adapters.values().cloned().collect()
    }

    pub fn get(&self, id: &str) -> crate::errors::AppResult<adapter_trait::AdapterInfo> {
        self.adapters
            .get(id)
            .cloned()
            .ok_or_else(|| crate::errors::AppError::AdapterNotFound(id.to_string()))
    }
}

pub fn cursor_capabilities() -> adapter_trait::AgentCapabilities {
    adapter_trait::AgentCapabilities {
        supports_streaming: true,
        supports_interactive_input: false,
        supports_approvals: true,
        supports_file_change_events: true,
        supports_command_events: true,
        supports_session_resume: false,
        supports_stop: true,
    }
}

fn stub_capabilities() -> adapter_trait::AgentCapabilities {
    adapter_trait::AgentCapabilities {
        supports_streaming: false,
        supports_interactive_input: false,
        supports_approvals: false,
        supports_file_change_events: false,
        supports_command_events: false,
        supports_session_resume: false,
        supports_stop: false,
    }
}

pub async fn start_cursor_session(
    supervisor: Arc<ProcessSupervisor>,
    input: StartSessionInput,
    cursor_settings: CursorSettings,
    event_handler: Arc<dyn Fn(AgentEvent) + Send + Sync>,
) -> AppResult<u32> {
    let built = CursorCommandBuilder::build(&input.prompt, &input.cwd, &cursor_settings)?;

    event_handler(AgentEvent::AgentStatus {
        session_id: input.session_id.clone(),
        status: AgentStatus::Starting,
        message: Some("Starting Cursor CLI".into()),
        timestamp: now_iso(),
    });

    event_handler(AgentEvent::AgentMessage {
        session_id: input.session_id.clone(),
        role: MessageRole::User,
        text: input.prompt.clone(),
        timestamp: now_iso(),
    });

    let (pid, mut rx) = supervisor
        .spawn_with_output(SpawnConfig {
            session_id: input.session_id.clone(),
            adapter_id: "cursor-cli".into(),
            executable: built.executable,
            args: built.args,
            cwd: built.cwd,
            env: built.env,
        })
        .await?;

    let session_id = input.session_id.clone();
    let handler = event_handler.clone();

    tauri::async_runtime::spawn(async move {
        while let Some((line, is_stderr)) = rx.recv().await {
            if is_stderr && !line.trim().is_empty() {
                handler(AgentEvent::AgentMessage {
                    session_id: session_id.clone(),
                    role: MessageRole::System,
                    text: line.clone(),
                    timestamp: now_iso(),
                });
            }

            let intermediate = CursorParser::parse_line(&line);
            let events = CursorEventMapper::map(&session_id, intermediate);
            for event in events {
                handler(event);
            }
        }
    });

    Ok(pid)
}
