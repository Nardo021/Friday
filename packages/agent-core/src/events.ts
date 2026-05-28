import type {
  FileAction,
  FridaySessionStatus,
  MessageRole,
  RiskLevel,
} from "./sessions.js";

export type AgentEvent =
  | {
      type: "session.discovered";
      sessionId: string;
      source: "process_scan" | "api" | "manual";
      timestamp: string;
    }
  | {
      type: "session.started";
      sessionId: string;
      timestamp: string;
    }
  | {
      type: "agent.status";
      sessionId: string;
      status: FridaySessionStatus;
      message?: string;
      timestamp: string;
    }
  | {
      type: "agent.message";
      sessionId: string;
      role: MessageRole;
      content: string;
      timestamp: string;
    }
  | {
      type: "tool.call";
      sessionId: string;
      toolName: string;
      title: string;
      args?: unknown;
      timestamp: string;
    }
  | {
      type: "file.changed";
      sessionId: string;
      path: string;
      action: FileAction;
      timestamp: string;
    }
  | {
      type: "command.started";
      sessionId: string;
      command: string;
      cwd?: string;
      risk: RiskLevel;
      timestamp: string;
    }
  | {
      type: "command.completed";
      sessionId: string;
      command: string;
      exitCode?: number;
      timestamp: string;
    }
  | {
      type: "approval.required";
      sessionId: string;
      approvalId: string;
      title: string;
      command?: string;
      risk: RiskLevel;
      timestamp: string;
    }
  | {
      type: "artifact.created";
      sessionId: string;
      artifactId: string;
      title: string;
      url?: string;
      timestamp: string;
    }
  | {
      type: "pr.created";
      sessionId: string;
      prUrl: string;
      timestamp: string;
    }
  | {
      type: "session.completed";
      sessionId: string;
      summary?: string;
      timestamp: string;
    }
  | {
      type: "session.error";
      sessionId: string;
      error: string;
      timestamp: string;
    };

export const AGENT_EVENT_CHANNEL = "agent-event";

export function eventSessionId(event: AgentEvent): string {
  return event.sessionId;
}
