use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppResult};
use crate::security::SecretStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionResult {
    pub transcript: String,
    pub duration_ms: u64,
}

pub struct SttAdapter;

impl SttAdapter {
    pub async fn transcribe(audio_base64: &str, language: Option<&str>) -> AppResult<TranscriptionResult> {
        let api_key = SecretStore::get_stt_api_key()?
            .ok_or_else(|| AppError::Other("STT API key not configured".into()))?;

        let audio_bytes = base64::engine::general_purpose::STANDARD
            .decode(audio_base64.trim())
            .map_err(|e| AppError::Other(format!("invalid audio data: {e}")))?;

        let part = reqwest::multipart::Part::bytes(audio_bytes)
            .file_name("audio.webm")
            .mime_str("audio/webm")
            .map_err(|e| AppError::Other(e.to_string()))?;

        let mut form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("model", "whisper-1");

        if let Some(lang) = language.filter(|l| !l.is_empty()) {
            form = form.text("language", lang.to_string());
        }

        let client = reqwest::Client::new();
        let resp = client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .bearer_auth(api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::Other(format!("STT request failed: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Other(format!("STT HTTP error: {body}")));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::Other(format!("STT parse failed: {e}")))?;

        let transcript = json["text"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_string();

        Ok(TranscriptionResult {
            transcript,
            duration_ms: 0,
        })
    }
}
