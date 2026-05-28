use crate::core::event::AgentStatus;

pub fn transition(current: AgentStatus, event_status: AgentStatus) -> AgentStatus {
    if matches!(current, AgentStatus::Cancelled | AgentStatus::Error) {
        return current;
    }

    match event_status {
        AgentStatus::Idle => current,
        AgentStatus::Starting => AgentStatus::Starting,
        AgentStatus::Thinking => AgentStatus::Thinking,
        AgentStatus::Reading => AgentStatus::Reading,
        AgentStatus::Editing => AgentStatus::Editing,
        AgentStatus::RunningCommand => AgentStatus::RunningCommand,
        AgentStatus::WaitingApproval => AgentStatus::WaitingApproval,
        AgentStatus::Testing => AgentStatus::Testing,
        AgentStatus::Paused => AgentStatus::Paused,
        AgentStatus::Completed => AgentStatus::Completed,
        AgentStatus::Error => AgentStatus::Error,
        AgentStatus::Cancelled => AgentStatus::Cancelled,
    }
}

pub fn status_from_tool_name(tool: &str) -> AgentStatus {
    let lower = tool.to_lowercase();
    if lower.contains("read") {
        AgentStatus::Reading
    } else if lower.contains("edit") || lower.contains("write") {
        AgentStatus::Editing
    } else if lower.contains("shell") || lower.contains("command") || lower.contains("terminal") {
        AgentStatus::RunningCommand
    } else if lower.contains("test") {
        AgentStatus::Testing
    } else {
        AgentStatus::Thinking
    }
}
