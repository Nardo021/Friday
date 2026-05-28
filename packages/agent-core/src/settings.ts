export interface FridaySettings {
  appearance: {
    theme: "system" | "light" | "dark";
    accentColor: string;
    petScale: number;
    reducedMotion: boolean;
  };
  behavior: {
    launchAtStartup: boolean;
    alwaysOnTop: boolean;
    showBubbleOnStatusChange: boolean;
    autoCollapseBubble: boolean;
    soundEffects: boolean;
  };
  security: {
    requireApprovalForHighRiskCommands: boolean;
    requireApprovalForMediumRiskCommands: boolean;
    redactSecrets: boolean;
    allowShellCommands: boolean;
  };
  cursor: {
    executablePath?: string;
    /** Set by backend; never send the key from the client. */
    apiKeyConfigured?: boolean;
    usePty: boolean;
    defaultMode: "interactive" | "headless";
    defaultOutputFormat: "text" | "json" | "stream-json";
    argTemplates: {
      headlessStream: string[];
    };
    terminalCols: number;
    terminalRows: number;
  };
  onboarding: {
    completed: boolean;
  };
  pet: {
    lastX?: number;
    lastY?: number;
    patrolEnabled: boolean;
  };
  voice: {
    pushToTalk: boolean;
    confirmBeforeSend: boolean;
    autoSendAfterTranscription: boolean;
    transcriptionLanguage: string;
    sttApiKeyConfigured?: boolean;
  };
  shortcuts: {
    quickBubble: string;
    openPanel: string;
    voiceInput: string;
    stopSession: string;
  };
  cloud?: {
    autoCreatePr: boolean;
    model?: string;
  };
  mobileBridge?: {
    enabled: boolean;
    port: number;
    authToken: string;
  };
}

export interface MobileBridgeSettingsView {
  enabled: boolean;
  port: number;
  authToken: string;
  localUrl: string;
}
