import type {
  AgentStatus,
  FileAction,
  MessageRole,
  RiskLevel,
} from "./types";

export type AgentEvent =
  | {
      type: "agent_status";
      session_id: string;
      status: AgentStatus;
      message?: string;
      timestamp: string;
    }
  | {
      type: "agent_message";
      session_id: string;
      role: MessageRole;
      text: string;
      timestamp: string;
    }
  | {
      type: "tool_started";
      session_id: string;
      tool: string;
      title: string;
      metadata?: Record<string, unknown>;
      timestamp: string;
    }
  | {
      type: "tool_completed";
      session_id: string;
      tool: string;
      success: boolean;
      output?: string;
      timestamp: string;
    }
  | {
      type: "file_changed";
      session_id: string;
      path: string;
      action: FileAction;
      timestamp: string;
    }
  | {
      type: "command_started";
      session_id: string;
      command: string;
      cwd: string;
      risk: RiskLevel;
      timestamp: string;
    }
  | {
      type: "command_completed";
      session_id: string;
      command: string;
      exit_code: number;
      output?: string;
      timestamp: string;
    }
  | {
      type: "approval_required";
      session_id: string;
      approval_id: string;
      title: string;
      description?: string;
      command?: string;
      risk: RiskLevel;
      timestamp: string;
    }
  | {
      type: "session_started";
      session_id: string;
      adapter_id: string;
      project_id: string;
      timestamp: string;
    }
  | {
      type: "session_completed";
      session_id: string;
      summary?: string;
      timestamp: string;
    }
  | {
      type: "session_error";
      session_id: string;
      message: string;
      timestamp: string;
    };

export const AGENT_EVENT_CHANNEL = "agent-event";
