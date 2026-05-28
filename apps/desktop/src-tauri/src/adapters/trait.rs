use std::sync::Arc;

use async_trait::async_trait;

use crate::core::event::{AgentSessionType, FridaySession};
use crate::errors::AppResult;
use crate::storage::settings_repo::CloudSettings;
use crate::storage::Database;

#[derive(Debug, Clone)]
pub struct CreateSessionInput {
    pub session_id: String,
    pub session_type: AgentSessionType,
    pub project_id: String,
    pub prompt: String,
    pub cwd: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AttachSessionInput {
    pub session_id: String,
    pub pid: u32,
    pub cwd: Option<String>,
    pub exe_name: Option<String>,
}

pub type EventHandler = Arc<dyn Fn(crate::core::event::AgentEvent) + Send + Sync>;

#[derive(Clone)]
pub struct AdapterContext {
    pub supervisor: Arc<crate::process::ProcessSupervisor>,
    pub pty_manager: Arc<crate::pty::PtyManager>,
    pub event_handler: EventHandler,
    pub cursor_settings: crate::storage::settings_repo::CursorSettings,
    pub cloud_settings: CloudSettings,
    pub db: Arc<Database>,
}

#[async_trait]
pub trait AgentAdapter: Send + Sync {
    fn id(&self) -> &str;

    async fn create_session(
        &self,
        input: CreateSessionInput,
        ctx: &AdapterContext,
    ) -> AppResult<FridaySession>;

    async fn attach_session(
        &self,
        input: AttachSessionInput,
        ctx: &AdapterContext,
    ) -> AppResult<FridaySession>;

    async fn stop_session(&self, session_id: &str, ctx: &AdapterContext) -> AppResult<()>;

    async fn send_message(
        &self,
        session_id: &str,
        message: &str,
        ctx: &AdapterContext,
    ) -> AppResult<()>;
}
