use rusqlite::params;

use crate::core::event::now_iso;
use crate::errors::{AppError, AppResult};
use crate::storage::sqlite::Database;

#[derive(Debug, Clone)]
pub struct CloudAgentRecord {
    pub id: String,
    pub session_id: String,
    pub agent_id: String,
    pub run_id: Option<String>,
    pub pr_url: Option<String>,
    pub status: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct CloudAgentsRepo<'a> {
    db: &'a Database,
}

impl<'a> CloudAgentsRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn upsert(
        &self,
        session_id: &str,
        agent_id: &str,
        run_id: Option<&str>,
        pr_url: Option<&str>,
        status: Option<&str>,
    ) -> AppResult<()> {
        let now = now_iso();
        self.db.with_conn(|conn| {
            let existing: Option<String> = conn
                .query_row(
                    "SELECT id FROM cloud_agents WHERE session_id = ?1",
                    params![session_id],
                    |row| row.get(0),
                )
                .ok();

            if let Some(id) = existing {
                conn.execute(
                    "UPDATE cloud_agents SET agent_id = ?2, run_id = ?3, pr_url = ?4,
                     status = ?5, updated_at = ?6 WHERE id = ?1",
                    params![id, agent_id, run_id, pr_url, status, now],
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            } else {
                let id = uuid::Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO cloud_agents (id, session_id, agent_id, run_id, pr_url, status, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![id, session_id, agent_id, run_id, pr_url, status, now, now],
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            }
            Ok(())
        })
    }

    pub fn update_run(&self, session_id: &str, run_id: &str, status: Option<&str>) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE cloud_agents SET run_id = ?2, status = COALESCE(?3, status), updated_at = ?4
                 WHERE session_id = ?1",
                params![session_id, run_id, status, now_iso()],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    pub fn update_pr_url(&self, session_id: &str, pr_url: &str) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE cloud_agents SET pr_url = ?2, updated_at = ?3 WHERE session_id = ?1",
                params![session_id, pr_url, now_iso()],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    pub fn get_by_session(&self, session_id: &str) -> AppResult<CloudAgentRecord> {
        self.db.with_conn(|conn| {
            conn.query_row(
                "SELECT id, session_id, agent_id, run_id, pr_url, status, created_at, updated_at
                 FROM cloud_agents WHERE session_id = ?1",
                params![session_id],
                |row| {
                    Ok(CloudAgentRecord {
                        id: row.get(0)?,
                        session_id: row.get(1)?,
                        agent_id: row.get(2)?,
                        run_id: row.get(3)?,
                        pr_url: row.get(4)?,
                        status: row.get(5)?,
                        created_at: row.get(6)?,
                        updated_at: row.get(7)?,
                    })
                },
            )
            .map_err(|_| AppError::Other(format!("cloud agent record not found for {session_id}")))
        })
    }

    pub fn list_with_run(&self) -> AppResult<Vec<CloudAgentRecord>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, session_id, agent_id, run_id, pr_url, status, created_at, updated_at
                     FROM cloud_agents WHERE run_id IS NOT NULL",
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;

            let rows = stmt
                .query_map([], |row| {
                    Ok(CloudAgentRecord {
                        id: row.get(0)?,
                        session_id: row.get(1)?,
                        agent_id: row.get(2)?,
                        run_id: row.get(3)?,
                        pr_url: row.get(4)?,
                        status: row.get(5)?,
                        created_at: row.get(6)?,
                        updated_at: row.get(7)?,
                    })
                })
                .map_err(|e| AppError::Storage(e.to_string()))?;

            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| AppError::Storage(e.to_string()))
        })
    }
}
