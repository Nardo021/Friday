use rusqlite::params;
use uuid::Uuid;

use crate::core::event::now_iso;
use crate::errors::{AppError, AppResult};
use crate::storage::sqlite::Database;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Idea {
    pub id: String,
    pub title: String,
    pub body: String,
    pub project_id: Option<String>,
    pub session_id: Option<String>,
    pub created_at: String,
}

pub struct IdeasRepo<'a> {
    db: &'a Database,
}

impl<'a> IdeasRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn insert(
        &self,
        title: &str,
        body: &str,
        project_id: Option<&str>,
        session_id: Option<&str>,
    ) -> AppResult<Idea> {
        let idea = Idea {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            body: body.to_string(),
            project_id: project_id.map(str::to_string),
            session_id: session_id.map(str::to_string),
            created_at: now_iso(),
        };
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO ideas (id, title, body, project_id, session_id, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
                params![
                    idea.id,
                    idea.title,
                    idea.body,
                    idea.project_id,
                    idea.session_id,
                    idea.created_at,
                ],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })?;
        Ok(idea)
    }

    pub fn list(&self) -> AppResult<Vec<Idea>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, title, body, project_id, session_id, created_at FROM ideas ORDER BY created_at DESC",
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(Idea {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        body: row.get(2)?,
                        project_id: row.get(3)?,
                        session_id: row.get(4)?,
                        created_at: row.get(5)?,
                    })
                })
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let mut ideas = Vec::new();
            for row in rows {
                ideas.push(row.map_err(|e| AppError::Storage(e.to_string()))?);
            }
            Ok(ideas)
        })
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute("DELETE FROM ideas WHERE id = ?1", params![id])
                .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }
}
