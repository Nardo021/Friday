import type { AgentStatus } from "./types";

export const STATUS_LABELS: Record<AgentStatus, string> = {
  idle: "Idle",
  starting: "Starting",
  thinking: "Thinking",
  reading: "Reading",
  editing: "Editing",
  running_command: "Running command",
  waiting_approval: "Waiting approval",
  testing: "Testing",
  paused: "Paused",
  completed: "Completed",
  error: "Error",
  cancelled: "Cancelled",
};

export function isRunningStatus(status: AgentStatus): boolean {
  return [
    "starting",
    "thinking",
    "reading",
    "editing",
    "running_command",
    "waiting_approval",
    "testing",
    "paused",
  ].includes(status);
}
