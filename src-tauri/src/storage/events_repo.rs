use rusqlite::params;
use uuid::Uuid;

use crate::core::event::AgentEvent;
use crate::errors::{AppError, AppResult};
use crate::security::SecretRedactor;
use crate::storage::sqlite::Database;

pub struct EventsRepo<'a> {
    db: &'a Database,
}

impl<'a> EventsRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn insert(&self, event: &AgentEvent) -> AppResult<()> {
        let payload = serde_json::to_string(event)?;
        let redacted = SecretRedactor::redact(&payload);
        let event_type = event_type_name(event);
        let session_id = event.session_id().to_string();
        let created_at = extract_timestamp(event);

        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO events (id, session_id, type, payload_json, created_at) VALUES (?1,?2,?3,?4,?5)",
                params![Uuid::new_v4().to_string(), session_id, event_type, redacted, created_at],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    pub fn list_for_session(&self, session_id: &str) -> AppResult<Vec<AgentEvent>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT payload_json FROM events WHERE session_id = ?1 ORDER BY created_at ASC",
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;

            let rows = stmt
                .query_map(params![session_id], |row| row.get::<_, String>(0))
                .map_err(|e| AppError::Storage(e.to_string()))?;

            let mut events = Vec::new();
            for row in rows {
                let json: String = row.map_err(|e| AppError::Storage(e.to_string()))?;
                if let Ok(event) = serde_json::from_str(&json) {
                    events.push(event);
                }
            }
            Ok(events)
        })
    }
}

fn event_type_name(event: &AgentEvent) -> &'static str {
    match event {
        AgentEvent::AgentStatus { .. } => "agent.status",
        AgentEvent::AgentMessage { .. } => "agent.message",
        AgentEvent::ToolStarted { .. } => "tool.started",
        AgentEvent::ToolCompleted { .. } => "tool.completed",
        AgentEvent::FileChanged { .. } => "file.changed",
        AgentEvent::CommandStarted { .. } => "command.started",
        AgentEvent::CommandCompleted { .. } => "command.completed",
        AgentEvent::ApprovalRequired { .. } => "approval.required",
        AgentEvent::SessionStarted { .. } => "session.started",
        AgentEvent::SessionCompleted { .. } => "session.completed",
        AgentEvent::SessionError { .. } => "session.error",
    }
}

fn extract_timestamp(event: &AgentEvent) -> String {
    match event {
        AgentEvent::AgentStatus { timestamp, .. }
        | AgentEvent::AgentMessage { timestamp, .. }
        | AgentEvent::ToolStarted { timestamp, .. }
        | AgentEvent::ToolCompleted { timestamp, .. }
        | AgentEvent::FileChanged { timestamp, .. }
        | AgentEvent::CommandStarted { timestamp, .. }
        | AgentEvent::CommandCompleted { timestamp, .. }
        | AgentEvent::ApprovalRequired { timestamp, .. }
        | AgentEvent::SessionStarted { timestamp, .. }
        | AgentEvent::SessionCompleted { timestamp, .. }
        | AgentEvent::SessionError { timestamp, .. } => timestamp.clone(),
    }
}
