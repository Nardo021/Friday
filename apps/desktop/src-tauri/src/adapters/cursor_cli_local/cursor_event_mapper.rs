use serde_json::Value;

use crate::adapters::cursor_cli_local::cursor_parser::{CursorIntermediateEvent, CursorIntermediateKind};
use crate::core::event::{
    AgentEvent, FileAction, FridaySessionStatus, MessageRole, RiskLevel, now_iso,
};
use crate::security::risk_classifier::classify_command_risk;

pub struct CursorEventMapper;

impl CursorEventMapper {
    pub fn map(session_id: &str, event: CursorIntermediateEvent) -> Vec<AgentEvent> {
        match event.kind {
            CursorIntermediateKind::Unknown => vec![],
            CursorIntermediateKind::SessionStart => vec![
                AgentEvent::SessionStarted {
                    session_id: session_id.to_string(),
                    timestamp: now_iso(),
                },
                AgentEvent::AgentStatus {
                    session_id: session_id.to_string(),
                    status: FridaySessionStatus::Thinking,
                    message: Some("Session started".into()),
                    timestamp: now_iso(),
                },
            ],
            CursorIntermediateKind::AssistantText => {
                if let Some(text) = extract_assistant_text(&event.payload) {
                    if text.is_empty() {
                        return vec![];
                    }
                    vec![
                        AgentEvent::AgentMessage {
                            session_id: session_id.to_string(),
                            role: MessageRole::Assistant,
                            content: text,
                            timestamp: now_iso(),
                        },
                        AgentEvent::AgentStatus {
                            session_id: session_id.to_string(),
                            status: FridaySessionStatus::Thinking,
                            message: None,
                            timestamp: now_iso(),
                        },
                    ]
                } else if let Value::String(s) = event.payload {
                    vec![AgentEvent::AgentMessage {
                        session_id: session_id.to_string(),
                        role: MessageRole::Assistant,
                        content: s,
                        timestamp: now_iso(),
                    }]
                } else {
                    vec![]
                }
            }
            CursorIntermediateKind::ToolUse => {
                let tool_name = event
                    .payload
                    .get("tool")
                    .or_else(|| event.payload.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("tool")
                    .to_string();
                let title = event
                    .payload
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&tool_name)
                    .to_string();
                let status = crate::core::state_machine::status_from_tool_name(&tool_name);
                vec![
                    AgentEvent::ToolCall {
                        session_id: session_id.to_string(),
                        tool_name: tool_name.clone(),
                        title,
                        args: Some(event.payload.clone()),
                        timestamp: now_iso(),
                    },
                    AgentEvent::AgentStatus {
                        session_id: session_id.to_string(),
                        status,
                        message: Some(format!("Using {tool_name}")),
                        timestamp: now_iso(),
                    },
                ]
            }
            CursorIntermediateKind::FileRead => {
                let msg = extract_text_or_raw(&event);
                vec![AgentEvent::AgentStatus {
                    session_id: session_id.to_string(),
                    status: FridaySessionStatus::Reading,
                    message: Some(msg),
                    timestamp: now_iso(),
                }]
            }
            CursorIntermediateKind::FileEdit => {
                let path = extract_path(&event.payload).unwrap_or_else(|| "unknown".into());
                vec![
                    AgentEvent::FileChanged {
                        session_id: session_id.to_string(),
                        path,
                        action: FileAction::Edited,
                        timestamp: now_iso(),
                    },
                    AgentEvent::AgentStatus {
                        session_id: session_id.to_string(),
                        status: FridaySessionStatus::Editing,
                        message: None,
                        timestamp: now_iso(),
                    },
                ]
            }
            CursorIntermediateKind::CommandRun => {
                let command = extract_text_or_raw(&event);
                let risk = classify_command_risk(&command);
                vec![AgentEvent::CommandStarted {
                    session_id: session_id.to_string(),
                    command: command.clone(),
                    cwd: None,
                    risk,
                    timestamp: now_iso(),
                }]
            }
            CursorIntermediateKind::Approval => vec![AgentEvent::ApprovalRequired {
                session_id: session_id.to_string(),
                approval_id: uuid::Uuid::new_v4().to_string(),
                title: "Approval required".into(),
                command: Some(extract_text_or_raw(&event)),
                risk: RiskLevel::High,
                timestamp: now_iso(),
            }],
            CursorIntermediateKind::SessionEnd => {
                let summary = event
                    .payload
                    .get("result")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                vec![
                    AgentEvent::SessionCompleted {
                        session_id: session_id.to_string(),
                        summary: summary.clone(),
                        timestamp: now_iso(),
                    },
                    AgentEvent::AgentStatus {
                        session_id: session_id.to_string(),
                        status: FridaySessionStatus::Done,
                        message: summary,
                        timestamp: now_iso(),
                    },
                ]
            }
            CursorIntermediateKind::Error => vec![
                AgentEvent::SessionError {
                    session_id: session_id.to_string(),
                    error: extract_text_or_raw(&event),
                    timestamp: now_iso(),
                },
                AgentEvent::AgentStatus {
                    session_id: session_id.to_string(),
                    status: FridaySessionStatus::Error,
                    message: None,
                    timestamp: now_iso(),
                },
            ],
        }
    }
}

fn extract_assistant_text(payload: &Value) -> Option<String> {
    if let Some(delta) = payload.get("delta").and_then(|v| v.as_str()) {
        return Some(delta.to_string());
    }
    if let Some(content) = payload.get("content").and_then(|v| v.as_str()) {
        return Some(content.to_string());
    }
    payload
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("")
        })
        .filter(|s| !s.is_empty())
}

fn extract_path(payload: &Value) -> Option<String> {
    payload
        .get("path")
        .or_else(|| payload.get("file"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| {
            payload
                .get("args")
                .and_then(|a| a.get("path"))
                .and_then(|v| v.as_str())
                .map(String::from)
        })
}

fn extract_text_or_raw(event: &CursorIntermediateEvent) -> String {
    if let Value::String(s) = &event.payload {
        return s.clone();
    }
    event.raw.clone()
}
