import { useEffect } from "react";

import { AgentBadge } from "@/components/friday/AgentBadge";
import { useAgentEventBridge } from "@/hooks/useAgentEventBridge";
import { listenAgentEvents } from "@/lib/tauri";
import { useActiveSession, useSessionStore } from "@/state/useSessionStore";
import { useSettingsStore } from "@/state/useSettingsStore";

import { ActionsBar } from "./ActionsBar";
import { ActiveSessionsList } from "./ActiveSessionsList";
import { ChatPortal } from "./ChatPortal";
import { CurrentStatusBar } from "./CurrentStatusBar";
import { SessionTimeline } from "./SessionTimeline";

export function FridayPanel() {
  const refreshProjects = useSessionStore((s) => s.refreshProjects);
  const refreshSessions = useSessionStore((s) => s.refreshSessions);
  const loadSettings = useSettingsStore((s) => s.load);
  const session = useActiveSession();

  useAgentEventBridge();

  useEffect(() => {
    refreshProjects();
    refreshSessions();
    loadSettings();
  }, [refreshProjects, refreshSessions, loadSettings]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listenAgentEvents((event) => {
      if (
        event.type === "session.completed" ||
        event.type === "session.error"
      ) {
        refreshSessions();
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, [refreshSessions]);

  return (
    <div className="flex h-screen flex-col bg-zinc-950 text-zinc-100">
      <header className="flex items-center justify-between border-b border-zinc-800 px-4 py-3">
        <h1 className="text-lg font-semibold">Friday Agent Portal</h1>
        {session && <AgentBadge adapterId={session.adapterId} />}
      </header>
      <CurrentStatusBar />
      <div className="flex min-h-0 flex-1">
        <ActiveSessionsList />
        <div className="flex min-w-0 flex-1 flex-col">
          <SessionTimeline />
          <ChatPortal />
          <ActionsBar />
        </div>
      </div>
    </div>
  );
}
