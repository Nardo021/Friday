import { useState } from "react";

import type { AgentMode, AgentSessionType } from "@friday/agent-core";
import { isRunningStatus } from "@friday/agent-core";

import { Button } from "@/components/ui/button";
import { createSession } from "@/lib/tauri";
import {
  useActiveSession,
  useSessionStore,
} from "@/state/useSessionStore";

import { ModeSelector } from "./ModeSelector";
import { PromptInput } from "./PromptInput";
import { RepoSelector } from "./RepoSelector";

const MODE_TO_SESSION_TYPE: Record<AgentMode, AgentSessionType> = {
  local_cli: "friday_owned_cli",
  sdk_local: "cursor_sdk_local",
  cloud_agent: "cursor_cloud",
};

export function ChatPortal() {
  const [prompt, setPrompt] = useState("");
  const [mode, setMode] = useState<AgentMode>("local_cli");
  const [busy, setBusy] = useState(false);

  const selectedProjectId = useSessionStore((s) => s.selectedProjectId);
  const activeSession = useActiveSession();
  const selectActiveSession = useSessionStore((s) => s.selectActiveSession);
  const clearTimeline = useSessionStore((s) => s.clearTimeline);
  const refreshSessions = useSessionStore((s) => s.refreshSessions);

  const running = activeSession ? isRunningStatus(activeSession.status) : false;

  const handleStart = async () => {
    if (!prompt.trim() || !selectedProjectId || busy) return;
    setBusy(true);
    try {
      const session = await createSession({
        projectId: selectedProjectId,
        prompt: prompt.trim(),
        mode,
        type: MODE_TO_SESSION_TYPE[mode],
      });
      clearTimeline(session.id);
      await selectActiveSession(session.id);
      await refreshSessions();
      setPrompt("");
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-4 border-t border-zinc-800 p-4">
      <h3 className="text-sm font-medium text-zinc-300">New task</h3>
      <RepoSelector />
      <ModeSelector value={mode} onChange={setMode} />
      <PromptInput
        value={prompt}
        onChange={setPrompt}
        disabled={running || busy || !selectedProjectId}
      />
      <Button
        onClick={() => void handleStart()}
        disabled={running || busy || !selectedProjectId || !prompt.trim()}
      >
        Start
      </Button>
    </div>
  );
}
