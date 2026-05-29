import { useState } from "react";

import type { AgentMode, AgentSessionType } from "@friday/agent-core";
import { isRunningStatus } from "@friday/agent-core";

import { Send } from "lucide-react";

import { Button } from "@/components/ui/button";
import { FieldGroup } from "@/components/ui/field";
import { invokeErrorMessage } from "@/lib/invokeError";
import { createSession } from "@/lib/tauri";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import { UX } from "@/lib/ux";
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
  const [showRepo, setShowRepo] = useState(false);

  const projects = useSessionStore((s) => s.projects);
  const selectedProjectId = useSessionStore((s) => s.selectedProjectId);
  const activeSession = useActiveSession();
  const selectActiveSession = useSessionStore((s) => s.selectActiveSession);
  const clearTimeline = useSessionStore((s) => s.clearTimeline);
  const refreshSessions = useSessionStore((s) => s.refreshSessions);

  const running = activeSession ? isRunningStatus(activeSession.status) : false;
  const canSend = !!prompt.trim() && !running && !busy;

  const handleSend = async () => {
    if (!canSend) return;
    setBusy(true);
    try {
      const session = await createSession({
        projectId: selectedProjectId ?? "",
        prompt: prompt.trim(),
        mode,
        type: MODE_TO_SESSION_TYPE[mode],
      });
      clearTimeline(session.id);
      await selectActiveSession(session.id);
      await refreshSessions();
      setPrompt("");
    } catch (e) {
      toast.error(invokeErrorMessage(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section
      aria-label="Chat with agent"
      className="motion-feedback shrink-0 border-t border-border bg-card/30 px-4 py-4"
    >
      <div className={cn("mx-auto flex max-w-2xl flex-col", UX.betweenGroups)}>
        <div className={UX.section}>
          <h2 className="text-sm font-medium">Chat</h2>
          <p className="text-xs text-muted-foreground">
            Type a message and send — no repo required. Link a folder only when
            you want the agent to work in a specific project.
          </p>
        </div>

        <FieldGroup className={UX.withinGroup}>
          <ModeSelector value={mode} onChange={setMode} />
          <PromptInput
            value={prompt}
            onChange={setPrompt}
            disabled={running || busy}
            onSubmit={() => void handleSend()}
          />
          {projects.length > 0 && (
            <div className="flex flex-col gap-2">
              <button
                type="button"
                className="w-fit text-xs text-muted-foreground underline-offset-2 hover:text-foreground hover:underline"
                onClick={() => setShowRepo((v) => !v)}
              >
                {showRepo ? "Hide folder link" : "Link to a repo (optional)"}
              </button>
              {showRepo && <RepoSelector optional />}
            </div>
          )}
        </FieldGroup>

        <Button
          size="lg"
          className="w-full"
          onClick={() => void handleSend()}
          disabled={!canSend}
        >
          <Send data-icon="inline-start" />
          Send
        </Button>
      </div>
    </section>
  );
}
