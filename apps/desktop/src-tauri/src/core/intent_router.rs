use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::core::event::{FridaySession, FridaySessionStatus, is_running_status};
use crate::errors::{AppError, AppResult};
use crate::security::SecretStore;

const RULE_CONFIDENCE: f32 = 0.92;
const LLM_THRESHOLD: f32 = 0.55;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlAction {
    Stop,
    Pause,
    Resume,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum QuickIntent {
    FollowUp {
        session_id: String,
        text: String,
    },
    NewTask {
        project_id: String,
        mode: String,
        prompt: String,
    },
    QueryStatus,
    Control {
        action: ControlAction,
        session_id: Option<String>,
    },
    SaveIdea {
        title: String,
        body: String,
        project_id: Option<String>,
        session_id: Option<String>,
    },
    OpenChat,
    Clarify {
        message: String,
        options: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteResult {
    pub intent: QuickIntent,
    pub confidence: f32,
    pub source: String,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RouteContext {
    pub text: String,
    pub session_id: Option<String>,
    pub project_id: Option<String>,
    pub mode: String,
    pub active_session: Option<FridaySession>,
    pub has_running_session: bool,
}

pub struct IntentRouter;

impl IntentRouter {
    pub async fn route(ctx: RouteContext) -> AppResult<RouteResult> {
        let trimmed = ctx.text.trim();
        if trimmed.is_empty() {
            return Err(AppError::Other("Input cannot be empty".into()));
        }

        if let Some(result) = Self::match_rules(&ctx) {
            return Ok(result);
        }

        if let Ok(result) = Self::llm_classify(&ctx).await {
            if result.confidence >= LLM_THRESHOLD {
                return Ok(result);
            }
        }

        Ok(Self::clarify_fallback(&ctx))
    }

    fn match_rules(ctx: &RouteContext) -> Option<RouteResult> {
        let lower = ctx.text.to_lowercase();
        let trimmed = ctx.text.trim();

        if Self::is_stop_intent(&lower) {
            return Some(RouteResult {
                intent: QuickIntent::Control {
                    action: ControlAction::Stop,
                    session_id: ctx.session_id.clone(),
                },
                confidence: RULE_CONFIDENCE,
                source: "rules".into(),
                status_message: None,
            });
        }

        if Self::is_pause_intent(&lower) {
            return Some(RouteResult {
                intent: QuickIntent::Control {
                    action: ControlAction::Pause,
                    session_id: ctx.session_id.clone(),
                },
                confidence: RULE_CONFIDENCE,
                source: "rules".into(),
                status_message: None,
            });
        }

        if Self::is_status_query(&lower) {
            return Some(RouteResult {
                intent: QuickIntent::QueryStatus,
                confidence: RULE_CONFIDENCE,
                source: "rules".into(),
                status_message: None,
            });
        }

        if Self::is_open_chat(&lower) {
            return Some(RouteResult {
                intent: QuickIntent::OpenChat,
                confidence: RULE_CONFIDENCE,
                source: "rules".into(),
                status_message: None,
            });
        }

        if let Some(idea) = Self::parse_save_idea(trimmed) {
            return Some(RouteResult {
                intent: QuickIntent::SaveIdea {
                    title: idea.0,
                    body: idea.1,
                    project_id: ctx.project_id.clone(),
                    session_id: ctx.session_id.clone(),
                },
                confidence: RULE_CONFIDENCE,
                source: "rules".into(),
                status_message: None,
            });
        }

        if ctx.has_running_session {
            if let Some(session) = &ctx.active_session {
                if is_running_status(session.status) {
                    return Some(RouteResult {
                        intent: QuickIntent::FollowUp {
                            session_id: session.id.clone(),
                            text: trimmed.to_string(),
                        },
                        confidence: 0.75,
                        source: "rules".into(),
                        status_message: None,
                    });
                }
            }
        }

        if Self::is_new_task_intent(&lower) {
            return Some(RouteResult {
                intent: QuickIntent::NewTask {
                    project_id: ctx.project_id.clone().unwrap_or_default(),
                    mode: ctx.mode.clone(),
                    prompt: trimmed.to_string(),
                },
                confidence: 0.7,
                source: "rules".into(),
                status_message: None,
            });
        }

        if let Some(session) = &ctx.active_session {
            if !is_running_status(session.status) {
                return Some(RouteResult {
                    intent: QuickIntent::NewTask {
                        project_id: ctx.project_id.clone().unwrap_or_default(),
                        mode: ctx.mode.clone(),
                        prompt: trimmed.to_string(),
                    },
                    confidence: 0.65,
                    source: "rules".into(),
                    status_message: None,
                });
            }
        }

        // Chat-first: unmatched text starts (or continues) an agent conversation.
        Some(RouteResult {
            intent: QuickIntent::NewTask {
                project_id: ctx.project_id.clone().unwrap_or_default(),
                mode: ctx.mode.clone(),
                prompt: trimmed.to_string(),
            },
            confidence: 0.6,
            source: "default".into(),
            status_message: None,
        })
    }

    fn is_stop_intent(lower: &str) -> bool {
        matches!(
            lower,
            "stop"
                | "pause"
                | "halt"
                | "暂停"
                | "停一下"
                | "先停"
                | "停止"
                | "先停一下"
                | "stop session"
                | "stop task"
        ) || lower.starts_with("stop ")
            || lower.contains("先停")
    }

    fn is_pause_intent(lower: &str) -> bool {
        matches!(lower, "pause task" | "暂停任务" | "先暂停")
    }

    fn is_status_query(lower: &str) -> bool {
        lower.contains("status")
            || lower.contains("progress")
            || lower.contains("做到哪")
            || lower.contains("进展")
            || lower.contains("现在怎么样")
            || lower.contains("what's the status")
            || lower.contains("what is the status")
            || lower.contains("现在做到")
            || lower.contains("到哪了")
            || lower.contains("explain what changed")
            || lower.contains("解释")
            || lower.contains("改了什么")
    }

    fn is_open_chat(lower: &str) -> bool {
        matches!(
            lower,
            "open chat" | "open panel" | "打开聊天" | "打开面板" | "open friday panel"
        )
    }

    fn is_new_task_intent(lower: &str) -> bool {
        lower.starts_with("help me")
            || lower.starts_with("帮我")
            || lower.starts_with("fix ")
            || lower.starts_with("implement ")
            || lower.starts_with("create ")
            || lower.starts_with("add ")
            || lower.contains("新任务")
            || lower.contains("开始任务")
    }

    fn parse_save_idea(text: &str) -> Option<(String, String)> {
        let prefixes = [
            "记一下",
            "save idea:",
            "save idea ",
            "note:",
            "note ",
            "idea:",
            "idea ",
        ];
        for prefix in prefixes {
            if let Some(rest) = text.strip_prefix(prefix) {
                let body = rest.trim();
                if body.is_empty() {
                    return None;
                }
                let title: String = body.chars().take(48).collect();
                return Some((title, body.to_string()));
            }
        }
        None
    }

    fn clarify_fallback(ctx: &RouteContext) -> RouteResult {
        let mut options = vec![
            "Send as follow-up to current session".into(),
            "Start a new Cursor task".into(),
            "Save as idea".into(),
            "Check session status".into(),
        ];
        RouteResult {
            intent: QuickIntent::Clarify {
                message: format!(
                    "I'm not sure how to handle: \"{}\". Pick an action or rephrase.",
                    ctx.text.chars().take(80).collect::<String>()
                ),
                options,
            },
            confidence: 0.3,
            source: "fallback".into(),
            status_message: None,
        }
    }

    async fn llm_classify(ctx: &RouteContext) -> AppResult<RouteResult> {
        // LLM routing uses OpenAI only. Cursor API keys belong to Cloud Agents — never send them to OpenAI.
        let api_key = match SecretStore::get_stt_api_key()? {
            Some(k) => k,
            None => return Err(AppError::Other("no OpenAI STT API key for LLM routing".into())),
        };

        let session_hint = ctx
            .active_session
            .as_ref()
            .map(|s| format!("active_session={} status={:?}", s.id, s.status))
            .unwrap_or_else(|| "no active session".into());

        let prompt = format!(
            r#"Classify this Friday desktop agent user input into exactly one JSON object.
Fields: kind (follow_up|new_task|query_status|control_stop|save_idea|open_chat|clarify), confidence (0-1), prompt (optional text), title (optional for save_idea).
Context: {session_hint}, project_id={:?}, mode={}
User input: {}"#,
            ctx.project_id, ctx.mode, ctx.text
        );

        let body = serde_json::json!({
            "model": "gpt-4o-mini",
            "messages": [
                {"role": "system", "content": "Reply with JSON only."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.1,
            "response_format": { "type": "json_object" }
        });

        let client = reqwest::Client::new();
        let resp = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Other(format!("LLM request failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::Other(format!("LLM HTTP {}", resp.status())));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::Other(format!("LLM parse failed: {e}")))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| AppError::Other("LLM empty response".into()))?;

        Self::parse_llm_json(content, ctx)
    }

    fn parse_llm_json(content: &str, ctx: &RouteContext) -> AppResult<RouteResult> {
        let v: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| AppError::Other(format!("LLM JSON invalid: {e}")))?;

        let kind = v["kind"].as_str().unwrap_or("clarify");
        let confidence = v["confidence"].as_f64().unwrap_or(0.6) as f32;

        let intent = match kind {
            "follow_up" => {
                let session_id = ctx
                    .session_id
                    .clone()
                    .or_else(|| ctx.active_session.as_ref().map(|s| s.id.clone()))
                    .ok_or_else(|| AppError::Other("no session for follow_up".into()))?;
                QuickIntent::FollowUp {
                    session_id,
                    text: v["prompt"]
                        .as_str()
                        .unwrap_or(ctx.text.as_str())
                        .to_string(),
                }
            }
            "new_task" => {
                let project_id = ctx
                    .project_id
                    .clone()
                    .ok_or_else(|| AppError::Other("no project for new_task".into()))?;
                QuickIntent::NewTask {
                    project_id,
                    mode: ctx.mode.clone(),
                    prompt: v["prompt"]
                        .as_str()
                        .unwrap_or(ctx.text.as_str())
                        .to_string(),
                }
            }
            "query_status" => QuickIntent::QueryStatus,
            "control_stop" => QuickIntent::Control {
                action: ControlAction::Stop,
                session_id: ctx.session_id.clone(),
            },
            "save_idea" => QuickIntent::SaveIdea {
                title: v["title"]
                    .as_str()
                    .unwrap_or("Idea")
                    .to_string(),
                body: v["prompt"]
                    .as_str()
                    .unwrap_or(ctx.text.as_str())
                    .to_string(),
                project_id: ctx.project_id.clone(),
                session_id: ctx.session_id.clone(),
            },
            "open_chat" => QuickIntent::OpenChat,
            _ => QuickIntent::Clarify {
                message: v["prompt"]
                    .as_str()
                    .unwrap_or("Please clarify your request.")
                    .to_string(),
                options: vec![],
            },
        };

        Ok(RouteResult {
            intent,
            confidence,
            source: "llm".into(),
            status_message: None,
        })
    }

    pub fn format_status_summary(session: &FridaySession, recent: &[String]) -> String {
        let mut lines = vec![
            format!("Session: {}", session.title),
            format!("Status: {:?}", session.status),
        ];
        if let Some(summary) = &session.summary {
            lines.push(format!("Summary: {summary}"));
        }
        if let Some(repo) = &session.repo {
            if let Some(path) = &repo.local_path {
                lines.push(format!("Repo: {path}"));
            }
            if let Some(branch) = &repo.branch {
                lines.push(format!("Branch: {branch}"));
            }
        }
        for line in recent.iter().take(3) {
            lines.push(format!("· {line}"));
        }
        lines.join("\n")
    }
}

#[allow(dead_code)]
fn _compile_patterns() {
    let _ = Regex::new(r"^stop$").unwrap();
}
