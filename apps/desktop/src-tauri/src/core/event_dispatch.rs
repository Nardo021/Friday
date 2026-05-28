use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::bridge::BridgeBroadcast;
use crate::core::event_pipeline::handle_event;
use crate::core::event::{AgentEvent, FridaySessionStatus, now_iso};
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

    if let AgentEvent::CommandStarted { ref command, .. } = event {
        let settings = core.settings_snapshot();
        if crate::security::CommandPolicy::requires_approval(command, &settings.security) {
            let risk = crate::security::risk_classifier::classify_command_risk(command);
            let (_approval_id, approval_event, rx) = core
                .approval_manager
                .lock()
                .await
                .request_approval(&session_id, command, risk)
                .await;

            handle_event(app.clone(), db.clone(), approval_event, None).await?;

            if !session_id.is_empty() {
                core.session_manager
                    .lock()
                    .await
                    .update_status(&session_id, FridaySessionStatus::WaitingPermission)?;
                SessionsRepo::new(&db).update_status(
                    &session_id,
                    FridaySessionStatus::WaitingPermission,
                    None,
                )?;
            }

            let approved = rx.await.unwrap_or(false);
            if !approved {
                let reject_event = AgentEvent::AgentStatus {
                    session_id: session_id.clone(),
                    status: FridaySessionStatus::Stopped,
                    message: Some("Command rejected by user".into()),
                    timestamp: now_iso(),
                };
                handle_event(app, db, reject_event, None).await?;
                return Ok(());
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
