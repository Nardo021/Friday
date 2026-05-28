pub mod cursor_event_mapper;
pub mod cursor_parser;
pub mod launcher;
pub mod stream;

use async_trait::async_trait;

use crate::adapters::cursor_cli_local::stream::{emit_stderr_line, feed_parser_lines, flush_parser_buffer};
use crate::adapters::cursor_cli_local::launcher::build_command;
use crate::adapters::r#trait::{
    AdapterContext, AgentAdapter, AttachSessionInput, CreateSessionInput,
};
use crate::adapters::registry::ADAPTER_CURSOR_CLI_LOCAL;
use crate::core::event::{
    AgentEvent, AgentSessionType, FridaySession, FridaySessionStatus, MessageRole,
    SessionOwnership, SessionProcess,
};
use crate::core::session_manager::SessionManager;
use crate::core::event::now_iso;
use crate::errors::{AppError, AppResult};
use crate::process::SpawnConfig;

pub struct CursorCliLocalAdapter;

#[async_trait]
impl AgentAdapter for CursorCliLocalAdapter {
    fn id(&self) -> &str {
        ADAPTER_CURSOR_CLI_LOCAL
    }

    async fn create_session(
        &self,
        input: CreateSessionInput,
        ctx: &AdapterContext,
    ) -> AppResult<FridaySession> {
        let built = build_command(&input.prompt, &input.cwd, &ctx.cursor_settings)?;

        (ctx.event_handler)(AgentEvent::AgentStatus {
            session_id: input.session_id.clone(),
            status: FridaySessionStatus::Starting,
            message: Some("Starting Cursor CLI".into()),
            timestamp: now_iso(),
        });

        (ctx.event_handler)(AgentEvent::AgentMessage {
            session_id: input.session_id.clone(),
            role: MessageRole::User,
            content: input.prompt.clone(),
            timestamp: now_iso(),
        });

        let session_id = input.session_id.clone();
        let handler = ctx.event_handler.clone();

        let (pid, pty_id) = if ctx.cursor_settings.use_pty {
            let (pty_id, mut rx) = ctx.pty_manager.create_pty(
                &session_id,
                ctx.cursor_settings.terminal_cols,
                ctx.cursor_settings.terminal_rows,
                &built.cwd,
                &built.executable,
                &built.args,
            )?;

            let sid = session_id.clone();
            let h = handler.clone();
            tauri::async_runtime::spawn(async move {
                let mut line_buf = String::new();
                while let Some(chunk) = rx.recv().await {
                    feed_parser_lines(&sid, &mut line_buf, &chunk, &h);
                }
                flush_parser_buffer(&sid, &mut line_buf, &h);
            });

            (0u32, Some(pty_id))
        } else {
            let (pid, mut rx) = ctx
                .supervisor
                .spawn_with_output(SpawnConfig {
                    session_id: session_id.clone(),
                    adapter_id: ADAPTER_CURSOR_CLI_LOCAL.into(),
                    executable: built.executable,
                    args: built.args,
                    cwd: built.cwd,
                    env: built.env,
                })
                .await?;

            let sid = session_id.clone();
            let h = handler.clone();
            tauri::async_runtime::spawn(async move {
                let mut line_buf = String::new();
                while let Some((line, is_stderr)) = rx.recv().await {
                    if is_stderr {
                        emit_stderr_line(&sid, &line, &h);
                    }
                    feed_parser_lines(&sid, &mut line_buf, line.as_bytes(), &h);
                }
                flush_parser_buffer(&sid, &mut line_buf, &h);
            });

            let sup = ctx.supervisor.clone();
            let sid = session_id.clone();
            let h = handler.clone();
            tauri::async_runtime::spawn(async move {
                sup.wait_session(sid, h).await;
            });

            (pid, None)
        };

        let process = SessionProcess {
            pid: if pid > 0 { Some(pid) } else { None },
            pty_id,
            cwd: Some(input.cwd.clone()),
        };

        Ok(FridaySession {
            id: input.session_id,
            title: truncate_title(&input.prompt),
            session_type: AgentSessionType::FridayOwnedCli,
            ownership: SessionOwnership::Friday,
            adapter_id: ADAPTER_CURSOR_CLI_LOCAL.into(),
            status: FridaySessionStatus::Starting,
            control_level: SessionManager::default_control_level(AgentSessionType::FridayOwnedCli),
            project_id: Some(input.project_id),
            prompt: Some(input.prompt),
            summary: None,
            repo: None,
            process: Some(process),
            cloud: None,
            created_at: now_iso(),
            started_at: Some(now_iso()),
            updated_at: now_iso(),
            completed_at: None,
        })
    }

    async fn attach_session(
        &self,
        _input: AttachSessionInput,
        _ctx: &AdapterContext,
    ) -> AppResult<FridaySession> {
        Err(AppError::Other(
            "cursor-cli-local does not support attach".into(),
        ))
    }

    async fn stop_session(&self, session_id: &str, ctx: &AdapterContext) -> AppResult<()> {
        if ctx.cursor_settings.use_pty {
            ctx.pty_manager.close_by_session(session_id, false)?;
        } else {
            ctx.supervisor.stop_session(session_id, 3).await?;
        }
        Ok(())
    }

    async fn send_message(
        &self,
        session_id: &str,
        message: &str,
        ctx: &AdapterContext,
    ) -> AppResult<()> {
        if ctx.cursor_settings.use_pty {
            if let Some(pty_id) = ctx.pty_manager.pty_id_for_session(session_id) {
                let data = format!("{message}\n");
                ctx.pty_manager.write(&pty_id, data.as_bytes())?;
                return Ok(());
            }
        }
        Err(AppError::Other(
            "follow-up requires active PTY session".into(),
        ))
    }
}

fn truncate_title(prompt: &str) -> String {
    let trimmed = prompt.trim();
    if trimmed.len() <= 60 {
        trimmed.to_string()
    } else {
        format!("{}...", &trimmed[..57])
    }
}
