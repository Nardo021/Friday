use std::sync::Arc;

use tauri::State;

use crate::core::AgentCore;
use crate::security::SecretStore;
use crate::voice::{SttAdapter, TranscriptionResult};

#[tauri::command]
pub async fn transcribe_audio(
    audio_base64: String,
    language: Option<String>,
) -> Result<TranscriptionResult, crate::errors::AppError> {
    SttAdapter::transcribe(&audio_base64, language.as_deref()).await
}

#[tauri::command]
pub fn save_stt_api_key(
    core: State<'_, Arc<AgentCore>>,
    api_key: String,
) -> Result<(), crate::errors::AppError> {
    SecretStore::save_stt_api_key(&api_key)?;
    core.reload_settings_cache()?;
    Ok(())
}

#[tauri::command]
pub fn clear_stt_api_key(
    core: State<'_, Arc<AgentCore>>,
) -> Result<(), crate::errors::AppError> {
    SecretStore::clear_stt_api_key()?;
    core.reload_settings_cache()?;
    Ok(())
}
