import { useEffect } from "react";

import { useAgentEventBridge } from "@/hooks/useAgentEventBridge";
import { listenAgentEvents } from "@/lib/tauri";
import { useSessionStore } from "@/state/useSessionStore";
import { useSettingsStore } from "@/state/useSettingsStore";

import { ChatHeader } from "./ChatHeader";
import { ChatInput } from "./ChatInput";
import { ChatTimeline } from "./ChatTimeline";
import { ProjectSelector } from "./ProjectSelector";
import { SessionStatusBar } from "./SessionStatusBar";

export function ChatWindow() {
  const refreshProjects = useSessionStore((s) => s.refreshProjects);
  const refreshSessions = useSessionStore((s) => s.refreshSessions);
  const loadSettings = useSettingsStore((s) => s.load);

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
        event.type === "session_completed" ||
        event.type === "session_error"
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
      <ChatHeader />
      <ProjectSelector />
      <SessionStatusBar />
      <ChatTimeline />
      <ChatInput />
    </div>
  );
}
