use serde_json::Value;

#[derive(Debug, Clone)]
pub enum CursorIntermediateKind {
    AssistantText,
    ToolUse,
    ToolCompleted,
    FileRead,
    FileEdit,
    CommandRun,
    CommandCompleted,
    Approval,
    SessionStart,
    SessionEnd,
    Error,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct CursorIntermediateEvent {
    pub kind: CursorIntermediateKind,
    pub payload: Value,
    pub raw: String,
}

pub struct CursorParser;

impl CursorParser {
    pub fn parse_line(line: &str) -> CursorIntermediateEvent {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return CursorIntermediateEvent {
                kind: CursorIntermediateKind::Unknown,
                payload: Value::Null,
                raw: line.to_string(),
            };
        }

        if let Ok(json) = serde_json::from_str::<Value>(trimmed) {
            return Self::parse_json(json, trimmed);
        }

        CursorIntermediateEvent {
            kind: classify_plain_text(trimmed),
            payload: Value::String(trimmed.to_string()),
            raw: line.to_string(),
        }
    }

    fn parse_json(json: Value, raw: &str) -> CursorIntermediateEvent {
        let event_type = json
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let kind = match event_type {
            "system" => {
                let subtype = json.get("subtype").and_then(|v| v.as_str()).unwrap_or("");
                if subtype == "init" || subtype.is_empty() {
                    CursorIntermediateKind::SessionStart
                } else {
                    CursorIntermediateKind::Unknown
                }
            }
            "user" => CursorIntermediateKind::Unknown,
            "assistant" => classify_assistant(&json),
            "result" => {
                let is_error = json
                    .get("is_error")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                    || json.get("subtype").and_then(|v| v.as_str()) == Some("error");
                if is_error {
                    CursorIntermediateKind::Error
                } else {
                    CursorIntermediateKind::SessionEnd
                }
            }
            "tool_call" | "tool_use" | "tool" => classify_tool_call(&json),
            "error" => CursorIntermediateKind::Error,
            "content" => CursorIntermediateKind::AssistantText,
            "permission" | "approval" | "ask" => CursorIntermediateKind::Approval,
            _ => {
                if json.get("tool").is_some() || json.get("tool_call").is_some() {
                    classify_tool_call(&json)
                } else {
                    classify_plain_text(raw)
                }
            }
        };

        CursorIntermediateEvent {
            kind,
            payload: json,
            raw: raw.to_string(),
        }
    }
}

/// With `--stream-partial-output`, only deltas with `timestamp_ms` and without
/// `model_call_id` carry new text. Without streaming, complete assistant
/// messages have neither field and should be kept.
fn classify_assistant(json: &Value) -> CursorIntermediateKind {
    let has_ts = json.get("timestamp_ms").is_some();
    let has_model_call = json.get("model_call_id").is_some();

    if has_ts && has_model_call {
        // Buffered duplicate flush before a tool call.
        return CursorIntermediateKind::Unknown;
    }
    if !has_ts && has_model_call {
        return CursorIntermediateKind::Unknown;
    }
    // Streaming delta (timestamp only) OR complete non-streamed message (neither).
    // Final flush has neither timestamp nor model_call_id but is a duplicate when
    // streaming was used — we still accept it when there is message content and
    // no prior streaming context is available; duplicates are harmless for chat.
    if !has_ts {
        // Final flush duplicate under --stream-partial-output: skip if empty-ish
        // and looks like a flush (no delta field). Keep complete messages.
        if json.get("delta").is_none()
            && json.get("message").is_some()
            && json.get("session_id").is_some()
        {
            // Could be either a complete message (no streaming) or final flush.
            // Prefer keeping it — UI can dedupe if needed; dropping loses text.
            return CursorIntermediateKind::AssistantText;
        }
    }
    CursorIntermediateKind::AssistantText
}

fn classify_tool_call(json: &Value) -> CursorIntermediateKind {
    let subtype = json
        .get("subtype")
        .and_then(|v| v.as_str())
        .unwrap_or("started");

    let tool_name = extract_tool_name(json).unwrap_or_default();
    let lower = tool_name.to_lowercase();

    let is_shell = lower.contains("shell")
        || lower.contains("bash")
        || lower.contains("terminal")
        || lower.contains("command");
    let is_read = lower.contains("read");
    let is_edit = lower.contains("edit")
        || lower.contains("write")
        || lower.contains("delete");

    match subtype {
        "completed" => {
            if is_shell {
                CursorIntermediateKind::CommandCompleted
            } else {
                CursorIntermediateKind::ToolCompleted
            }
        }
        _ => {
            if is_shell {
                CursorIntermediateKind::CommandRun
            } else if is_read {
                CursorIntermediateKind::FileRead
            } else if is_edit {
                CursorIntermediateKind::FileEdit
            } else {
                CursorIntermediateKind::ToolUse
            }
        }
    }
}

pub fn extract_tool_name(payload: &Value) -> Option<String> {
    if let Some(name) = payload
        .get("tool")
        .or_else(|| payload.get("name"))
        .and_then(|v| v.as_str())
    {
        return Some(name.to_string());
    }

    if let Some(obj) = payload.get("tool_call").and_then(|v| v.as_object()) {
        if let Some(key) = obj.keys().next() {
            return Some(key.clone());
        }
    }

    if let Some(name) = payload
        .pointer("/tool_call/function/name")
        .and_then(|v| v.as_str())
    {
        return Some(name.to_string());
    }

    None
}

fn classify_plain_text(text: &str) -> CursorIntermediateKind {
    let lower = text.to_lowercase();
    if lower.contains("reading ") || lower.starts_with("read ") {
        CursorIntermediateKind::FileRead
    } else if lower.contains("editing ") || lower.contains("writing ") {
        CursorIntermediateKind::FileEdit
    } else if lower.starts_with('$') || lower.contains("running command") {
        CursorIntermediateKind::CommandRun
    } else if lower.contains("approval") || lower.contains("permission") {
        CursorIntermediateKind::Approval
    } else if lower.contains("error") {
        CursorIntermediateKind::Error
    } else {
        CursorIntermediateKind::AssistantText
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_system_init() {
        let line = r#"{"type":"system","subtype":"init","session_id":"abc"}"#;
        let ev = CursorParser::parse_line(line);
        assert!(matches!(ev.kind, CursorIntermediateKind::SessionStart));
    }

    #[test]
    fn keeps_complete_assistant_without_timestamp() {
        let line = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"hi"}]},"session_id":"abc"}"#;
        let ev = CursorParser::parse_line(line);
        assert!(matches!(ev.kind, CursorIntermediateKind::AssistantText));
    }

    #[test]
    fn skips_buffered_assistant_flush() {
        let line = r#"{"type":"assistant","timestamp_ms":1,"model_call_id":"x","message":{"content":[{"type":"text","text":"dup"}]}}"#;
        let ev = CursorParser::parse_line(line);
        assert!(matches!(ev.kind, CursorIntermediateKind::Unknown));
    }

    #[test]
    fn classifies_shell_tool_started() {
        let line = r#"{"type":"tool_call","subtype":"started","call_id":"1","tool_call":{"shellToolCall":{"args":{"command":"ls"}}}}"#;
        let ev = CursorParser::parse_line(line);
        assert!(matches!(ev.kind, CursorIntermediateKind::CommandRun));
    }

    #[test]
    fn classifies_shell_tool_completed() {
        let line = r#"{"type":"tool_call","subtype":"completed","call_id":"1","tool_call":{"shellToolCall":{"args":{"command":"ls"},"result":{}}}}"#;
        let ev = CursorParser::parse_line(line);
        assert!(matches!(ev.kind, CursorIntermediateKind::CommandCompleted));
    }

    #[test]
    fn classifies_result_success() {
        let line = r#"{"type":"result","subtype":"success","is_error":false,"result":"done"}"#;
        let ev = CursorParser::parse_line(line);
        assert!(matches!(ev.kind, CursorIntermediateKind::SessionEnd));
    }
}
