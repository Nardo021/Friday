use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::bridge::BridgeBroadcast;
use crate::core::event_pipeline::handle_event;
use crate::core::event::{AgentEvent, FridaySessionStatus};
use crate::core::AgentCore;
use crate::errors::AppResult;
use crate::storage::{Database, SessionsRepo};

pub async fn dispatch_event(
    core: &AgentCore,
    app: AppHandle,
    db: Arc<Database>,
    event: AgentEvent,
) -> AppResult<()> {
    let session_id = event.session_id().to_string();

    // Sync in-memory SessionManager so owned-CLI slot frees on Done/Error/Stopped.
    sync_session_manager(core, &event).await;

    if let AgentEvent::CommandStarted { ref command, .. } = event {
        let settings = core.settings_snapshot();
        if crate::security::CommandPolicy::requires_approval(command, &settings.security) {
            let risk = crate::security::risk_classifier::classify_command_risk(command);
            let (_approval_id, approval_event, _rx) = core
                .approval_manager
                .lock()
                .await
                .request_approval(&session_id, command, risk)
                .await;

            // Non-blocking: Cursor CLI keeps running; Reject stops the session.
            // Blocking the event worker previously stalled the UI while the agent continued.
            handle_event(app.clone(), db.clone(), approval_event, None).await?;

            if !session_id.is_empty() {
                let _ = core
                    .session_manager
                    .lock()
                    .await
                    .update_status(&session_id, FridaySessionStatus::WaitingPermission);
                let _ = SessionsRepo::new(&db).update_status(
                    &session_id,
                    FridaySessionStatus::WaitingPermission,
                    None,
                );
            }
        }
    }

    if let AgentEvent::CommandCompleted { .. } = &event {
        if !session_id.is_empty() {
            core.try_flush_instruction_queue(&app, &session_id).await?;
        }
    }

    let bridge = app.try_state::<BridgeBroadcast>().map(|b| b.inner().clone());
    handle_event(app, db, event, bridge).await
}

async fn sync_session_manager(core: &AgentCore, event: &AgentEvent) {
    let session_id = event.session_id();
    if session_id.is_empty() {
        return;
    }

    match event {
        AgentEvent::AgentStatus { status, .. } => {
            let mut mgr = core.session_manager.lock().await;
            if let Ok(existing) = mgr.get(session_id) {
                // Don't resurrect or downgrade terminal sessions from late process-exit events.
                if matches!(
                    existing.status,
                    FridaySessionStatus::Done
                        | FridaySessionStatus::Error
                        | FridaySessionStatus::Stopped
                ) {
                    return;
                }
                let _ = mgr.update_status(session_id, *status);
            }
        }
        AgentEvent::SessionCompleted { summary, .. } => {
            let mut mgr = core.session_manager.lock().await;
            if mgr.get(session_id).is_ok() {
                if let Some(summary) = summary {
                    let _ = mgr.set_summary(session_id, summary.clone());
                }
                let _ = mgr.update_status(session_id, FridaySessionStatus::Done);
            }
        }
        AgentEvent::SessionError { .. } => {
            let mut mgr = core.session_manager.lock().await;
            if mgr.get(session_id).is_ok() {
                let _ = mgr.update_status(session_id, FridaySessionStatus::Error);
            }
        }
        AgentEvent::SessionStarted { .. } => {
            let mut mgr = core.session_manager.lock().await;
            if let Ok(existing) = mgr.get(session_id) {
                if matches!(
                    existing.status,
                    FridaySessionStatus::Done
                        | FridaySessionStatus::Error
                        | FridaySessionStatus::Stopped
                ) {
                    return;
                }
                let _ = mgr.update_status(session_id, FridaySessionStatus::Thinking);
            }
        }
        _ => {}
    }
}
