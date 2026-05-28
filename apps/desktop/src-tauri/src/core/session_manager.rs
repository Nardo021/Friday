use std::collections::HashMap;

use crate::core::event::{
    AgentSessionType, ControlLevel, FridaySession, FridaySessionStatus, SessionOwnership,
    SessionProcess, is_running_status, now_iso,
};
use crate::errors::{AppError, AppResult};

#[derive(Default)]
pub struct SessionManager {
    sessions: HashMap<String, FridaySession>,
    active_session_id: Option<String>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&mut self, session: FridaySession) -> FridaySession {
        let id = session.id.clone();
        self.sessions.insert(id.clone(), session.clone());
        self.active_session_id = Some(id);
        session
    }

    pub fn get(&self, id: &str) -> AppResult<FridaySession> {
        self.sessions
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::SessionNotFound(id.to_string()))
    }

    pub fn get_mut(&mut self, id: &str) -> AppResult<&mut FridaySession> {
        self.sessions
            .get_mut(id)
            .ok_or_else(|| AppError::SessionNotFound(id.to_string()))
    }

    pub fn list(&self) -> Vec<FridaySession> {
        let mut sessions: Vec<_> = self.sessions.values().cloned().collect();
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        sessions
    }

    pub fn set_active(&mut self, id: &str) -> AppResult<()> {
        if !self.sessions.contains_key(id) {
            return Err(AppError::SessionNotFound(id.to_string()));
        }
        self.active_session_id = Some(id.to_string());
        Ok(())
    }

    pub fn active_session_id(&self) -> Option<&str> {
        self.active_session_id.as_deref()
    }

    pub fn active_session(&self) -> Option<FridaySession> {
        self.active_session_id
            .as_ref()
            .and_then(|id| self.sessions.get(id).cloned())
    }

    pub fn update_status(&mut self, id: &str, status: FridaySessionStatus) -> AppResult<()> {
        let session = self.get_mut(id)?;
        session.status = status;
        session.updated_at = now_iso();
        if matches!(
            status,
            FridaySessionStatus::Done | FridaySessionStatus::Error | FridaySessionStatus::Stopped
        ) {
            session.completed_at = Some(now_iso());
            if self.active_session_id.as_deref() == Some(id) {
                self.active_session_id = None;
            }
        }
        Ok(())
    }

    pub fn has_pid(&self, pid: u32) -> bool {
        self.sessions.values().any(|s| {
            s.process
                .as_ref()
                .and_then(|p| p.pid)
                .is_some_and(|p| p == pid)
        })
    }

    pub fn find_by_pid(&self, pid: u32) -> Option<FridaySession> {
        self.sessions.values().find_map(|s| {
            s.process
                .as_ref()
                .and_then(|p| p.pid)
                .filter(|&p| p == pid)
                .map(|_| s.clone())
        })
    }

    pub fn set_process(&mut self, id: &str, process: SessionProcess) -> AppResult<()> {
        let session = self.get_mut(id)?;
        session.process = Some(process);
        session.updated_at = now_iso();
        Ok(())
    }

    pub fn set_summary(&mut self, id: &str, summary: String) -> AppResult<()> {
        let session = self.get_mut(id)?;
        session.summary = Some(summary);
        session.updated_at = now_iso();
        Ok(())
    }

    pub fn load_from_db(&mut self, sessions: Vec<FridaySession>) {
        for session in sessions {
            self.sessions.insert(session.id.clone(), session);
        }
    }

    pub fn count_running_owned_cli(&self) -> usize {
        self.sessions
            .values()
            .filter(|s| {
                s.session_type == AgentSessionType::FridayOwnedCli
                    && is_running_status(s.status)
            })
            .count()
    }

    pub fn can_start_owned_cli(&self) -> bool {
        self.count_running_owned_cli() == 0
    }

    pub fn can_attach_external(&self) -> bool {
        true
    }

    pub fn ensure_can_create(&self, session_type: AgentSessionType) -> AppResult<()> {
        match session_type {
            AgentSessionType::FridayOwnedCli => {
                if !self.can_start_owned_cli() {
                    return Err(AppError::SessionAlreadyRunning);
                }
            }
            AgentSessionType::ExternalCli => {
                if !self.can_attach_external() {
                    return Err(AppError::Other("Cannot attach external session".into()));
                }
            }
            AgentSessionType::CursorSdkLocal | AgentSessionType::CursorCloud => {}
        }
        Ok(())
    }

    pub fn ownership_for_type(session_type: AgentSessionType) -> SessionOwnership {
        match session_type {
            AgentSessionType::ExternalCli => SessionOwnership::External,
            _ => SessionOwnership::Friday,
        }
    }

    pub fn default_control_level(session_type: AgentSessionType) -> ControlLevel {
        match session_type {
            AgentSessionType::ExternalCli => ControlLevel::Observe,
            AgentSessionType::FridayOwnedCli | AgentSessionType::CursorCloud => ControlLevel::Full,
            _ => ControlLevel::None,
        }
    }
}
