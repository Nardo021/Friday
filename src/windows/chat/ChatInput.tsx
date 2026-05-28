import { useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { startAgentSession, stopAgentSession } from "@/lib/tauri";
import { useAgentStore } from "@/state/useAgentStore";
import { useSessionStore } from "@/state/useSessionStore";
import { isRunningStatus } from "@/agent/status";

export function ChatInput() {
  const [prompt, setPrompt] = useState("");
  const [busy, setBusy] = useState(false);
  const selectedProjectId = useSessionStore((s) => s.selectedProjectId);
  const session = useAgentStore((s) => s.currentSession);
  const status = useAgentStore((s) => s.status);
  const setSession = useAgentStore((s) => s.setSession);
  const clearTimeline = useAgentStore((s) => s.clearTimeline);

  const running = isRunningStatus(status);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!prompt.trim() || !selectedProjectId || busy) return;
    setBusy(true);
    try {
      clearTimeline();
      const newSession = await startAgentSession(
        selectedProjectId,
        prompt.trim(),
      );
      setSession(newSession);
      setPrompt("");
    } finally {
      setBusy(false);
    }
  };

  const handleStop = async () => {
    if (!session) return;
    await stopAgentSession(session.id);
  };

  return (
    <div className="border-t border-zinc-800 p-4">
      <form onSubmit={handleSubmit} className="flex gap-2">
        <Input
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          placeholder="Ask Friday..."
          disabled={running || busy || !selectedProjectId}
        />
        <Button type="submit" disabled={running || busy || !selectedProjectId}>
          Send
        </Button>
        {session && running && (
          <Button type="button" variant="destructive" onClick={handleStop}>
            Stop
          </Button>
        )}
      </form>
    </div>
  );
}
