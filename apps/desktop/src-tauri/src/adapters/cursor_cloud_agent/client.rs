use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppResult};
use crate::security::SecretStore;

const BASE_URL: &str = "https://api.cursor.com";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentRequest {
    pub prompt: PromptBody,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelSelection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repos: Option<Vec<RepoConfig>>,
    /// Cursor API field name is `autoCreatePR` (not camelCase `autoCreatePr`).
    #[serde(
        rename = "autoCreatePR",
        skip_serializing_if = "Option::is_none"
    )]
    pub auto_create_pr: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptBody {
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelSelection {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoConfig {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starting_ref: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentResponse {
    pub agent: CloudAgent,
    pub run: CloudRun,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRunResponse {
    pub run: CloudRun,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudAgent {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudRun {
    pub id: String,
    pub agent_id: String,
    pub status: String,
    #[serde(default)]
    pub result: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub git: Option<RunGit>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RunGit {
    #[serde(default)]
    pub branches: Vec<GitBranch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitBranch {
    #[serde(default)]
    pub repo_url: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub pr_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactItem {
    pub path: String,
    #[serde(default)]
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArtifactsListResponse {
    #[serde(default)]
    pub items: Vec<ArtifactItem>,
}

pub struct CursorCloudClient {
    http: reqwest::Client,
    api_key: String,
}

fn basic_auth_header(api_key: &str) -> String {
    let encoded = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        format!("{}:", api_key.trim()),
    );
    format!("Basic {encoded}")
}

impl CursorCloudClient {
    pub fn new() -> AppResult<Self> {
        let api_key = SecretStore::get_cursor_api_key()?
            .ok_or_else(|| AppError::Other("Cursor API key not configured".into()))?;
        Ok(Self {
            http: reqwest::Client::new(),
            api_key,
        })
    }

    pub fn with_api_key(api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
        }
    }

    /// Validates a Cursor dashboard API key against the Cloud Agents API.
    pub async fn verify_api_key(api_key: &str) -> AppResult<()> {
        let trimmed = api_key.trim();
        if trimmed.is_empty() {
            return Err(AppError::Other("API key cannot be empty".into()));
        }
        let client = reqwest::Client::new();
        let auth = basic_auth_header(trimmed);
        for path in ["/v1/me", "/v0/me"] {
            let resp = client
                .get(format!("{BASE_URL}{path}"))
                .header(AUTHORIZATION, auth.clone())
                .send()
                .await
                .map_err(|e| AppError::Other(format!("Could not reach Cursor API: {e}")))?;

            if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
                return Err(AppError::Other(
                    "Invalid Cursor API key (unauthorized). Use a key from \
                     cursor.com/dashboard → Integrations → API Keys."
                        .into(),
                ));
            }
            if resp.status().is_success() {
                return Ok(());
            }
        }
        Err(AppError::Other(
            "Cursor API key could not be verified. Check your network and try again.".into(),
        ))
    }

    /// Lists GitHub repository URLs linked to the Cursor account (for Cloud agents).
    pub async fn list_repository_urls(&self) -> AppResult<Vec<String>> {
        for path in ["/v1/repositories", "/v0/repositories"] {
            let resp = self
                .http
                .get(format!("{BASE_URL}{path}"))
                .header(AUTHORIZATION, self.auth_header())
                .send()
                .await
                .map_err(|e| AppError::Other(format!("list repositories failed: {e}")))?;

            if !resp.status().is_success() {
                continue;
            }

            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| AppError::Other(format!("list repositories parse failed: {e}")))?;

            let urls = extract_repo_urls(&json);
            if !urls.is_empty() {
                return Ok(urls);
            }
        }
        Ok(vec![])
    }

    fn auth_header(&self) -> String {
        basic_auth_header(&self.api_key)
    }

    pub async fn create_agent(&self, req: CreateAgentRequest) -> AppResult<CreateAgentResponse> {
        let resp = self
            .http
            .post(format!("{BASE_URL}/v1/agents"))
            .header(AUTHORIZATION, self.auth_header())
            .header(CONTENT_TYPE, "application/json")
            .json(&req)
            .send()
            .await
            .map_err(|e| AppError::Other(format!("cloud API request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Other(format!(
                "cloud create agent failed ({status}): {body}"
            )));
        }

        resp.json()
            .await
            .map_err(|e| AppError::Other(format!("cloud create agent parse failed: {e}")))
    }

    pub async fn create_run(&self, agent_id: &str, prompt: &str) -> AppResult<CreateRunResponse> {
        let body = serde_json::json!({
            "prompt": { "text": prompt }
        });

        let resp = self
            .http
            .post(format!("{BASE_URL}/v1/agents/{agent_id}/runs"))
            .header(AUTHORIZATION, self.auth_header())
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Other(format!("cloud follow-up failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Other(format!(
                "cloud follow-up failed ({status}): {body}"
            )));
        }

        resp.json()
            .await
            .map_err(|e| AppError::Other(format!("cloud follow-up parse failed: {e}")))
    }

    pub async fn get_run(&self, agent_id: &str, run_id: &str) -> AppResult<CloudRun> {
        let resp = self
            .http
            .get(format!("{BASE_URL}/v1/agents/{agent_id}/runs/{run_id}"))
            .header(AUTHORIZATION, self.auth_header())
            .send()
            .await
            .map_err(|e| AppError::Other(format!("cloud get run failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Other(format!(
                "cloud get run failed ({status}): {body}"
            )));
        }

        resp.json()
            .await
            .map_err(|e| AppError::Other(format!("cloud get run parse failed: {e}")))
    }

    pub async fn cancel_run(&self, agent_id: &str, run_id: &str) -> AppResult<()> {
        let resp = self
            .http
            .post(format!(
                "{BASE_URL}/v1/agents/{agent_id}/runs/{run_id}/cancel"
            ))
            .header(AUTHORIZATION, self.auth_header())
            .send()
            .await
            .map_err(|e| AppError::Other(format!("cloud cancel failed: {e}")))?;

        if resp.status().is_success() || resp.status().as_u16() == 409 {
            return Ok(());
        }

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Err(AppError::Other(format!(
            "cloud cancel failed ({status}): {body}"
        )))
    }

    pub async fn stream_run(
        &self,
        agent_id: &str,
        run_id: &str,
        last_event_id: Option<&str>,
    ) -> AppResult<reqwest::Response> {
        let mut req = self
            .http
            .get(format!(
                "{BASE_URL}/v1/agents/{agent_id}/runs/{run_id}/stream"
            ))
            .header(AUTHORIZATION, self.auth_header())
            .header(ACCEPT, "text/event-stream");

        if let Some(id) = last_event_id {
            req = req.header("Last-Event-ID", id);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AppError::Other(format!("cloud stream failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Other(format!(
                "cloud stream failed ({status}): {body}"
            )));
        }

        Ok(resp)
    }

    pub async fn list_artifacts(&self, agent_id: &str) -> AppResult<Vec<ArtifactItem>> {
        let resp = self
            .http
            .get(format!("{BASE_URL}/v1/agents/{agent_id}/artifacts"))
            .header(AUTHORIZATION, self.auth_header())
            .send()
            .await
            .map_err(|e| AppError::Other(format!("cloud list artifacts failed: {e}")))?;

        if !resp.status().is_success() {
            return Ok(vec![]);
        }

        let list: ArtifactsListResponse = resp
            .json()
            .await
            .unwrap_or(ArtifactsListResponse { items: vec![] });
        Ok(list.items)
    }
}

fn extract_repo_urls(json: &serde_json::Value) -> Vec<String> {
    let mut urls = Vec::new();
    match json {
        serde_json::Value::Array(items) => {
            for item in items {
                push_repo_url(item, &mut urls);
            }
        }
        serde_json::Value::Object(map) => {
            for key in ["repositories", "items", "data"] {
                if let Some(serde_json::Value::Array(items)) = map.get(key) {
                    for item in items {
                        push_repo_url(item, &mut urls);
                    }
                }
            }
        }
        _ => {}
    }
    urls.sort();
    urls.dedup();
    urls
}

fn push_repo_url(value: &serde_json::Value, urls: &mut Vec<String>) {
    match value {
        serde_json::Value::String(s) if s.starts_with("http") => urls.push(s.clone()),
        serde_json::Value::Object(map) => {
            for key in ["url", "htmlUrl", "html_url", "cloneUrl", "clone_url"] {
                if let Some(serde_json::Value::String(s)) = map.get(key) {
                    if s.starts_with("http") {
                        urls.push(s.clone());
                        return;
                    }
                }
            }
        }
        _ => {}
    }
}
