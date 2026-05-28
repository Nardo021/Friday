import { create } from "zustand";

import type { AdapterInfo, FridaySettings } from "@friday/agent-core";
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
    usePty: true,
    defaultMode: "headless",
    defaultOutputFormat: "stream-json",
    argTemplates: {
      headlessStream: ["--print", "--output-format", "stream-json"],
    },
    terminalCols: 120,
    terminalRows: 30,
  },
  onboarding: {
    completed: false,
  },
  pet: {
    patrolEnabled: true,
  },
  voice: {
    pushToTalk: false,
    confirmBeforeSend: true,
    autoSendAfterTranscription: false,
    transcriptionLanguage: "en",
  },
  shortcuts: {
    quickBubble: "CommandOrControl+Space",
    openPanel: "CommandOrControl+Shift+F",
    voiceInput: "CommandOrControl+Shift+V",
    stopSession: "CommandOrControl+Period",
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
