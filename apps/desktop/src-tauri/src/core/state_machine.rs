use crate::core::event::FridaySessionStatus;

pub fn transition(current: FridaySessionStatus, event_status: FridaySessionStatus) -> FridaySessionStatus {
    if matches!(
        current,
        FridaySessionStatus::Stopped | FridaySessionStatus::Error
    ) {
        return current;
    }

    match event_status {
        FridaySessionStatus::Idle | FridaySessionStatus::Discovered => current,
        _ => event_status,
    }
}

pub fn status_from_tool_name(tool: &str) -> FridaySessionStatus {
    let lower = tool.to_lowercase();
    if lower.contains("read") {
        FridaySessionStatus::Reading
    } else if lower.contains("edit") || lower.contains("write") {
        FridaySessionStatus::Editing
    } else if lower.contains("shell") || lower.contains("command") || lower.contains("terminal") {
        FridaySessionStatus::RunningCommand
    } else if lower.contains("test") {
        FridaySessionStatus::Testing
    } else {
        FridaySessionStatus::Thinking
    }
}
