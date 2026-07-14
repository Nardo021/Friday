use serde_json::Value;

use crate::adapters::cursor_cli_local::cursor_parser::{
    extract_tool_name, CursorIntermediateEvent, CursorIntermediateKind,
};
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
            CursorIntermediateKind::ToolUse | CursorIntermediateKind::ToolCompleted => {
                let tool_name = extract_tool_name(&event.payload)
                    .unwrap_or_else(|| "tool".into());
                let title = event
                    .payload
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&tool_name)
                    .to_string();
                let status = crate::core::state_machine::status_from_tool_name(&tool_name);
                let mut events = vec![
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
                ];
                if matches!(event.kind, CursorIntermediateKind::ToolCompleted) {
                    if let Some(path) = extract_path(&event.payload) {
                        let action = if tool_name.to_lowercase().contains("delete") {
                            FileAction::Deleted
                        } else if tool_name.to_lowercase().contains("write") {
                            FileAction::Created
                        } else {
                            FileAction::Edited
                        };
                        events.push(AgentEvent::FileChanged {
                            session_id: session_id.to_string(),
                            path,
                            action,
                            timestamp: now_iso(),
                        });
                    }
                }
                events
            }
            CursorIntermediateKind::FileRead => {
                let path = extract_path(&event.payload);
                let msg = path
                    .clone()
                    .unwrap_or_else(|| extract_text_or_raw(&event));
                let mut events = vec![AgentEvent::AgentStatus {
                    session_id: session_id.to_string(),
                    status: FridaySessionStatus::Reading,
                    message: Some(msg),
                    timestamp: now_iso(),
                }];
                if let Some(path) = path {
                    events.insert(
                        0,
                        AgentEvent::ToolCall {
                            session_id: session_id.to_string(),
                            tool_name: "read".into(),
                            title: format!("Read {path}"),
                            args: Some(event.payload.clone()),
                            timestamp: now_iso(),
                        },
                    );
                }
                events
            }
            CursorIntermediateKind::FileEdit => {
                let path = extract_path(&event.payload).unwrap_or_else(|| "unknown".into());
                let tool_name = extract_tool_name(&event.payload).unwrap_or_else(|| "edit".into());
                vec![
                    AgentEvent::ToolCall {
                        session_id: session_id.to_string(),
                        tool_name,
                        title: format!("Edit {path}"),
                        args: Some(event.payload.clone()),
                        timestamp: now_iso(),
                    },
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
                let command = extract_command(&event);
                let risk = classify_command_risk(&command);
                vec![
                    AgentEvent::CommandStarted {
                        session_id: session_id.to_string(),
                        command: command.clone(),
                        cwd: extract_cwd(&event.payload),
                        risk,
                        timestamp: now_iso(),
                    },
                    AgentEvent::AgentStatus {
                        session_id: session_id.to_string(),
                        status: FridaySessionStatus::RunningCommand,
                        message: Some(command),
                        timestamp: now_iso(),
                    },
                ]
            }
            CursorIntermediateKind::CommandCompleted => {
                let command = extract_command(&event);
                vec![
                    AgentEvent::CommandCompleted {
                        session_id: session_id.to_string(),
                        command,
                        exit_code: extract_exit_code(&event.payload),
                        timestamp: now_iso(),
                    },
                    AgentEvent::AgentStatus {
                        session_id: session_id.to_string(),
                        status: FridaySessionStatus::Thinking,
                        message: Some("Command finished".into()),
                        timestamp: now_iso(),
                    },
                ]
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
            CursorIntermediateKind::Error => {
                let error = event
                    .payload
                    .get("result")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| extract_text_or_raw(&event));
                vec![
                    AgentEvent::SessionError {
                        session_id: session_id.to_string(),
                        error,
                        timestamp: now_iso(),
                    },
                    AgentEvent::AgentStatus {
                        session_id: session_id.to_string(),
                        status: FridaySessionStatus::Error,
                        message: None,
                        timestamp: now_iso(),
                    },
                ]
            }
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
        .or_else(|| {
            payload
                .get("tool_call")
                .and_then(|tc| tc.as_object())
                .and_then(|obj| obj.values().next())
                .and_then(|tool| {
                    tool.pointer("/args/path")
                        .or_else(|| tool.pointer("/result/success/path"))
                })
                .and_then(|v| v.as_str())
                .map(String::from)
        })
}

fn extract_command(event: &CursorIntermediateEvent) -> String {
    if let Some(cmd) = event
        .payload
        .pointer("/tool_call/shellToolCall/args/command")
        .or_else(|| event.payload.pointer("/tool_call/bashToolCall/args/command"))
        .or_else(|| event.payload.pointer("/args/command"))
        .and_then(|v| v.as_str())
    {
        return cmd.to_string();
    }
    extract_text_or_raw(event)
}

fn extract_cwd(payload: &Value) -> Option<String> {
    payload
        .pointer("/tool_call/shellToolCall/args/working_directory")
        .or_else(|| payload.pointer("/tool_call/shellToolCall/args/cwd"))
        .or_else(|| payload.pointer("/args/cwd"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

fn extract_exit_code(payload: &Value) -> Option<i32> {
    payload
        .pointer("/tool_call/shellToolCall/result/success/exitCode")
        .or_else(|| payload.pointer("/tool_call/shellToolCall/result/exitCode"))
        .or_else(|| payload.get("exit_code"))
        .and_then(|v| v.as_i64())
        .map(|n| n as i32)
}

fn extract_text_or_raw(event: &CursorIntermediateEvent) -> String {
    if let Value::String(s) = &event.payload {
        return s.clone();
    }
    event.raw.clone()
}
