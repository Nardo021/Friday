import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  AgentEvent,
  AgentMode,
  AgentSessionType,
  AdapterInfo,
  FridaySession,
  FridaySettings,
  MobileBridgeSettingsView,
  Project,
} from "@friday/agent-core";
import { AGENT_EVENT_CHANNEL } from "@friday/agent-core";

import { usePanelPageStore } from "@/state/usePanelPageStore";
import type { PanelPageId } from "@/windows/panel/PanelNav";

export interface CreateSessionInput {
  /** Empty = general workspace (home directory), no repo required. */
  projectId?: string;
  prompt: string;
  mode?: AgentMode;
  type?: AgentSessionType;
  model?: string;
}

export interface ResizeTerminalInput {
  sessionId: string;
  cols: number;
  rows: number;
}

export async function createSession(
  input: CreateSessionInput,
): Promise<FridaySession> {
  return invoke("create_session", {
    sessionType: input.type ?? "friday_owned_cli",
    projectId: input.projectId ?? "",
    prompt: input.prompt,
  });
}

export async function closeSession(sessionId: string): Promise<void> {
  return invoke("close_session", { sessionId });
}

export async function selectActiveSession(
  sessionId: string | null,
): Promise<void> {
  if (!sessionId) return;
  return invoke("select_active_session", { sessionId });
}

export async function resizeTerminal(
  input: ResizeTerminalInput,
): Promise<void> {
  return invoke("resize_terminal", {
    sessionId: input.sessionId,
    cols: input.cols,
    rows: input.rows,
  });
}

export async function listActiveSessions(): Promise<FridaySession[]> {
  return invoke("list_active_sessions");
}

export async function followUp(
  sessionId: string,
  message: string,
): Promise<void> {
  return invoke("follow_up", { sessionId, message });
}

export async function listSessions(): Promise<FridaySession[]> {
  return invoke("list_sessions");
}

export async function getSessionStatus(
  sessionId: string,
): Promise<FridaySession> {
  return invoke("get_session_status", { sessionId });
}

export async function getSessionEvents(
  sessionId: string,
): Promise<AgentEvent[]> {
  return invoke("get_session_events", { sessionId });
}

export async function approveCommand(approvalId: string): Promise<void> {
  return invoke("approve_command", { approvalId });
}

export async function rejectCommand(approvalId: string): Promise<void> {
  return invoke("reject_command", { approvalId });
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

export interface CursorCliProbe {
  found: boolean;
  path: string;
  error?: string;
}

export async function probeCursorCli(): Promise<CursorCliProbe> {
  return invoke("probe_cursor_cli");
}

export async function saveSettings(
  settings: FridaySettings,
): Promise<void> {
  return invoke("save_settings", { settings });
}

export async function listAdapters(): Promise<AdapterInfo[]> {
  return invoke("list_adapters");
}

export async function openPanel(page?: PanelPageId): Promise<void> {
  if (page) usePanelPageStore.getState().setPage(page);
  return invoke("open_panel");
}

/** Opens quick chat, or hides it if already visible (pet click / shortcut). */
export async function openQuickBubble(): Promise<void> {
  return invoke("open_quick_bubble");
}

/** @deprecated Use openPanel(page) — command center is merged into panel. */
export async function openCommandCenter(page?: PanelPageId): Promise<void> {
  return openPanel(page);
}

export async function showWindow(label: string): Promise<void> {
  return invoke("show_window", { label });
}

export interface WindowPosition {
  x: number;
  y: number;
}

export interface MonitorInfo {
  name?: string;
  x: number;
  y: number;
  width: number;
  height: number;
  scaleFactor: number;
  workAreaX: number;
  workAreaY: number;
  workAreaWidth: number;
  workAreaHeight: number;
}

export async function getPetPosition(): Promise<WindowPosition> {
  return invoke("get_pet_position");
}

export async function setPetPosition(
  x: number,
  y: number,
): Promise<WindowPosition> {
  return invoke("set_pet_position", { x, y });
}

export async function getWindowPosition(label: string): Promise<WindowPosition> {
  return invoke("get_window_position", { label });
}

export async function setWindowPosition(
  label: string,
  x: number,
  y: number,
): Promise<WindowPosition> {
  return invoke("set_window_position", { label, x, y });
}

export async function anchorWindow(
  label: string,
  offsetX: number,
  offsetY: number,
): Promise<WindowPosition> {
  return invoke("anchor_window", { label, offsetX, offsetY });
}

export async function setWindowClickThrough(
  label: string,
  enabled: boolean,
): Promise<void> {
  return invoke("set_window_click_through", { label, enabled });
}

export async function getMonitorInfo(): Promise<MonitorInfo> {
  return invoke("get_monitor_info");
}

export async function showStatusBubble(): Promise<void> {
  return invoke("show_status_bubble");
}

export async function hideStatusBubble(): Promise<void> {
  return invoke("hide_status_bubble");
}

export async function hideQuickBubble(): Promise<void> {
  return invoke("hide_quick_bubble");
}

export async function finishOnboarding(): Promise<void> {
  return invoke("finish_onboarding");
}

export async function saveCursorApiKey(apiKey: string): Promise<void> {
  return invoke("save_cursor_api_key", { apiKey });
}

export async function verifyCursorApiKey(apiKey: string): Promise<void> {
  return invoke("verify_cursor_api_key", { apiKey });
}

export async function clearCursorApiKey(): Promise<void> {
  return invoke("clear_cursor_api_key");
}

export async function getLocalDataPath(): Promise<string> {
  return invoke("get_local_data_path");
}

export async function clearLocalData(): Promise<void> {
  return invoke("clear_local_data");
}

export async function openOnboarding(): Promise<void> {
  return invoke("open_onboarding");
}

export async function hideWindow(label: string): Promise<void> {
  return invoke("hide_window", { label });
}

export async function petSurfaceReady(): Promise<void> {
  return invoke("pet_surface_ready");
}

export type QuickIntentKind =
  | { kind: "followUp"; sessionId: string; text: string }
  | { kind: "newTask"; projectId: string; mode: string; prompt: string }
  | { kind: "queryStatus" }
  | { kind: "control"; action: "stop" | "pause" | "resume"; sessionId?: string }
  | { kind: "saveIdea"; title: string; body: string; projectId?: string; sessionId?: string }
  | { kind: "openChat" }
  | { kind: "clarify"; message: string; options: string[] };

export interface RouteResult {
  intent: QuickIntentKind;
  confidence: number;
  source: string;
  statusMessage?: string;
}

export interface SubmitQuickInputResult {
  route: RouteResult;
  executed: boolean;
  message?: string;
  sessionId?: string;
}

export interface Idea {
  id: string;
  title: string;
  body: string;
  projectId?: string;
  sessionId?: string;
  createdAt: string;
}

export interface StoredMessage {
  id: string;
  sessionId: string;
  role: string;
  content: string;
  createdAt: string;
}

export interface TranscriptionResult {
  transcript: string;
  durationMs: number;
}

export async function submitQuickInput(params: {
  text: string;
  sessionId?: string | null;
  projectId?: string | null;
  mode?: string;
}): Promise<SubmitQuickInputResult> {
  return invoke("submit_quick_input", {
    params: {
      text: params.text,
      sessionId: params.sessionId ?? null,
      projectId: params.projectId ?? null,
      mode: params.mode ?? "local_cli",
    },
  });
}

export async function executeQuickIntent(
  intent: QuickIntentKind,
): Promise<SubmitQuickInputResult> {
  return invoke("execute_quick_intent", { intent });
}

export async function transcribeAudio(
  audioBase64: string,
  language?: string,
): Promise<TranscriptionResult> {
  return invoke("transcribe_audio", { audioBase64, language });
}

export async function listIdeas(): Promise<Idea[]> {
  return invoke("list_ideas");
}

export async function deleteIdea(id: string): Promise<void> {
  return invoke("delete_idea", { id });
}

export async function listMessages(sessionId: string): Promise<StoredMessage[]> {
  return invoke("list_messages", { sessionId });
}

export async function exportSessionMarkdown(sessionId: string): Promise<string> {
  return invoke("export_session_markdown", { sessionId });
}

export async function deleteSession(sessionId: string): Promise<void> {
  return invoke("delete_session", { sessionId });
}

export async function saveSttApiKey(apiKey: string): Promise<void> {
  return invoke("save_stt_api_key", { apiKey });
}

export async function clearSttApiKey(): Promise<void> {
  return invoke("clear_stt_api_key");
}

export async function getMobileBridgeSettings(): Promise<MobileBridgeSettingsView> {
  return invoke("get_mobile_bridge_settings");
}

export async function updateMobileBridgeSettings(input: {
  enabled: boolean;
  port: number;
  authToken: string;
}): Promise<MobileBridgeSettingsView> {
  return invoke("update_mobile_bridge_settings", { input });
}

export async function regenerateMobileBridgeToken(): Promise<MobileBridgeSettingsView> {
  return invoke("regenerate_mobile_bridge_token");
}

export async function getLocalBridgeUrl(): Promise<string> {
  return invoke("get_local_bridge_url");
}

export function listenAgentEvents(
  handler: (event: AgentEvent) => void,
): Promise<UnlistenFn> {
  return listen<AgentEvent>(AGENT_EVENT_CHANNEL, (e) => handler(e.payload));
}

/** @deprecated Use createSession */
export async function startAgentSession(
  projectId: string,
  prompt: string,
): Promise<FridaySession> {
  return createSession({
    projectId,
    prompt,
    mode: "local_cli",
    type: "friday_owned_cli",
  });
}

/** @deprecated Use closeSession */
export async function stopAgentSession(sessionId: string): Promise<void> {
  return closeSession(sessionId);
}

/** @deprecated Use followUp */
export async function sendAgentMessage(
  sessionId: string,
  message: string,
): Promise<void> {
  return followUp(sessionId, message);
}

/** @deprecated Use openPanel */
export async function openChat(): Promise<void> {
  return openPanel();
}
