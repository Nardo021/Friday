use std::sync::Arc;

use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::bridge::BridgeBroadcast;
use crate::core::event::AgentEvent;
use crate::core::event_bus::emit_agent_event;
use crate::errors::AppResult;
use crate::security::SecretRedactor;
use crate::storage::{logs_dir, persist_agent_event, should_skip_event, Database};

pub async fn handle_event(
    app: AppHandle,
    db: Arc<Database>,
    event: AgentEvent,
    bridge: Option<BridgeBroadcast>,
) -> AppResult<()> {
    if should_skip_event(&event) {
        return Ok(());
    }

    let session_id = event.session_id().to_string();
    let payload_json = serde_json::to_string(&event)?;
    let redacted_json = SecretRedactor::redact(&payload_json);

    persist_agent_event(&db, &event, &redacted_json)?;

    if !session_id.is_empty() {
        let sid = session_id.clone();
        let line = redacted_json.clone();
        tauri::async_runtime::spawn_blocking(move || append_session_log_line(&sid, &line));
    }

    if let AgentEvent::ApprovalRequired { command, .. } = &event {
        let body = command
            .as_deref()
            .unwrap_or("Command requires your approval");
        let _ = app
            .notification()
            .builder()
            .title("Friday — Approval needed")
            .body(body)
            .show();
    }

    emit_agent_event(&app, &event)?;

    let broadcast = bridge.or_else(|| {
        app.try_state::<BridgeBroadcast>()
            .map(|b| b.inner().clone())
    });
    if let Some(broadcast) = broadcast {
        broadcast.publish(redacted_json);
    }

    Ok(())
}

fn append_session_log_line(session_id: &str, line: &str) -> Result<(), crate::errors::AppError> {
    let dir = logs_dir()?;
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{session_id}.log"));
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{line}")?;
    Ok(())
}
