use std::sync::Arc;

use tauri::AppHandle;

use crate::bridge::auth::SharedAuthToken;
use crate::bridge::broadcast::BridgeBroadcast;
use crate::core::AgentCore;

#[derive(Clone)]
pub struct BridgeState {
    pub core: Arc<AgentCore>,
    pub app: AppHandle,
    pub auth_token: SharedAuthToken,
    pub broadcast: BridgeBroadcast,
}
