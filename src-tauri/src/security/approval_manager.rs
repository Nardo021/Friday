use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{Mutex, oneshot};

use crate::core::event::{AgentEvent, RiskLevel, now_iso};
use crate::errors::{AppError, AppResult};

struct PendingApproval {
    session_id: String,
    command: String,
    risk: RiskLevel,
    tx: oneshot::Sender<bool>,
}

#[derive(Default)]
pub struct ApprovalManager {
    pending: HashMap<String, PendingApproval>,
}

impl ApprovalManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn request_approval(
        &mut self,
        session_id: &str,
        command: &str,
        risk: RiskLevel,
    ) -> (String, AgentEvent, oneshot::Receiver<bool>) {
        let approval_id = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = oneshot::channel();

        self.pending.insert(
            approval_id.clone(),
            PendingApproval {
                session_id: session_id.to_string(),
                command: command.to_string(),
                risk,
                tx,
            },
        );

        let event = AgentEvent::ApprovalRequired {
            session_id: session_id.to_string(),
            approval_id: approval_id.clone(),
            title: "Command approval required".into(),
            description: Some("Friday paused until you approve or reject.".into()),
            command: Some(command.to_string()),
            risk,
            timestamp: now_iso(),
        };

        (approval_id, event, rx)
    }

    pub fn approve(&mut self, approval_id: &str) -> AppResult<()> {
        let pending = self
            .pending
            .remove(approval_id)
            .ok_or_else(|| AppError::ApprovalNotFound(approval_id.to_string()))?;
        let _ = pending.tx.send(true);
        Ok(())
    }

    pub fn reject(&mut self, approval_id: &str) -> AppResult<()> {
        let pending = self
            .pending
            .remove(approval_id)
            .ok_or_else(|| AppError::ApprovalNotFound(approval_id.to_string()))?;
        let _ = pending.tx.send(false);
        Ok(())
    }
}

pub type SharedApprovalManager = Arc<Mutex<ApprovalManager>>;

pub fn create_approval_manager() -> SharedApprovalManager {
    Arc::new(Mutex::new(ApprovalManager::new()))
}
