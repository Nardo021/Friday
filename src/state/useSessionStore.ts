import { create } from "zustand";

import type { AgentSession, Project } from "@/agent/types";
import { listProjects, listSessions } from "@/lib/tauri";

interface SessionState {
  sessions: AgentSession[];
  projects: Project[];
  selectedProjectId: string | null;
  loading: boolean;
  refreshSessions: () => Promise<void>;
  refreshProjects: () => Promise<void>;
  setSelectedProject: (id: string | null) => void;
}

export const useSessionStore = create<SessionState>((set) => ({
  sessions: [],
  projects: [],
  selectedProjectId: null,
  loading: false,
  setSelectedProject: (id) => set({ selectedProjectId: id }),
  refreshSessions: async () => {
    set({ loading: true });
    try {
      const sessions = await listSessions();
      set({ sessions });
    } finally {
      set({ loading: false });
    }
  },
  refreshProjects: async () => {
    set({ loading: true });
    try {
      const projects = await listProjects();
      set((state) => ({
        projects,
        selectedProjectId:
          state.selectedProjectId ?? projects[0]?.id ?? null,
      }));
    } finally {
      set({ loading: false });
    }
  },
}));
