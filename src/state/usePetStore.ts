import { create } from "zustand";

import type { AgentStatus } from "@/agent/types";
import { STATUS_TO_MOOD, type PetMood } from "@/agent/mood-map";

interface PetState {
  x: number;
  y: number;
  mood: PetMood;
  bubbleVisible: boolean;
  bubbleText: string;
  setPosition: (x: number, y: number) => void;
  setFromStatus: (status: AgentStatus, message?: string) => void;
  showBubble: (text: string) => void;
  hideBubble: () => void;
}

export const usePetStore = create<PetState>((set) => ({
  x: 100,
  y: 100,
  mood: "calm",
  bubbleVisible: false,
  bubbleText: "",
  setPosition: (x, y) => set({ x, y }),
  setFromStatus: (status, message) =>
    set({
      mood: STATUS_TO_MOOD[status],
      bubbleText: message ?? "",
      bubbleVisible: Boolean(message),
    }),
  showBubble: (text) => set({ bubbleVisible: true, bubbleText: text }),
  hideBubble: () => set({ bubbleVisible: false }),
}));
