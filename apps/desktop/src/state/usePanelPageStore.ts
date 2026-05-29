import { create } from "zustand";

import type { PanelPageId } from "@/windows/panel/PanelNav";

interface PanelPageState {
  page: PanelPageId;
  setPage: (page: PanelPageId) => void;
}

export const usePanelPageStore = create<PanelPageState>((set) => ({
  page: "agent",
  setPage: (page) => set({ page }),
}));
