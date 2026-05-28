import { create } from "zustand";

import type { AdapterInfo, FridaySettings } from "@/agent/types";
import { getSettings, listAdapters, saveSettings } from "@/lib/tauri";

const defaultSettings: FridaySettings = {
  appearance: {
    theme: "system",
    accentColor: "#6366f1",
    petScale: 1,
    reducedMotion: false,
  },
  behavior: {
    launchAtStartup: false,
    alwaysOnTop: false,
    showBubbleOnStatusChange: true,
    autoCollapseBubble: true,
    soundEffects: false,
  },
  security: {
    requireApprovalForHighRiskCommands: true,
    requireApprovalForMediumRiskCommands: false,
    redactSecrets: true,
    allowShellCommands: true,
  },
  cursor: {
    defaultMode: "headless",
    defaultOutputFormat: "stream-json",
  },
};

interface SettingsState {
  settings: FridaySettings;
  adapters: AdapterInfo[];
  loaded: boolean;
  load: () => Promise<void>;
  update: (settings: FridaySettings) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: defaultSettings,
  adapters: [],
  loaded: false,
  load: async () => {
    const [settings, adapters] = await Promise.all([
      getSettings(),
      listAdapters(),
    ]);
    set({ settings, adapters, loaded: true });
  },
  update: async (settings) => {
    await saveSettings(settings);
    set({ settings });
  },
}));
