use std::sync::{Arc, Weak};

use tauri::{AppHandle, Manager};
use tokio::sync::mpsc;

use crate::core::event::AgentEvent;
use crate::core::AgentCore;
use crate::storage::Database;

const EVENT_QUEUE_CAPACITY: usize = 2048;

/// Single async worker drains agent events — avoids spawning a task per CLI line.
pub fn spawn_event_worker(
    self_weak: Weak<AgentCore>,
    app: AppHandle,
    db: Arc<Database>,
) -> mpsc::Sender<(AppHandle, AgentEvent)> {
    let (tx, mut rx) = mpsc::channel(EVENT_QUEUE_CAPACITY);

    tauri::async_runtime::spawn(async move {
        while let Some((app, event)) = rx.recv().await {
            if let Some(core) = self_weak.upgrade() {
                let _ = core.dispatch_event(app, event).await;
            } else {
                let bridge = app
                    .try_state::<crate::bridge::BridgeBroadcast>()
                    .map(|b| b.inner().clone());
                let _ =
                    crate::core::event_pipeline::handle_event(app, db.clone(), event, bridge).await;
            }
        }
    });

    tx
}

pub fn make_event_handler(
    tx: mpsc::Sender<(AppHandle, AgentEvent)>,
    app: AppHandle,
) -> crate::adapters::r#trait::EventHandler {
    Arc::new(move |event: AgentEvent| {
        let app = app.clone();
        match tx.try_send((app, event)) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full((app, event))) => {
                let tx = tx.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = tx.send((app, event)).await;
                });
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {}
        }
    })
}
