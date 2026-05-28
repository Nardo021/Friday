import type { ControlLevel } from "./sessions.js";

export interface AgentCapabilities {  canCreate: boolean;
  canAttach: boolean;
  canObserve: boolean;
  canSendFollowUp: boolean;
  canStop: boolean;
  canResume: boolean;
  canStreamEvents: boolean;
  canReadArtifacts: boolean;
  canOpenPR: boolean;
  controlLevel: ControlLevel;
}

export interface AdapterInfo {
  id: string;
  name: string;
  available: boolean;
  sessionType: import("./sessions.js").AgentSessionType;
  capabilities: AgentCapabilities;
}

export const EXTERNAL_CURSOR_OBSERVER_CAPABILITIES: AgentCapabilities = {
  canCreate: false,
  canAttach: true,
  canObserve: true,
  canSendFollowUp: false,
  canStop: false,
  canResume: false,
  canStreamEvents: false,
  canReadArtifacts: false,
  canOpenPR: false,
  controlLevel: "observe",
};

export const FRIDAY_OWNED_CLI_CAPABILITIES: AgentCapabilities = {
  canCreate: true,
  canAttach: false,
  canObserve: true,
  canSendFollowUp: true,
  canStop: true,
  canResume: false,
  canStreamEvents: true,
  canReadArtifacts: false,
  canOpenPR: false,
  controlLevel: "full",
};

export const CURSOR_CLOUD_CAPABILITIES: AgentCapabilities = {
  canCreate: true,
  canAttach: true,
  canObserve: true,
  canSendFollowUp: true,
  canStop: true,
  canResume: true,
  canStreamEvents: true,
  canReadArtifacts: true,
  canOpenPR: true,
  controlLevel: "full",
};

export const STUB_CAPABILITIES: AgentCapabilities = {
  canCreate: false,
  canAttach: false,
  canObserve: false,
  canSendFollowUp: false,
  canStop: false,
  canResume: false,
  canStreamEvents: false,
  canReadArtifacts: false,
  canOpenPR: false,
  controlLevel: "none",
};
