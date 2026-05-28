import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { AgentEvent } from "@/agent/events";
import { AGENT_EVENT_CHANNEL } from "@/agent/events";
import type {
  AdapterInfo,
  AgentSession,
  FridaySettings,
  Project,
} from "@/agent/types";

export async function startAgentSession(
  projectId: string,
  prompt: string,
): Promise<AgentSession> {
  return invoke("start_agent_session", { projectId, prompt });
}

export async function stopAgentSession(sessionId: string): Promise<void> {
  return invoke("stop_agent_session", { sessionId });
}

export async function sendAgentMessage(
  sessionId: string,
  message: string,
): Promise<void> {
  return invoke("send_agent_message", { sessionId, message });
}

export async function getSessionStatus(
  sessionId: string,
): Promise<AgentSession> {
  return invoke("get_session_status", { sessionId });
}

export async function approveCommand(approvalId: string): Promise<void> {
  return invoke("approve_command", { approvalId });
}

export async function rejectCommand(approvalId: string): Promise<void> {
  return invoke("reject_command", { approvalId });
}

export async function listSessions(): Promise<AgentSession[]> {
  return invoke("list_sessions");
}

export async function getSessionEvents(
  sessionId: string,
): Promise<AgentEvent[]> {
  return invoke("get_session_events", { sessionId });
}

export async function addProject(
  name: string,
  path: string,
  trusted = true,
): Promise<Project> {
  return invoke("add_project", { name, path, trusted });
}

export async function listProjects(): Promise<Project[]> {
  return invoke("list_projects");
}

export async function getSettings(): Promise<FridaySettings> {
  return invoke("get_settings");
}

export async function saveSettings(
  settings: FridaySettings,
): Promise<void> {
  return invoke("save_settings", { settings });
}

export async function listAdapters(): Promise<AdapterInfo[]> {
  return invoke("list_adapters");
}

export async function openChat(): Promise<void> {
  return invoke("open_chat");
}

export async function openQuickBubble(): Promise<void> {
  return invoke("open_quick_bubble");
}

export async function openCommandCenter(): Promise<void> {
  return invoke("open_command_center");
}

export async function showWindow(label: string): Promise<void> {
  return invoke("show_window", { label });
}

export function listenAgentEvents(
  handler: (event: AgentEvent) => void,
): Promise<UnlistenFn> {
  return listen<AgentEvent>(AGENT_EVENT_CHANNEL, (e) => handler(e.payload));
}
