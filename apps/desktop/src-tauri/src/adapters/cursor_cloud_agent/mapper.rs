use serde_json::Value;

use crate::adapters::cursor_cloud_agent::client::{CloudRun, GitBranch};
use crate::core::event::{
    AgentEvent, FridaySessionStatus, MessageRole, now_iso,
};

pub fn map_run_status(status: &str) -> FridaySessionStatus {
    match status.to_uppercase().as_str() {
        "CREATING" => FridaySessionStatus::Starting,
        "RUNNING" => FridaySessionStatus::Thinking,
        "FINISHED" => FridaySessionStatus::Done,
        "ERROR" => FridaySessionStatus::Error,
        "CANCELLED" | "CANCELED" => FridaySessionStatus::Stopped,
        "EXPIRED" => FridaySessionStatus::Stopped,
        _ => FridaySessionStatus::Thinking,
    }
}

pub fn map_sse_event(
    session_id: &str,
    event_type: &str,
    data: &str,
) -> Vec<AgentEvent> {
    let ts = now_iso();
    let payload: Value = serde_json::from_str(data).unwrap_or(Value::Null);

    match event_type {
        "status" => {
            let status = payload["status"].as_str().unwrap_or("RUNNING");
            vec![AgentEvent::AgentStatus {
                session_id: session_id.to_string(),
                status: map_run_status(status),
                message: Some(format!("Cloud run: {status}")),
                timestamp: ts,
            }]
        }
        "assistant" => {
            let text = payload["text"].as_str().unwrap_or("");
            if text.is_empty() {
                return vec![];
            }
            vec![AgentEvent::AgentMessage {
                session_id: session_id.to_string(),
                role: MessageRole::Assistant,
                content: text.to_string(),
                timestamp: ts,
            }]
        }
        "thinking" => {
            let text = payload["text"].as_str().unwrap_or("");
            if text.is_empty() {
                return vec![];
            }
            vec![AgentEvent::AgentStatus {
                session_id: session_id.to_string(),
                status: FridaySessionStatus::Thinking,
                message: Some(text.chars().take(120).collect()),
                timestamp: ts,
            }]
        }
        "tool_call" => {
            let name = payload["name"].as_str().unwrap_or("tool");
            let status = payload["status"].as_str().unwrap_or("running");
            let title = if status == "completed" {
                format!("{name} completed")
            } else {
                format!("{name} running")
            };
            vec![AgentEvent::ToolCall {
                session_id: session_id.to_string(),
                tool_name: name.to_string(),
                title,
                args: payload.get("args").cloned(),
                timestamp: ts,
            }]
        }
        "result" => map_terminal_result(session_id, &payload, ts),
        "error" => {
            let message = payload["message"]
                .as_str()
                .unwrap_or("Cloud agent error");
            vec![AgentEvent::SessionError {
                session_id: session_id.to_string(),
                error: message.to_string(),
                timestamp: ts,
            }]
        }
        _ => vec![],
    }
}

fn map_terminal_result(
    session_id: &str,
    payload: &Value,
    ts: String,
) -> Vec<AgentEvent> {
    let mut events = Vec::new();
    let status = payload["status"].as_str().unwrap_or("FINISHED");
    let friday_status = map_run_status(status);

    if let Some(text) = payload["text"].as_str() {
        if !text.is_empty() {
            events.push(AgentEvent::AgentMessage {
                session_id: session_id.to_string(),
                role: MessageRole::Assistant,
                content: text.to_string(),
                timestamp: ts.clone(),
            });
        }
    }

    if let Some(git) = payload.get("git") {
        events.extend(map_git_branches(session_id, git, &ts));
    }

    match friday_status {
        FridaySessionStatus::Error => {
            events.push(AgentEvent::SessionError {
                session_id: session_id.to_string(),
                error: payload["text"]
                    .as_str()
                    .unwrap_or("Cloud run failed")
                    .to_string(),
                timestamp: ts,
            });
        }
        FridaySessionStatus::Done => {
            events.push(AgentEvent::SessionCompleted {
                session_id: session_id.to_string(),
                summary: payload["text"].as_str().map(String::from),
                timestamp: ts,
            });
        }
        FridaySessionStatus::Stopped => {
            events.push(AgentEvent::AgentStatus {
                session_id: session_id.to_string(),
                status: FridaySessionStatus::Stopped,
                message: Some("Cloud run cancelled".into()),
                timestamp: ts,
            });
        }
        _ => {
            events.push(AgentEvent::AgentStatus {
                session_id: session_id.to_string(),
                status: friday_status,
                message: None,
                timestamp: ts,
            });
        }
    }

    events
}

pub fn map_git_branches(session_id: &str, git: &Value, ts: &str) -> Vec<AgentEvent> {
    let mut events = Vec::new();
    if let Some(branches) = git["branches"].as_array() {
        for branch in branches {
            if let Some(pr_url) = branch["prUrl"].as_str().or_else(|| branch["pr_url"].as_str()) {
                events.push(AgentEvent::PrCreated {
                    session_id: session_id.to_string(),
                    pr_url: pr_url.to_string(),
                    timestamp: ts.to_string(),
                });
            }
        }
    }
    events
}

pub fn map_run_terminal(session_id: &str, run: &CloudRun) -> Vec<AgentEvent> {
    let ts = now_iso();
    let status = run.status.as_str();
    let mut payload = serde_json::json!({
        "status": status,
        "text": run.result,
        "git": run.git,
    });
    if let Some(ms) = run.duration_ms {
        payload["durationMs"] = serde_json::json!(ms);
    }
    map_terminal_result(session_id, &payload, ts)
}

pub fn artifact_events(
    session_id: &str,
    artifacts: &[crate::adapters::cursor_cloud_agent::client::ArtifactItem],
) -> Vec<AgentEvent> {
    let ts = now_iso();
    artifacts
        .iter()
        .map(|a| AgentEvent::ArtifactCreated {
            session_id: session_id.to_string(),
            artifact_id: a.path.clone(),
            title: a.path.clone(),
            url: None,
            timestamp: ts.clone(),
        })
        .collect()
}

#[allow(dead_code)]
pub fn normalize_pr_url(branch: &GitBranch) -> Option<String> {
    branch.pr_url.clone()
}
