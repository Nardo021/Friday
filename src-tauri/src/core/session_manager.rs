use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::core::event::{AgentSession, AgentStatus, now_iso};
use crate::errors::{AppError, AppResult};

#[derive(Default)]
pub struct SessionManager {
    sessions: HashMap<String, AgentSession>,
    active_session_id: Option<String>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_session(
        &mut self,
        id: String,
        title: String,
        adapter_id: String,
        project_id: String,
        cwd: String,
        prompt: String,
    ) -> AgentSession {
        let session = AgentSession {
            id: id.clone(),
            title,
            adapter_id,
            project_id,
            cwd,
            status: AgentStatus::Starting,
            prompt,
            summary: None,
            created_at: now_iso(),
            started_at: Some(now_iso()),
            completed_at: None,
            model: None,
            branch: None,
            pid: None,
        };
        self.sessions.insert(id.clone(), session.clone());
        self.active_session_id = Some(id);
        session
    }

    pub fn get(&self, id: &str) -> AppResult<AgentSession> {
        self.sessions
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::SessionNotFound(id.to_string()))
    }

    pub fn get_mut(&mut self, id: &str) -> AppResult<&mut AgentSession> {
        self.sessions
            .get_mut(id)
            .ok_or_else(|| AppError::SessionNotFound(id.to_string()))
    }

    pub fn list(&self) -> Vec<AgentSession> {
        let mut sessions: Vec<_> = self.sessions.values().cloned().collect();
        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        sessions
    }

    pub fn active_session_id(&self) -> Option<&str> {
        self.active_session_id.as_deref()
    }

    pub fn has_active_running(&self) -> bool {
        self.active_session_id.as_ref().is_some_and(|id| {
            self.sessions.get(id).is_some_and(|s| {
                matches!(
                    s.status,
                    AgentStatus::Starting
                        | AgentStatus::Thinking
                        | AgentStatus::Reading
                        | AgentStatus::Editing
                        | AgentStatus::RunningCommand
                        | AgentStatus::WaitingApproval
                        | AgentStatus::Testing
                        | AgentStatus::Paused
                )
            })
        })
    }

    pub fn set_status(&mut self, id: &str, status: AgentStatus) -> AppResult<()> {
        let session = self.get_mut(id)?;
        session.status = status;
        if matches!(status, AgentStatus::Completed | AgentStatus::Error | AgentStatus::Cancelled) {
            session.completed_at = Some(now_iso());
            if self.active_session_id.as_deref() == Some(id) {
                self.active_session_id = None;
            }
        }
        Ok(())
    }

    pub fn set_pid(&mut self, id: &str, pid: u32) -> AppResult<()> {
        self.get_mut(id)?.pid = Some(pid);
        Ok(())
    }

    pub fn set_summary(&mut self, id: &str, summary: String) -> AppResult<()> {
        self.get_mut(id)?.summary = Some(summary);
        Ok(())
    }

    pub fn load_from_db(&mut self, sessions: Vec<AgentSession>) {
        for session in sessions {
            self.sessions.insert(session.id.clone(), session);
        }
    }
}

pub type SharedSessionManager = Arc<RwLock<SessionManager>>;
