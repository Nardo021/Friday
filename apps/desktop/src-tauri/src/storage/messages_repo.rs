use rusqlite::params;
use uuid::Uuid;

use crate::core::event::{AgentEvent, MessageRole};
use crate::errors::{AppError, AppResult};
use crate::security::{DataCrypto, SecretRedactor};
use crate::storage::sqlite::Database;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

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
            content,
            timestamp,
        } = event
        {
            let content = SecretRedactor::redact(content);
            let content = DataCrypto::encrypt(&content)?;
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

    pub fn list_for_session(&self, session_id: &str) -> AppResult<Vec<StoredMessage>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, session_id, role, content, created_at FROM messages WHERE session_id = ?1 ORDER BY created_at ASC",
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let rows = stmt
                .query_map(params![session_id], |row| {
                    Ok(StoredMessage {
                        id: row.get(0)?,
                        session_id: row.get(1)?,
                        role: row.get(2)?,
                        content: row.get(3)?,
                        created_at: row.get(4)?,
                    })
                })
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let mut messages = Vec::new();
            for row in rows {
                let mut msg = row.map_err(|e| AppError::Storage(e.to_string()))?;
                msg.content = DataCrypto::decrypt(&msg.content)?;
                messages.push(msg);
            }
            Ok(messages)
        })
    }
}

fn role_to_str(role: MessageRole) -> &'static str {
    match role {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
    }
}
