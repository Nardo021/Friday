use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::intent_router::{QuickIntent, RouteResult};
use crate::core::AgentCore;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitQuickInputParams {
    pub text: String,
    pub session_id: Option<String>,
    pub project_id: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitQuickInputResult {
    pub route: RouteResult,
    pub executed: bool,
    pub message: Option<String>,
    pub session_id: Option<String>,
}

#[tauri::command]
pub async fn route_quick_input(
    core: State<'_, Arc<AgentCore>>,
    text: String,
    session_id: Option<String>,
    project_id: Option<String>,
    mode: Option<String>,
) -> Result<RouteResult, crate::errors::AppError> {
    core.route_quick_input(text, session_id, project_id, mode).await
}

#[tauri::command]
pub async fn submit_quick_input(
    app: tauri::AppHandle,
    core: State<'_, Arc<AgentCore>>,
    params: SubmitQuickInputParams,
) -> Result<SubmitQuickInputResult, crate::errors::AppError> {
    core.submit_quick_input(
        app,
        params.text,
        params.session_id,
        params.project_id,
        params.mode,
    )
    .await
}

#[tauri::command]
pub async fn execute_quick_intent(
    app: tauri::AppHandle,
    core: State<'_, Arc<AgentCore>>,
    intent: QuickIntent,
) -> Result<SubmitQuickInputResult, crate::errors::AppError> {
    core.execute_quick_intent(app, intent).await
}
