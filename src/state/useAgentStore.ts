import { create } from "zustand";

import type { AgentEvent } from "@/agent/events";
import type { AgentSession, AgentStatus, TimelineItem } from "@/agent/types";
import { redactSecrets } from "@/lib/redaction";

interface AgentState {
  currentSession: AgentSession | null;
  status: AgentStatus;
  statusMessage?: string;
  timeline: TimelineItem[];
  pendingApproval?: {
    approvalId: string;
    command?: string;
    risk: "low" | "medium" | "high";
  };
  setSession: (session: AgentSession | null) => void;
  handleEvent: (event: AgentEvent) => void;
  clearTimeline: () => void;
}

function eventToTimeline(event: AgentEvent): TimelineItem | null {
  switch (event.type) {
    case "agent_message":
      return {
        kind: "message",
        role: event.role,
        text: redactSecrets(event.text),
        timestamp: event.timestamp,
      };
    case "tool_started":
      return {
        kind: "tool",
        tool: event.tool,
        title: event.title,
        timestamp: event.timestamp,
      };
    case "command_started":
      return {
        kind: "command",
        command: event.command,
        risk: event.risk,
        timestamp: event.timestamp,
      };
    case "file_changed":
      return {
        kind: "file",
        path: event.path,
        action: event.action,
        timestamp: event.timestamp,
      };
    case "approval_required":
      return {
        kind: "approval",
        approvalId: event.approval_id,
        command: event.command,
        risk: event.risk,
        timestamp: event.timestamp,
      };
    case "agent_status":
      return {
        kind: "status",
        status: event.status,
        message: event.message,
        timestamp: event.timestamp,
      };
    default:
      return null;
  }
}

export const useAgentStore = create<AgentState>((set, get) => ({
  currentSession: null,
  status: "idle",
  timeline: [],
  setSession: (session) => set({ currentSession: session }),
  clearTimeline: () => set({ timeline: [] }),
  handleEvent: (event) => {
    const item = eventToTimeline(event);
    const updates: Partial<AgentState> = {};

    if (item) {
      updates.timeline = [...get().timeline, item];
    }

    if (event.type === "agent_status") {
      updates.status = event.status;
      updates.statusMessage = event.message;
    }

    if (event.type === "approval_required") {
      updates.pendingApproval = {
        approvalId: event.approval_id,
        command: event.command,
        risk: event.risk,
      };
      updates.status = "waiting_approval";
    }

    if (event.type === "session_completed") {
      updates.status = "completed";
      updates.pendingApproval = undefined;
    }

    if (event.type === "session_error") {
      updates.status = "error";
      updates.statusMessage = event.message;
    }

    set(updates);
  },
}));
