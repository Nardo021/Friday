import { create } from "zustand";

import type { PetMood } from "@friday/agent-core";

interface PetState {
  mood: PetMood;
  setMood: (mood: PetMood) => void;
}

export const usePetStore = create<PetState>((set, get) => ({
  mood: "calm",
  setMood: (mood) => {
    if (get().mood === mood) return;
    set({ mood });
  },
}));
