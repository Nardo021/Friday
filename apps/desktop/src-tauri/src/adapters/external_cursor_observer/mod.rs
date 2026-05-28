use async_trait::async_trait;

use crate::adapters::r#trait::{
    AdapterContext, AgentAdapter, AttachSessionInput, CreateSessionInput,
};
use crate::adapters::registry::ADAPTER_EXTERNAL_CURSOR_OBSERVER;
use crate::core::event::{
    AgentEvent, AgentSessionType, DiscoverySource, FridaySession, FridaySessionStatus,
    SessionOwnership, SessionProcess,
};
use crate::core::event::now_iso;
use crate::core::session_manager::SessionManager;
use crate::errors::{AppError, AppResult};

pub struct ExternalCursorObserverAdapter;

#[async_trait]
impl AgentAdapter for ExternalCursorObserverAdapter {
    fn id(&self) -> &str {
        ADAPTER_EXTERNAL_CURSOR_OBSERVER
    }

    async fn create_session(
        &self,
        _input: CreateSessionInput,
        _ctx: &AdapterContext,
    ) -> AppResult<FridaySession> {
        Err(AppError::Other(
            "external-cursor-observer only supports attach".into(),
        ))
    }

    async fn attach_session(
        &self,
        input: AttachSessionInput,
        ctx: &AdapterContext,
    ) -> AppResult<FridaySession> {
        (ctx.event_handler)(AgentEvent::SessionDiscovered {
            session_id: input.session_id.clone(),
            source: DiscoverySource::ProcessScan,
            timestamp: now_iso(),
        });

        let title = input
            .exe_name
            .clone()
            .unwrap_or_else(|| format!("Cursor Agent ({})", input.pid));

        Ok(FridaySession {
            id: input.session_id,
            title,
            session_type: AgentSessionType::ExternalCli,
            ownership: SessionOwnership::External,
            adapter_id: ADAPTER_EXTERNAL_CURSOR_OBSERVER.into(),
            status: FridaySessionStatus::Discovered,
            control_level: SessionManager::default_control_level(AgentSessionType::ExternalCli),
            project_id: None,
            prompt: None,
            summary: None,
            repo: None,
            process: Some(SessionProcess {
                pid: Some(input.pid),
                pty_id: None,
                cwd: input.cwd,
            }),
            cloud: None,
            created_at: now_iso(),
            started_at: None,
            updated_at: now_iso(),
            completed_at: None,
        })
    }

    async fn stop_session(&self, _session_id: &str, _ctx: &AdapterContext) -> AppResult<()> {
        Ok(())
    }

    async fn send_message(
        &self,
        _session_id: &str,
        _message: &str,
        _ctx: &AdapterContext,
    ) -> AppResult<()> {
        Err(AppError::Other(
            "external observer is read-only".into(),
        ))
    }
}
