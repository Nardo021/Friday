import { create } from "zustand";

import type { AdapterInfo, FridaySettings } from "@friday/agent-core";
import { invokeErrorMessage } from "@/lib/invokeError";
import { getSettings, listAdapters, saveSettings } from "@/lib/tauri";
import { toast } from "sonner";

const defaultSettings: FridaySettings = {
  appearance: {
    theme: "system",
    accentColor: "#c9a227",
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
      headlessStream: [
        "--print",
        "--output-format",
        "{outputFormat}",
        "--stream-partial-output",
        "{prompt}",
      ],
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
    try {
      const [settings, adapters] = await Promise.all([
        getSettings(),
        listAdapters(),
      ]);
      set({ settings, adapters, loaded: true });
    } catch (e) {
      toast.error(invokeErrorMessage(e));
      throw e;
    }
  },
  update: async (settings) => {
    try {
      await saveSettings(settings);
      set({ settings });
    } catch (e) {
      toast.error(invokeErrorMessage(e));
      throw e;
    }
  },
}));
