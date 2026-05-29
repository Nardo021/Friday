import { create } from "zustand";
import { useShallow } from "zustand/react/shallow";

import type {
  AgentEvent,
  FridaySession,
  FridaySessionStatus,
  Project,
  RiskLevel,
  TimelineItem,
} from "@friday/agent-core";
import { isRunningStatus } from "@friday/agent-core";
import { redactSecrets } from "@friday/shared";

import {
  listProjects,
  listSessions,
  selectActiveSession as selectActiveSessionIpc,
} from "@/lib/tauri";

export interface PendingApproval {
  approvalId: string;
  command?: string;
  risk: RiskLevel;
  title?: string;
}

interface SessionState {
  sessions: Record<string, FridaySession>;
  activeSessionId: string | null;
  timelines: Record<string, TimelineItem[]>;
  pendingApprovals: Record<string, PendingApproval>;
  statusMessages: Record<string, string | undefined>;
  projects: Project[];
  selectedProjectId: string | null;
  loading: boolean;
  bootstrap: () => Promise<void>;
  refreshSessions: () => Promise<void>;
  refreshProjects: () => Promise<void>;
  selectActiveSession: (id: string | null) => Promise<void>;
  hydrate: (sessions: FridaySession[]) => void;
  handleEvent: (event: AgentEvent) => void;
  setSelectedProject: (id: string | null) => void;
  clearTimeline: (sessionId: string) => void;
}

let loadingRequests = 0;

function beginLoading(set: (partial: Partial<SessionState>) => void) {
  loadingRequests += 1;
  set({ loading: true });
}

function endLoading(set: (partial: Partial<SessionState>) => void) {
  loadingRequests = Math.max(0, loadingRequests - 1);
  set({ loading: loadingRequests > 0 });
}

function eventToTimeline(event: AgentEvent): TimelineItem | null {
  switch (event.type) {
    case "agent.message":
      return {
        kind: "message",
        role: event.role,
        content: redactSecrets(event.content),
        timestamp: event.timestamp,
      };
    case "tool.call":
      return {
        kind: "tool",
        toolName: event.toolName,
        title: event.title,
        timestamp: event.timestamp,
      };
    case "command.started":
      return {
        kind: "command",
        command: event.command,
        risk: event.risk,
        timestamp: event.timestamp,
      };
    case "file.changed":
      return {
        kind: "file",
        path: event.path,
        action: event.action,
        timestamp: event.timestamp,
      };
    case "approval.required":
      return {
        kind: "approval",
        approvalId: event.approvalId,
        command: event.command,
        risk: event.risk,
        timestamp: event.timestamp,
      };
    case "agent.status":
      return {
        kind: "status",
        status: event.status,
        message: event.message,
        timestamp: event.timestamp,
      };
    case "artifact.created":
      return {
        kind: "artifact",
        artifactId: event.artifactId,
        title: event.title,
        timestamp: event.timestamp,
      };
    case "pr.created":
      return {
        kind: "pr",
        prUrl: event.prUrl,
        timestamp: event.timestamp,
      };
    case "session.error":
      return {
        kind: "status",
        status: "error",
        message: event.error,
        timestamp: event.timestamp,
      };
    case "session.completed":
      return {
        kind: "status",
        status: "done",
        message: event.summary,
        timestamp: event.timestamp,
      };
    default:
      return null;
  }
}

function sessionsRecord(list: FridaySession[]): Record<string, FridaySession> {
  return Object.fromEntries(list.map((s) => [s.id, s]));
}

const EMPTY_TIMELINE: TimelineItem[] = [];

/** Matches Rust `SURFACE_SESSION_ID` for errors before a session exists. */
export const SURFACE_SESSION_ID = "friday-surface";
const MAX_TIMELINE_ITEMS = 250;

export const useSessionStore = create<SessionState>((set, get) => ({
  sessions: {},
  activeSessionId: null,
  timelines: {},
  pendingApprovals: {},
  statusMessages: {},
  projects: [],
  selectedProjectId: null,
  loading: false,

  setSelectedProject: (id) => set({ selectedProjectId: id }),

  hydrate: (list) => {
    const sessions = sessionsRecord(list);
    set((state) => {
      const nextActive =
        state.activeSessionId && sessions[state.activeSessionId]
          ? state.activeSessionId
          : list.find((s) => isRunningStatus(s.status))?.id ??
            state.activeSessionId ??
            list[0]?.id ??
            null;

      const sessionsChanged =
        list.length !== Object.keys(state.sessions).length ||
        list.some((s) => {
          const prev = state.sessions[s.id];
          return (
            !prev ||
            prev.status !== s.status ||
            prev.updatedAt !== s.updatedAt ||
            prev.title !== s.title
          );
        });

      if (!sessionsChanged && nextActive === state.activeSessionId) {
        return state;
      }

      return {
        sessions: { ...state.sessions, ...sessions },
        activeSessionId: nextActive,
      };
    });
  },

  bootstrap: async () => {
    beginLoading(set);
    try {
      const [projects, sessions] = await Promise.all([
        listProjects().catch(() => [] as Project[]),
        listSessions().catch(() => [] as FridaySession[]),
      ]);
      get().hydrate(sessions);
      set((state) => ({
        projects,
        selectedProjectId:
          state.selectedProjectId &&
          projects.some((p) => p.id === state.selectedProjectId)
            ? state.selectedProjectId
            : null,
      }));
    } finally {
      endLoading(set);
    }
  },

  refreshSessions: async () => {
    beginLoading(set);
    try {
      const list = await listSessions().catch(() => [] as FridaySession[]);
      get().hydrate(list);
    } finally {
      endLoading(set);
    }
  },

  refreshProjects: async () => {
    beginLoading(set);
    try {
      const projects = await listProjects().catch(() => [] as Project[]);
      set((state) => {
        const nextSelected =
          state.selectedProjectId &&
          projects.some((p) => p.id === state.selectedProjectId)
            ? state.selectedProjectId
            : null;
        const unchanged =
          state.selectedProjectId === nextSelected &&
          state.projects.length === projects.length &&
          state.projects.every(
            (p, i) =>
              p.id === projects[i]?.id &&
              p.name === projects[i]?.name &&
              p.path === projects[i]?.path,
          );
        if (unchanged) return state;
        return {
          projects,
          selectedProjectId: nextSelected,
        };
      });
    } finally {
      endLoading(set);
    }
  },

  selectActiveSession: async (id) => {
    await selectActiveSessionIpc(id);
    set({ activeSessionId: id });
  },

  clearTimeline: (sessionId) =>
    set((state) => ({
      timelines: { ...state.timelines, [sessionId]: [] },
    })),

  handleEvent: (event) => {
    const sessionId = event.sessionId;
    const updates: Partial<SessionState> = {};
    const state = get();

    const item = eventToTimeline(event);
    if (item) {
      const timeline = state.timelines[sessionId] ?? [];
      const last = timeline[timeline.length - 1];
      const duplicate =
        last &&
        last.kind === item.kind &&
        (item.kind === "status"
          ? last.kind === "status" &&
            last.status === item.status &&
            last.message === item.message
          : last.timestamp === item.timestamp &&
            (item.kind !== "message" ||
              (last.kind === "message" && last.content === item.content)));
      if (!duplicate) {
        const nextTimeline = [...timeline, item].slice(-MAX_TIMELINE_ITEMS);
        updates.timelines = {
          ...state.timelines,
          [sessionId]: nextTimeline,
        };
      }
    }

    switch (event.type) {
      case "session.discovered":
      case "session.started":
        if (state.activeSessionId !== sessionId) {
          updates.activeSessionId = sessionId;
        }
        break;
      case "agent.status": {
        const session = state.sessions[sessionId];
        const messageChanged =
          state.statusMessages[sessionId] !== event.message;
        const statusChanged = session && session.status !== event.status;

        if (session && statusChanged) {
          updates.sessions = {
            ...state.sessions,
            [sessionId]: { ...session, status: event.status },
          };
        }
        if (messageChanged) {
          updates.statusMessages = {
            ...state.statusMessages,
            [sessionId]: event.message,
          };
        }
        break;
      }
      case "approval.required": {
        const existing = state.pendingApprovals[sessionId];
        const nextApproval = {
          approvalId: event.approvalId,
          command: event.command,
          risk: event.risk,
          title: event.title,
        };
        if (
          existing &&
          existing.approvalId === nextApproval.approvalId &&
          existing.command === nextApproval.command &&
          existing.risk === nextApproval.risk &&
          existing.title === nextApproval.title
        ) {
          break;
        }
        updates.pendingApprovals = {
          ...state.pendingApprovals,
          [sessionId]: nextApproval,
        };
        break;
      }
      case "session.completed": {
        const session = state.sessions[sessionId];
        if (session) {
          updates.sessions = {
            ...state.sessions,
            [sessionId]: {
              ...session,
              status: "done" as FridaySessionStatus,
              summary: event.summary ?? session.summary,
            },
          };
        }
        if (state.pendingApprovals[sessionId]) {
          const nextApprovals = { ...state.pendingApprovals };
          delete nextApprovals[sessionId];
          updates.pendingApprovals = nextApprovals;
        }
        break;
      }
      case "session.error": {
        const session = state.sessions[sessionId];
        const timelineKey = session
          ? sessionId
          : (state.activeSessionId ?? sessionId);
        if (session) {
          updates.sessions = {
            ...state.sessions,
            [sessionId]: { ...session, status: "error" as FridaySessionStatus },
          };
        } else if (state.activeSessionId && state.sessions[state.activeSessionId]) {
          const active = state.sessions[state.activeSessionId];
          updates.sessions = {
            ...state.sessions,
            [state.activeSessionId]: {
              ...active,
              status: "error" as FridaySessionStatus,
            },
          };
        }
        updates.statusMessages = {
          ...state.statusMessages,
          [timelineKey]: event.error,
        };
        break;
      }
      default:
        break;
    }

    if (Object.keys(updates).length > 0) {
      set(updates);
    }
  },
}));

export function useActiveSession(): FridaySession | null {
  return useSessionStore((s) =>
    s.activeSessionId ? s.sessions[s.activeSessionId] ?? null : null,
  );
}

export function useActiveTimeline(): TimelineItem[] {
  return useSessionStore((s) => {
    if (s.activeSessionId) {
      return s.timelines[s.activeSessionId] ?? EMPTY_TIMELINE;
    }
    return s.timelines[SURFACE_SESSION_ID] ?? EMPTY_TIMELINE;
  });
}

export function useActivePendingApproval(): PendingApproval | undefined {
  return useSessionStore((s) =>
    s.activeSessionId
      ? s.pendingApprovals[s.activeSessionId]
      : undefined,
  );
}

export function useActiveStatusMessage(): string | undefined {
  return useSessionStore((s) => {
    if (s.activeSessionId) {
      return s.statusMessages[s.activeSessionId];
    }
    return s.statusMessages[SURFACE_SESSION_ID];
  });
}

export function useSessionList(): FridaySession[] {
  return useSessionStore(useShallow((s) => Object.values(s.sessions)));
}
