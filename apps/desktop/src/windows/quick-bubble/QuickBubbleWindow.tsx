import { useEffect, useMemo, useState } from "react";

import type { AgentMode } from "@friday/agent-core";
import { isRunningStatus } from "@friday/agent-core";

import { ApprovalCard } from "@/components/friday/ApprovalCard";
import { StatusPill } from "@/components/friday/StatusPill";
import { VoiceRecorder } from "@/components/friday/VoiceRecorder";
import { Button } from "@/components/ui/button";
import { useAgentEventBridge } from "@/hooks/useAgentEventBridge";
import {
  closeSession,
  executeQuickIntent,
  hideQuickBubble,
  openPanel,
  submitQuickInput,
  type QuickIntentKind,
} from "@/lib/tauri";
import { ModeSelector } from "@/windows/panel/ModeSelector";
import { RepoSelector } from "@/windows/panel/RepoSelector";
import {
  useActivePendingApproval,
  useActiveSession,
  useActiveStatusMessage,
  useActiveTimeline,
  useSessionList,
  useSessionStore,
} from "@/state/useSessionStore";

const SUGGESTIONS = ["Status?", "Stop", "记一下…", "Open panel"];

function BubbleHeader({ onClose }: { onClose: () => void }) {
  return (
    <div className="flex shrink-0 items-center justify-between border-b border-zinc-800 px-2.5 py-1.5">
      <span className="text-xs font-semibold text-zinc-200">Friday</span>
      <div className="flex items-center gap-0.5">
        <button
          type="button"
          aria-label="Hide bubble"
          className="flex h-6 w-6 items-center justify-center rounded text-zinc-400 hover:bg-zinc-800"
          onClick={onClose}
        >
          ×
        </button>
      </div>
    </div>
  );
}

function SessionPicker() {
  const sessions = useSessionList();
  const activeSessionId = useSessionStore((s) => s.activeSessionId);
  const selectActiveSession = useSessionStore((s) => s.selectActiveSession);

  return (
    <select
      className="w-full rounded border border-zinc-700 bg-zinc-900 px-2 py-1 text-[11px] text-zinc-200"
      value={activeSessionId ?? ""}
      onChange={(e) => void selectActiveSession(e.target.value || null)}
    >
      <option value="">No active session</option>
      {sessions.map((s) => (
        <option key={s.id} value={s.id}>
          {s.title} ({s.status})
        </option>
      ))}
    </select>
  );
}

export function QuickBubbleWindow() {
  useAgentEventBridge();

  const session = useActiveSession();
  const message = useActiveStatusMessage();
  const timeline = useActiveTimeline();
  const pendingApproval = useActivePendingApproval();
  const projects = useSessionStore((s) => s.projects);
  const selectedProjectId = useSessionStore((s) => s.selectedProjectId);
  const refreshProjects = useSessionStore((s) => s.refreshProjects);
  const refreshSessions = useSessionStore((s) => s.refreshSessions);

  const [draft, setDraft] = useState("");
  const [busy, setBusy] = useState(false);
  const [feedback, setFeedback] = useState<string | null>(null);
  const [clarify, setClarify] = useState<QuickIntentKind | null>(null);
  const [mode, setMode] = useState<AgentMode>("local_cli");
  const [listening, setListening] = useState(false);

  useEffect(() => {
    document.body.classList.add("bg-transparent");
    void refreshProjects();
    void refreshSessions();
    return () => document.body.classList.remove("bg-transparent");
  }, [refreshProjects, refreshSessions]);

  const project =
    projects.find((p) => p.id === selectedProjectId) ?? projects[0];
  const running = session ? isRunningStatus(session.status) : false;

  const recentLines = useMemo(() => {
    return timeline
      .filter((item) => item.kind === "message" || item.kind === "status")
      .slice(-6)
      .map((item) => {
        if (item.kind === "message") {
          return {
            key: `${item.timestamp}-${item.role}`,
            role: item.role,
            text: item.content,
          };
        }
        return {
          key: item.timestamp,
          role: "status" as const,
          text: item.message ?? item.status,
        };
      });
  }, [timeline]);

  const handleClose = () => void hideQuickBubble();

  const dispatchResult = async (result: Awaited<ReturnType<typeof submitQuickInput>>) => {
    if (result.message) setFeedback(result.message);
    if (result.route.intent.kind === "clarify") {
      setClarify(result.route.intent);
      return;
    }
    setClarify(null);
    if (result.sessionId) {
      await useSessionStore.getState().selectActiveSession(result.sessionId);
    }
    await refreshSessions();
    if (result.executed) setDraft("");
  };

  const handleSend = async () => {
    const text = draft.trim();
    if (!text || busy) return;
    if (!project && !session) {
      void openPanel();
      return;
    }

    setBusy(true);
    setFeedback(null);
    try {
      const result = await submitQuickInput({
        text,
        sessionId: session?.id ?? null,
        projectId: project?.id ?? null,
        mode,
      });
      await dispatchResult(result);
    } catch (e) {
      setFeedback(e instanceof Error ? e.message : "Send failed");
    } finally {
      setBusy(false);
    }
  };

  const runClarify = async (intent: QuickIntentKind) => {
    setBusy(true);
    try {
      const result = await executeQuickIntent(intent);
      await dispatchResult(result);
    } finally {
      setBusy(false);
    }
  };

  const onVoiceTranscript = (text: string) => {
    if (text.trim()) setDraft(text);
  };

  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden rounded-xl border border-zinc-700/90 bg-zinc-950/95 text-zinc-100 shadow-2xl backdrop-blur-md">
      <BubbleHeader onClose={handleClose} />

      <div className="flex min-h-0 flex-1 flex-col gap-2 p-2.5">
        {listening && (
          <p className="text-[11px] text-indigo-300">Friday is listening…</p>
        )}

        {session ? (
          <div className="flex shrink-0 items-center justify-between gap-2">
            <p className="truncate text-xs font-medium text-zinc-300">
              {session.title}
            </p>
            <StatusPill status={session.status} />
          </div>
        ) : (
          <p className="shrink-0 text-xs text-zinc-500">
            {project ? "Ask Friday anything" : "Add a project to start"}
          </p>
        )}

        <SessionPicker />
        <div className="grid grid-cols-2 gap-2">
          <RepoSelector />
        </div>
        <ModeSelector value={mode} onChange={setMode} />

        <div className="min-h-0 flex-1 space-y-1 overflow-y-auto rounded-md border border-zinc-800/80 bg-zinc-900/50 p-2">
          {feedback && (
            <p className="whitespace-pre-wrap text-xs text-indigo-200">{feedback}</p>
          )}
          {recentLines.length === 0 && !feedback && (
            <p className="text-xs text-zinc-500">
              {message ?? "Send a prompt or follow-up below."}
            </p>
          )}
          {recentLines.map((line) => (
            <div key={line.key} className="text-xs leading-relaxed">
              <span className="font-medium text-zinc-500">
                {line.role === "status" ? "·" : `${line.role}: `}
              </span>
              <span className="text-zinc-300">{line.text}</span>
            </div>
          ))}
        </div>

        {clarify?.kind === "clarify" && (
          <div className="shrink-0 space-y-1 rounded border border-amber-900/50 bg-amber-950/30 p-2">
            <p className="text-xs text-amber-100">{clarify.message}</p>
            <div className="flex flex-wrap gap-1">
              <Button
                size="sm"
                className="h-6 px-2 text-[10px]"
                onClick={() =>
                  void runClarify({
                    kind: "followUp",
                    sessionId: session!.id,
                    text: draft,
                  })
                }
                disabled={!session}
              >
                Follow-up
              </Button>
              <Button
                size="sm"
                className="h-6 px-2 text-[10px]"
                disabled={!project}
                onClick={() =>
                  void runClarify({
                    kind: "newTask",
                    projectId: project!.id,
                    mode,
                    prompt: draft,
                  })
                }
              >
                New task
              </Button>
              <Button
                size="sm"
                variant="secondary"
                className="h-6 px-2 text-[10px]"
                onClick={() =>
                  void runClarify({
                    kind: "saveIdea",
                    title: draft.slice(0, 48),
                    body: draft,
                    projectId: project?.id,
                    sessionId: session?.id,
                  })
                }
              >
                Save idea
              </Button>
            </div>
          </div>
        )}

        {pendingApproval && (
          <div className="shrink-0 scale-90 origin-top">
            <ApprovalCard
              approvalId={pendingApproval.approvalId}
              command={pendingApproval.command}
              risk={pendingApproval.risk}
            />
          </div>
        )}

        <div className="flex shrink-0 gap-1.5">
          <textarea
            className="min-h-[52px] flex-1 resize-none rounded-md border border-zinc-700 bg-zinc-900 px-2 py-1.5 text-xs text-zinc-100 placeholder:text-zinc-500 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-indigo-500"
            placeholder={running ? "Follow up…" : "Describe a task…"}
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            disabled={busy}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                void handleSend();
              }
            }}
          />
          <div className="flex flex-col justify-end gap-1">
            <VoiceRecorder
              disabled={busy}
              onTranscript={onVoiceTranscript}
              onListeningChange={setListening}
            />
            <Button
              size="sm"
              className="h-7 px-2 text-xs"
              disabled={busy || !draft.trim()}
              onClick={() => void handleSend()}
            >
              Send
            </Button>
          </div>
        </div>

        <div className="flex shrink-0 flex-wrap gap-1">
          {SUGGESTIONS.map((s) => (
            <button
              key={s}
              type="button"
              className="rounded-full border border-zinc-700 px-2 py-0.5 text-[10px] text-zinc-400 hover:border-zinc-500"
              onClick={() => setDraft(s)}
            >
              {s}
            </button>
          ))}
          <Button
            size="sm"
            variant="secondary"
            className="h-6 px-2 text-[10px]"
            onClick={() => void openPanel()}
          >
            Panel
          </Button>
          {session && running && (
            <Button
              size="sm"
              variant="destructive"
              className="h-6 px-2 text-[10px]"
              onClick={() => void closeSession(session.id)}
            >
              Stop
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
