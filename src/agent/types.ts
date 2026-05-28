export type AgentStatus =
  | "idle"
  | "starting"
  | "thinking"
  | "reading"
  | "editing"
  | "running_command"
  | "waiting_approval"
  | "testing"
  | "paused"
  | "completed"
  | "error"
  | "cancelled";

export type MessageRole = "user" | "assistant" | "system";
export type FileAction = "created" | "edited" | "deleted";
export type RiskLevel = "low" | "medium" | "high";

export interface AgentSession {
  id: string;
  title: string;
  adapterId: string;
  projectId: string;
  cwd: string;
  status: AgentStatus;
  prompt: string;
  summary?: string;
  createdAt: string;
  startedAt?: string;
  completedAt?: string;
  model?: string;
  branch?: string;
  pid?: number;
}

export interface Project {
  id: string;
  name: string;
  path: string;
  projectType?: string;
  trusted: boolean;
  defaultAdapterId: string;
  createdAt: string;
  lastUsedAt: string;
}

export interface AgentCapabilities {
  supportsStreaming: boolean;
  supportsInteractiveInput: boolean;
  supportsApprovals: boolean;
  supportsFileChangeEvents: boolean;
  supportsCommandEvents: boolean;
  supportsSessionResume: boolean;
  supportsStop: boolean;
}

export interface AdapterInfo {
  id: string;
  name: string;
  available: boolean;
  capabilities: AgentCapabilities;
}

export interface FridaySettings {
  appearance: {
    theme: "system" | "light" | "dark";
    accentColor: string;
    petScale: number;
    reducedMotion: boolean;
  };
  behavior: {
    launchAtStartup: boolean;
    alwaysOnTop: boolean;
    showBubbleOnStatusChange: boolean;
    autoCollapseBubble: boolean;
    soundEffects: boolean;
  };
  security: {
    requireApprovalForHighRiskCommands: boolean;
    requireApprovalForMediumRiskCommands: boolean;
    redactSecrets: boolean;
    allowShellCommands: boolean;
  };
  cursor: {
    executablePath?: string;
    defaultMode: "interactive" | "headless";
    defaultOutputFormat: "text" | "json" | "stream-json";
  };
}

export type TimelineItem =
  | { kind: "message"; role: MessageRole; text: string; timestamp: string }
  | { kind: "tool"; tool: string; title: string; timestamp: string }
  | { kind: "command"; command: string; risk: RiskLevel; timestamp: string }
  | { kind: "file"; path: string; action: FileAction; timestamp: string }
  | { kind: "approval"; approvalId: string; command?: string; risk: RiskLevel; timestamp: string }
  | { kind: "status"; status: AgentStatus; message?: string; timestamp: string };
