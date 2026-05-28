export type AgentSessionType =
  | "external_cli"
  | "friday_owned_cli"
  | "cursor_sdk_local"
  | "cursor_cloud";

export type SessionOwnership = "external" | "friday";

export type ControlLevel = "none" | "observe" | "partial" | "full";

export type FridaySessionStatus =
  | "discovered"
  | "idle"
  | "starting"
  | "thinking"
  | "reading"
  | "editing"
  | "running_command"
  | "waiting_permission"
  | "testing"
  | "done"
  | "error"
  | "stopped";

export interface SessionRepo {
  id: string;
  name: string;
  localPath?: string;
  remoteUrl?: string;
  branch?: string;
}

export interface SessionProcess {
  pid?: number;
  ptyId?: string;
  cwd?: string;
}

export interface SessionCloud {
  agentId?: string;
  runId?: string;
  prUrl?: string;
  artifactIds?: string[];
}

export interface FridaySession {
  id: string;
  title: string;
  type: AgentSessionType;
  ownership: SessionOwnership;
  adapterId: string;
  status: FridaySessionStatus;
  controlLevel: ControlLevel;
  projectId?: string;
  prompt?: string;
  summary?: string;
  repo?: SessionRepo;
  process?: SessionProcess;
  cloud?: SessionCloud;
  createdAt: string;
  startedAt?: string;
  updatedAt: string;
  completedAt?: string;
}

export type MessageRole = "user" | "assistant" | "system";
export type FileAction = "created" | "edited" | "deleted";
export type RiskLevel = "low" | "medium" | "high";

export interface Project {
  id: string;
  name: string;
  path: string;
  projectType?: string;
  remoteUrl?: string;
  trusted: boolean;
  defaultAdapterId: string;
  createdAt: string;
  lastUsedAt: string;
}

export type AgentMode = "local_cli" | "sdk_local" | "cloud_agent";

export type TimelineItem =
  | { kind: "message"; role: MessageRole; content: string; timestamp: string }
  | { kind: "tool"; toolName: string; title: string; timestamp: string }
  | { kind: "command"; command: string; risk: RiskLevel; timestamp: string }
  | { kind: "file"; path: string; action: FileAction; timestamp: string }
  | {
      kind: "approval";
      approvalId: string;
      command?: string;
      risk: RiskLevel;
      timestamp: string;
    }
  | {
      kind: "status";
      status: FridaySessionStatus;
      message?: string;
      timestamp: string;
    }
  | { kind: "artifact"; artifactId: string; title: string; timestamp: string }
  | { kind: "pr"; prUrl: string; timestamp: string };

export function isRunningStatus(status: FridaySessionStatus): boolean {
  return [
    "starting",
    "thinking",
    "reading",
    "editing",
    "running_command",
    "waiting_permission",
    "testing",
  ].includes(status);
}
