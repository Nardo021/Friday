use crate::adapters::cursor_cli_local::cursor_event_mapper::CursorEventMapper;
use crate::adapters::cursor_cli_local::cursor_parser::CursorParser;
use crate::adapters::r#trait::EventHandler;
use crate::core::event::{AgentEvent, MessageRole, now_iso};

/// Feed PTY/pipe bytes into the NDJSON line parser with carry-over buffering.
pub fn feed_parser_lines(
    session_id: &str,
    buffer: &mut String,
    chunk: &[u8],
    handler: &EventHandler,
) {
    buffer.push_str(&String::from_utf8_lossy(chunk));

    while let Some(newline) = buffer.find('\n') {
        let mut line = buffer[..newline].to_string();
        buffer.drain(..=newline);
        if line.ends_with('\r') {
            line.pop();
        }
        if line.is_empty() {
            continue;
        }

        let intermediate = CursorParser::parse_line(&line);
        for event in CursorEventMapper::map(session_id, intermediate) {
            handler(event);
        }
    }
}

pub fn flush_parser_buffer(session_id: &str, buffer: &mut String, handler: &EventHandler) {
    let line = buffer.trim();
    if line.is_empty() {
        buffer.clear();
        return;
    }
    let intermediate = CursorParser::parse_line(line);
    for event in CursorEventMapper::map(session_id, intermediate) {
        handler(event);
    }
    buffer.clear();
}

pub fn emit_stderr_line(session_id: &str, line: &str, handler: &EventHandler) {
    if line.trim().is_empty() {
        return;
    }
    handler(AgentEvent::AgentMessage {
        session_id: session_id.to_string(),
        role: MessageRole::System,
        content: line.to_string(),
        timestamp: now_iso(),
    });
}
