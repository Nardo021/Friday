use rusqlite::params;

use crate::core::event::{
    AgentSessionType, ControlLevel, FridaySession, FridaySessionStatus, SessionCloud,
    SessionOwnership, SessionProcess,
};
use crate::core::event::now_iso;
use crate::errors::{AppError, AppResult};
use crate::storage::sqlite::Database;

pub struct SessionsRepo<'a> {
    db: &'a Database,
}

impl<'a> SessionsRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn insert(&self, session: &FridaySession) -> AppResult<()> {
        let repo_json = session
            .repo
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let process_json = session
            .process
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let cloud_json = session
            .cloud
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let cwd = session
            .process
            .as_ref()
            .and_then(|p| p.cwd.clone())
            .or_else(|| session.repo.as_ref().and_then(|r| r.local_path.clone()));
        let pid = session.process.as_ref().and_then(|p| p.pid);

        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO sessions (id, project_id, adapter_id, title, prompt, status, cwd, pid, summary,
                 created_at, started_at, completed_at, session_type, ownership, control_level, updated_at,
                 repo_json, process_json, cloud_json)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
                params![
                    session.id,
                    session.project_id,
                    session.adapter_id,
                    session.title,
                    session.prompt,
                    status_to_str(session.status),
                    cwd,
                    pid,
                    session.summary,
                    session.created_at,
                    session.started_at,
                    session.completed_at,
                    session_type_to_str(session.session_type),
                    ownership_to_str(session.ownership),
                    control_level_to_str(session.control_level),
                    session.updated_at,
                    repo_json,
                    process_json,
                    cloud_json,
                ],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    pub fn upsert(&self, session: &FridaySession) -> AppResult<()> {
        if self.get(&session.id).is_ok() {
            self.update(session)
        } else {
            self.insert(session)
        }
    }

    pub fn update(&self, session: &FridaySession) -> AppResult<()> {
        let repo_json = session
            .repo
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let process_json = session
            .process
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let cloud_json = session
            .cloud
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let cwd = session
            .process
            .as_ref()
            .and_then(|p| p.cwd.clone());
        let pid = session.process.as_ref().and_then(|p| p.pid);

        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE sessions SET project_id=?2, adapter_id=?3, title=?4, prompt=?5, status=?6, cwd=?7,
                 pid=?8, summary=?9, started_at=?10, completed_at=?11, session_type=?12, ownership=?13,
                 control_level=?14, updated_at=?15, repo_json=?16, process_json=?17, cloud_json=?18 WHERE id=?1",
                params![
                    session.id,
                    session.project_id,
                    session.adapter_id,
                    session.title,
                    session.prompt,
                    status_to_str(session.status),
                    cwd,
                    pid,
                    session.summary,
                    session.started_at,
                    session.completed_at,
                    session_type_to_str(session.session_type),
                    ownership_to_str(session.ownership),
                    control_level_to_str(session.control_level),
                    session.updated_at,
                    repo_json,
                    process_json,
                    cloud_json,
                ],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    pub fn update_status(
        &self,
        id: &str,
        status: FridaySessionStatus,
        summary: Option<&str>,
    ) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE sessions SET status = ?1, summary = COALESCE(?2, summary),
                 completed_at = ?3, updated_at = ?4 WHERE id = ?5",
                params![
                    status_to_str(status),
                    summary,
                    if matches!(
                        status,
                        FridaySessionStatus::Done
                            | FridaySessionStatus::Error
                            | FridaySessionStatus::Stopped
                    ) {
                        Some(now_iso())
                    } else {
                        None::<String>
                    },
                    now_iso(),
                    id,
                ],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    pub fn list(&self) -> AppResult<Vec<FridaySession>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, project_id, adapter_id, title, prompt, status, cwd, pid, summary,
                     created_at, started_at, completed_at, session_type, ownership, control_level,
                     updated_at, repo_json, process_json, cloud_json
                     FROM sessions ORDER BY updated_at DESC, created_at DESC",
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;

            let rows = stmt
                .query_map([], map_session_row)
                .map_err(|e| AppError::Storage(e.to_string()))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| AppError::Storage(e.to_string()))
        })
    }

    pub fn get(&self, id: &str) -> AppResult<FridaySession> {
        self.db.with_conn(|conn| {
            conn.query_row(
                "SELECT id, project_id, adapter_id, title, prompt, status, cwd, pid, summary,
                 created_at, started_at, completed_at, session_type, ownership, control_level,
                 updated_at, repo_json, process_json, cloud_json
                 FROM sessions WHERE id = ?1",
                params![id],
                map_session_row,
            )
            .map_err(|_| AppError::SessionNotFound(id.to_string()))
        })
    }
}

fn map_session_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<FridaySession> {
    let id: String = row.get(0)?;
    let project_id: Option<String> = row.get(1)?;
    let adapter_id: String = row.get(2)?;
    let title: String = row.get(3)?;
    let prompt: Option<String> = row.get(4)?;
    let status: String = row.get(5)?;
    let cwd: Option<String> = row.get(6)?;
    let pid: Option<u32> = row.get(7)?;
    let summary: Option<String> = row.get(8)?;
    let created_at: String = row.get(9)?;
    let started_at: Option<String> = row.get(10)?;
    let completed_at: Option<String> = row.get(11)?;
    let session_type: Option<String> = row.get(12)?;
    let ownership: Option<String> = row.get(13)?;
    let control_level: Option<String> = row.get(14)?;
    let updated_at: Option<String> = row.get(15)?;
    let repo_json: Option<String> = row.get(16)?;
    let process_json: Option<String> = row.get(17)?;
    let cloud_json: Option<String> = row.get(18)?;

    let mut process: Option<SessionProcess> = process_json
        .as_ref()
        .and_then(|j| serde_json::from_str(j).ok());
    if process.is_none() && (pid.is_some() || cwd.is_some()) {
        process = Some(SessionProcess {
            pid,
            pty_id: None,
            cwd,
        });
    }

    Ok(FridaySession {
        id,
        title,
        session_type: session_type
            .map(|s| str_to_session_type(&s))
            .unwrap_or(AgentSessionType::FridayOwnedCli),
        ownership: ownership
            .map(|s| str_to_ownership(&s))
            .unwrap_or(SessionOwnership::Friday),
        adapter_id,
        status: str_to_status(status),
        control_level: control_level
            .map(|s| str_to_control_level(&s))
            .unwrap_or(ControlLevel::Full),
        project_id,
        prompt,
        summary,
        repo: repo_json.and_then(|j| serde_json::from_str(&j).ok()),
        process,
        cloud: cloud_json.and_then(|j| serde_json::from_str::<SessionCloud>(&j).ok()),
        created_at: created_at.clone(),
        started_at,
        updated_at: updated_at.unwrap_or(created_at),
        completed_at,
    })
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

fn str_to_status(s: String) -> FridaySessionStatus {
    match s.as_str() {
        "discovered" => FridaySessionStatus::Discovered,
        "idle" => FridaySessionStatus::Idle,
        "starting" => FridaySessionStatus::Starting,
        "thinking" => FridaySessionStatus::Thinking,
        "reading" => FridaySessionStatus::Reading,
        "editing" => FridaySessionStatus::Editing,
        "running_command" => FridaySessionStatus::RunningCommand,
        "waiting_permission" => FridaySessionStatus::WaitingPermission,
        "testing" => FridaySessionStatus::Testing,
        "done" | "completed" => FridaySessionStatus::Done,
        "error" => FridaySessionStatus::Error,
        "stopped" | "cancelled" => FridaySessionStatus::Stopped,
        _ => FridaySessionStatus::Idle,
    }
}

fn session_type_to_str(t: AgentSessionType) -> &'static str {
    match t {
        AgentSessionType::ExternalCli => "external_cli",
        AgentSessionType::FridayOwnedCli => "friday_owned_cli",
        AgentSessionType::CursorSdkLocal => "cursor_sdk_local",
        AgentSessionType::CursorCloud => "cursor_cloud",
    }
}

fn str_to_session_type(s: &str) -> AgentSessionType {
    match s {
        "external_cli" => AgentSessionType::ExternalCli,
        "cursor_sdk_local" => AgentSessionType::CursorSdkLocal,
        "cursor_cloud" => AgentSessionType::CursorCloud,
        _ => AgentSessionType::FridayOwnedCli,
    }
}

fn ownership_to_str(o: SessionOwnership) -> &'static str {
    match o {
        SessionOwnership::External => "external",
        SessionOwnership::Friday => "friday",
    }
}

fn str_to_ownership(s: &str) -> SessionOwnership {
    match s {
        "external" => SessionOwnership::External,
        _ => SessionOwnership::Friday,
    }
}

fn control_level_to_str(c: ControlLevel) -> &'static str {
    match c {
        ControlLevel::None => "none",
        ControlLevel::Observe => "observe",
        ControlLevel::Partial => "partial",
        ControlLevel::Full => "full",
    }
}

fn str_to_control_level(s: &str) -> ControlLevel {
    match s {
        "observe" => ControlLevel::Observe,
        "partial" => ControlLevel::Partial,
        "full" => ControlLevel::Full,
        _ => ControlLevel::None,
    }
}
