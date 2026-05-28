use rusqlite::params;
use uuid::Uuid;

use crate::core::event::{AgentEvent, MessageRole};
use crate::errors::{AppError, AppResult};
use crate::security::SecretRedactor;
use crate::storage::sqlite::Database;

pub struct MessagesRepo<'a> {
    db: &'a Database,
}

impl<'a> MessagesRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn insert_from_event(&self, event: &AgentEvent) -> AppResult<()> {
        if let AgentEvent::AgentMessage {
            session_id,
            role,
            text,
            timestamp,
        } = event
        {
            let content = SecretRedactor::redact(text);
            self.db.with_conn(|conn| {
                conn.execute(
                    "INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?1,?2,?3,?4,?5)",
                    params![
                        Uuid::new_v4().to_string(),
                        session_id,
                        role_to_str(*role),
                        content,
                        timestamp,
                    ],
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
                Ok(())
            })?;
        }
        Ok(())
    }
}

fn role_to_str(role: MessageRole) -> &'static str {
    match role {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
    }
}
