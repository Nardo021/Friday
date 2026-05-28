use serde_json::Value;

#[derive(Debug, Clone)]
pub enum CursorIntermediateKind {
    AssistantText,
    ToolUse,
    FileRead,
    FileEdit,
    CommandRun,
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
            "start" => CursorIntermediateKind::SessionStart,
            "assistant" => {
                if json.get("timestamp_ms").is_some() {
                    CursorIntermediateKind::AssistantText
                } else {
                    CursorIntermediateKind::Unknown
                }
            }
            "result" => CursorIntermediateKind::SessionEnd,
            "tool_use" | "tool_call" | "tool" => CursorIntermediateKind::ToolUse,
            "error" => CursorIntermediateKind::Error,
            "content" => CursorIntermediateKind::AssistantText,
            _ => {
                if json.get("tool").is_some() {
                    CursorIntermediateKind::ToolUse
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

fn classify_plain_text(text: &str) -> CursorIntermediateKind {
    let lower = text.to_lowercase();
    if lower.contains("reading ") || lower.starts_with("read ") {
        CursorIntermediateKind::FileRead
    } else if lower.contains("editing ") || lower.contains("writing ") {
        CursorIntermediateKind::FileEdit
    } else if lower.starts_with("$") || lower.contains("running command") {
        CursorIntermediateKind::CommandRun
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
    fn parses_assistant_delta() {
        let line = r#"{"type":"assistant","timestamp_ms":1,"message":{"content":[{"text":"hello"}]}}"#;
        let ev = CursorParser::parse_line(line);
        assert!(matches!(ev.kind, CursorIntermediateKind::AssistantText));
    }

    #[test]
    fn skips_buffered_assistant() {
        let line = r#"{"type":"assistant","model_call_id":"abc","message":{"content":[{"text":"dup"}]}}"#;
        let ev = CursorParser::parse_line(line);
        assert!(matches!(ev.kind, CursorIntermediateKind::Unknown));
    }

    #[test]
    fn plain_text_fallback() {
        let ev = CursorParser::parse_line("Reading src/main.rs");
        assert!(matches!(ev.kind, CursorIntermediateKind::FileRead));
    }
}
