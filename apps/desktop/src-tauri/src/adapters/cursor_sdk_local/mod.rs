use async_trait::async_trait;

use crate::adapters::r#trait::{
    AdapterContext, AgentAdapter, AttachSessionInput, CreateSessionInput,
};
use crate::adapters::registry::ADAPTER_CURSOR_SDK_LOCAL;
use crate::core::event::FridaySession;
use crate::errors::{AppError, AppResult};

pub struct CursorSdkLocalAdapter;

#[async_trait]
impl AgentAdapter for CursorSdkLocalAdapter {
    fn id(&self) -> &str {
        ADAPTER_CURSOR_SDK_LOCAL
    }

    async fn create_session(
        &self,
        _input: CreateSessionInput,
        _ctx: &AdapterContext,
    ) -> AppResult<FridaySession> {
        Err(AppError::Other("cursor-sdk-local is not implemented".into()))
    }

    async fn attach_session(
        &self,
        _input: AttachSessionInput,
        _ctx: &AdapterContext,
    ) -> AppResult<FridaySession> {
        Err(AppError::Other("cursor-sdk-local is not implemented".into()))
    }

    async fn stop_session(&self, _session_id: &str, _ctx: &AdapterContext) -> AppResult<()> {
        Err(AppError::Other("cursor-sdk-local is not implemented".into()))
    }

    async fn send_message(
        &self,
        _session_id: &str,
        _message: &str,
        _ctx: &AdapterContext,
    ) -> AppResult<()> {
        Err(AppError::Other("cursor-sdk-local is not implemented".into()))
    }
}
