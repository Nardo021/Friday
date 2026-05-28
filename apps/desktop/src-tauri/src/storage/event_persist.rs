use rusqlite::params;
use uuid::Uuid;

use crate::core::event::{AgentEvent, FridaySessionStatus, MessageRole};
use crate::errors::{AppError, AppResult};
use crate::security::{DataCrypto, SecretRedactor};
use crate::storage::sqlite::Database;

/// Skip noisy events that should not hit storage or the UI bridge.
pub fn should_skip_event(event: &AgentEvent) -> bool {
    match event {
        AgentEvent::AgentMessage { content, .. } => content.trim().is_empty(),
        _ => false,
    }
}

/// Persist event, optional message row, and session status in one DB lock.
pub fn persist_agent_event(db: &Database, event: &AgentEvent, payload_json: &str) -> AppResult<()> {
    if should_skip_event(event) {
        return Ok(());
    }

    let encrypted = DataCrypto::encrypt(payload_json)?;
    let event_type = event_type_name(event);
    let session_id = event.session_id().to_string();
    let created_at = extract_timestamp(event);

    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO session_events (id, session_id, type, payload_json, created_at) VALUES (?1,?2,?3,?4,?5)",
            params![
                Uuid::new_v4().to_string(),
                session_id,
                event_type,
                encrypted,
                created_at
            ],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;

        if let AgentEvent::AgentMessage {
            session_id,
            role,
            content,
            timestamp,
        } = event
        {
            let content = SecretRedactor::redact(content);
            let content = DataCrypto::encrypt(&content)?;
            conn.execute(
                "INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?1,?2,?3,?4,?5)",
                params![
                    Uuid::new_v4().to_string(),
                    session_id,
                    role_to_str(*role),
                    content,
                    timestamp
                ],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        }

        if !session_id.is_empty() {
            if let AgentEvent::AgentStatus { status, .. } = event {
                conn.execute(
                    "UPDATE sessions SET status = ?1, updated_at = ?2 WHERE id = ?3",
                    params![status_to_str(*status), created_at, session_id],
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            } else if let AgentEvent::SessionCompleted { summary, .. } = event {
                conn.execute(
                    "UPDATE sessions SET status = ?1, summary = ?2, updated_at = ?3 WHERE id = ?4",
                    params![
                        "done",
                        summary.as_deref(),
                        created_at,
                        session_id
                    ],
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            } else if let AgentEvent::SessionError { .. } = event {
                conn.execute(
                    "UPDATE sessions SET status = ?1, updated_at = ?2 WHERE id = ?3",
                    params!["error", created_at, session_id],
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            }
        }

        Ok(())
    })
}

fn event_type_name(event: &AgentEvent) -> &'static str {
    match event {
        AgentEvent::SessionDiscovered { .. } => "session.discovered",
        AgentEvent::SessionStarted { .. } => "session.started",
        AgentEvent::AgentStatus { .. } => "agent.status",
        AgentEvent::AgentMessage { .. } => "agent.message",
        AgentEvent::ToolCall { .. } => "tool.call",
        AgentEvent::FileChanged { .. } => "file.changed",
        AgentEvent::CommandStarted { .. } => "command.started",
        AgentEvent::CommandCompleted { .. } => "command.completed",
        AgentEvent::ApprovalRequired { .. } => "approval.required",
        AgentEvent::ArtifactCreated { .. } => "artifact.created",
        AgentEvent::PrCreated { .. } => "pr.created",
        AgentEvent::SessionCompleted { .. } => "session.completed",
        AgentEvent::SessionError { .. } => "session.error",
    }
}

fn extract_timestamp(event: &AgentEvent) -> String {
    match event {
        AgentEvent::SessionDiscovered { timestamp, .. }
        | AgentEvent::SessionStarted { timestamp, .. }
        | AgentEvent::AgentStatus { timestamp, .. }
        | AgentEvent::AgentMessage { timestamp, .. }
        | AgentEvent::ToolCall { timestamp, .. }
        | AgentEvent::FileChanged { timestamp, .. }
        | AgentEvent::CommandStarted { timestamp, .. }
        | AgentEvent::CommandCompleted { timestamp, .. }
        | AgentEvent::ApprovalRequired { timestamp, .. }
        | AgentEvent::ArtifactCreated { timestamp, .. }
        | AgentEvent::PrCreated { timestamp, .. }
        | AgentEvent::SessionCompleted { timestamp, .. }
        | AgentEvent::SessionError { timestamp, .. } => timestamp.clone(),
    }
}

fn role_to_str(role: MessageRole) -> &'static str {
    match role {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
    }
}

fn status_to_str(status: FridaySessionStatus) -> &'static str {
    match status {
        FridaySessionStatus::Discovered => "discovered",
        FridaySessionStatus::Idle => "idle",
        FridaySessionStatus::Starting => "starting",
        FridaySessionStatus::Thinking => "thinking",
        FridaySessionStatus::Reading => "reading",
        FridaySessionStatus::Editing => "editing",
        FridaySessionStatus::RunningCommand => "running_command",
        FridaySessionStatus::WaitingPermission => "waiting_permission",
        FridaySessionStatus::Testing => "testing",
        FridaySessionStatus::Done => "done",
        FridaySessionStatus::Error => "error",
        FridaySessionStatus::Stopped => "stopped",
    }
}
