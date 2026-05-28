use rusqlite::params;
use uuid::Uuid;

use crate::core::event::now_iso;
use crate::errors::{AppError, AppResult};
use crate::storage::sqlite::Database;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuedInstruction {
    pub id: String,
    pub session_id: String,
    pub text: String,
    pub created_at: String,
}

pub struct QueuedInstructionsRepo<'a> {
    db: &'a Database,
}

impl<'a> QueuedInstructionsRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn enqueue(&self, session_id: &str, text: &str) -> AppResult<QueuedInstruction> {
        let item = QueuedInstruction {
            id: Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            text: text.to_string(),
            created_at: now_iso(),
        };
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO queued_instructions (id, session_id, text, created_at) VALUES (?1,?2,?3,?4)",
                params![item.id, item.session_id, item.text, item.created_at],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })?;
        Ok(item)
    }

    pub fn list_for_session(&self, session_id: &str) -> AppResult<Vec<QueuedInstruction>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, session_id, text, created_at FROM queued_instructions WHERE session_id = ?1 ORDER BY created_at ASC",
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let rows = stmt
                .query_map(params![session_id], |row| {
                    Ok(QueuedInstruction {
                        id: row.get(0)?,
                        session_id: row.get(1)?,
                        text: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                })
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let mut items = Vec::new();
            for row in rows {
                items.push(row.map_err(|e| AppError::Storage(e.to_string()))?);
            }
            Ok(items)
        })
    }

    pub fn pop_next(&self, session_id: &str) -> AppResult<Option<QueuedInstruction>> {
        let items = self.list_for_session(session_id)?;
        let Some(first) = items.first() else {
            return Ok(None);
        };
        let id = first.id.clone();
        let item = first.clone();
        self.db.with_conn(|conn| {
            conn.execute("DELETE FROM queued_instructions WHERE id = ?1", params![id])
                .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })?;
        Ok(Some(item))
    }
}
