import type { AgentCapabilities } from "./capabilities.js";
import type { AgentEvent } from "./events.js";
import type { AgentMode, AgentSessionType, FridaySession } from "./sessions.js";

export interface CreateSessionInput {
  type: AgentSessionType;
  mode: AgentMode;
  projectId: string;
  prompt: string;
  model?: string;
}

export interface AttachSessionInput {
  pid: number;
  adapterId?: string;
}

export interface AgentAdapterContract {
  id: string;
  name: string;
  sessionType: AgentSessionType;
  capabilities: AgentCapabilities;

  createSession(input: CreateSessionInput): Promise<FridaySession>;
  attachSession?(input: AttachSessionInput): Promise<FridaySession>;
  sendMessage?(sessionId: string, message: string): Promise<void>;
  stopSession?(sessionId: string): Promise<void>;
  resumeSession?(sessionId: string): Promise<void>;
  onEvent(callback: (event: AgentEvent) => void): void;
}
