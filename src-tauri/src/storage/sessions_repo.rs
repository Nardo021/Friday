use rusqlite::params;

use crate::core::event::{AgentSession, AgentStatus, now_iso};
use crate::errors::{AppError, AppResult};
use crate::storage::sqlite::Database;

pub struct SessionsRepo<'a> {
    db: &'a Database,
}

impl<'a> SessionsRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn insert(&self, session: &AgentSession) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO sessions (id, project_id, adapter_id, title, prompt, status, cwd, pid, summary, created_at, started_at, completed_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
                params![
                    session.id,
                    session.project_id,
                    session.adapter_id,
                    session.title,
                    session.prompt,
                    status_to_str(session.status),
                    session.cwd,
                    session.pid,
                    session.summary,
                    session.created_at,
                    session.started_at,
                    session.completed_at,
                ],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    pub fn update_status(&self, id: &str, status: AgentStatus, summary: Option<&str>) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE sessions SET status = ?1, summary = COALESCE(?2, summary), completed_at = ?3 WHERE id = ?4",
                params![
                    status_to_str(status),
                    summary,
                    if matches!(status, AgentStatus::Completed | AgentStatus::Error | AgentStatus::Cancelled) {
                        Some(now_iso())
                    } else {
                        None::<String>
                    },
                    id,
                ],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    pub fn list(&self) -> AppResult<Vec<AgentSession>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, project_id, adapter_id, title, prompt, status, cwd, pid, summary, created_at, started_at, completed_at
                     FROM sessions ORDER BY created_at DESC",
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;

            let rows = stmt
                .query_map([], |row| {
                    Ok(AgentSession {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        adapter_id: row.get(2)?,
                        title: row.get(3)?,
                        prompt: row.get(4)?,
                        status: str_to_status(row.get::<_, String>(5)?),
                        cwd: row.get(6)?,
                        pid: row.get(7)?,
                        summary: row.get(8)?,
                        created_at: row.get(9)?,
                        started_at: row.get(10)?,
                        completed_at: row.get(11)?,
                        model: None,
                        branch: None,
                    })
                })
                .map_err(|e| AppError::Storage(e.to_string()))?;

            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| AppError::Storage(e.to_string()))
        })
    }

    pub fn get(&self, id: &str) -> AppResult<AgentSession> {
        self.db.with_conn(|conn| {
            conn.query_row(
                "SELECT id, project_id, adapter_id, title, prompt, status, cwd, pid, summary, created_at, started_at, completed_at
                 FROM sessions WHERE id = ?1",
                params![id],
                |row| {
                    Ok(AgentSession {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        adapter_id: row.get(2)?,
                        title: row.get(3)?,
                        prompt: row.get(4)?,
                        status: str_to_status(row.get::<_, String>(5)?),
                        cwd: row.get(6)?,
                        pid: row.get(7)?,
                        summary: row.get(8)?,
                        created_at: row.get(9)?,
                        started_at: row.get(10)?,
                        completed_at: row.get(11)?,
                        model: None,
                        branch: None,
                    })
                },
            )
            .map_err(|_| AppError::SessionNotFound(id.to_string()))
        })
    }
}

fn status_to_str(status: AgentStatus) -> &'static str {
    match status {
        AgentStatus::Idle => "idle",
        AgentStatus::Starting => "starting",
        AgentStatus::Thinking => "thinking",
        AgentStatus::Reading => "reading",
        AgentStatus::Editing => "editing",
        AgentStatus::RunningCommand => "running_command",
        AgentStatus::WaitingApproval => "waiting_approval",
        AgentStatus::Testing => "testing",
        AgentStatus::Paused => "paused",
        AgentStatus::Completed => "completed",
        AgentStatus::Error => "error",
        AgentStatus::Cancelled => "cancelled",
    }
}

fn str_to_status(s: String) -> AgentStatus {
    match s.as_str() {
        "idle" => AgentStatus::Idle,
        "starting" => AgentStatus::Starting,
        "thinking" => AgentStatus::Thinking,
        "reading" => AgentStatus::Reading,
        "editing" => AgentStatus::Editing,
        "running_command" => AgentStatus::RunningCommand,
        "waiting_approval" => AgentStatus::WaitingApproval,
        "testing" => AgentStatus::Testing,
        "paused" => AgentStatus::Paused,
        "completed" => AgentStatus::Completed,
        "error" => AgentStatus::Error,
        "cancelled" => AgentStatus::Cancelled,
        _ => AgentStatus::Idle,
    }
}
