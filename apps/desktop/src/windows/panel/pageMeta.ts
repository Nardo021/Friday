import type { PanelPageId } from "./PanelNav";

export type PageMeta = {
  title: string;
  description: string;
};

export const PAGE_META: Record<PanelPageId, PageMeta> = {
  agent: {
    title: "Agent",
    description: "Chat with the agent, follow up on runs, and watch live output.",
  },
  dashboard: {
    title: "Dashboard",
    description: "At-a-glance status, recent sessions, and quick links.",
  },
  sessions: {
    title: "All sessions",
    description: "Search, export, or open any past run in the agent view.",
  },
  projects: {
    title: "Projects",
    description: "Optional repo folders when you want the agent scoped to a codebase.",
  },
  ideas: {
    title: "Ideas",
    description: "Quick captures from Quick Chat — turn them into agent tasks.",
  },
  approvals: {
    title: "Approvals",
    description: "Confirm high-risk shell commands before they run.",
  },
  logs: {
    title: "Logs",
    description: "Session log files stored on this device.",
  },
  adapters: {
    title: "Adapters",
    description: "How Friday connects to Cursor CLI, cloud agents, and observers.",
  },
  settings: {
    title: "Settings",
    description: "API keys, pet, shortcuts, and security. Most changes save automatically.",
  },
};

export function panelPageTitle(id: PanelPageId): string {
  return PAGE_META[id].title;
}

export function panelPageDescription(id: PanelPageId): string {
  return PAGE_META[id].description;
}
