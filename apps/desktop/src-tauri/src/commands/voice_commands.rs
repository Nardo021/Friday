use crate::voice::{SttAdapter, TranscriptionResult};

#[tauri::command]
pub async fn transcribe_audio(
    audio_base64: String,
    language: Option<String>,
) -> Result<TranscriptionResult, crate::errors::AppError> {
    SttAdapter::transcribe(&audio_base64, language.as_deref()).await
}

#[tauri::command]
pub fn save_stt_api_key(api_key: String) -> Result<(), crate::errors::AppError> {
    crate::security::SecretStore::save_stt_api_key(&api_key)
}

#[tauri::command]
pub fn clear_stt_api_key() -> Result<(), crate::errors::AppError> {
    crate::security::SecretStore::clear_stt_api_key()
}
