import { useEffect, useMemo, useState } from "react";
import { ListPlus, MessageSquare, PanelTop, Reply, X } from "lucide-react";

import type { AgentMode, FridaySessionStatus } from "@friday/agent-core";
import { isRunningStatus } from "@friday/agent-core";

import { ApprovalCard } from "@/components/friday/ApprovalCard";
import { StatusPill } from "@/components/friday/StatusPill";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { useAgentEventBridge } from "@/hooks/useAgentEventBridge";
import { useFridayReady } from "@/hooks/useFridayReady";
import { invokeErrorMessage } from "@/lib/invokeError";
import {
  closeSession,
  executeQuickIntent,
  hideQuickBubble,
  openPanel,
  submitQuickInput,
  type QuickIntentKind,
} from "@/lib/tauri";
import {
  useActivePendingApproval,
  useActiveSession,
  useActiveTimeline,
  useSessionStore,
} from "@/state/useSessionStore";
import { useSettingsStore } from "@/state/useSettingsStore";
import { toast } from "sonner";

import { QuickComposer } from "./QuickComposer";

type BubbleNotice =
  | { kind: "error"; message: string }
  | { kind: "info"; message: string };

function BubbleHeader({
  sessionTitle,
  sessionStatus,
  onClose,
  onOpenPanel,
}: {
  sessionTitle?: string;
  sessionStatus?: FridaySessionStatus;
  onClose: () => void;
  onOpenPanel: () => void;
}) {
  return (
    <div className="flex shrink-0 items-center justify-between gap-2 border-b border-border/80 px-3 py-2">
      <div className="min-w-0 flex-1">
        {sessionTitle ? (
          <div className="flex items-center gap-2">
            <p className="truncate text-xs font-medium text-foreground">
              {sessionTitle}
            </p>
            {sessionStatus && <StatusPill status={sessionStatus} />}
          </div>
        ) : (
          <span className="text-xs font-semibold">Friday</span>
        )}
      </div>
      <div className="flex shrink-0 items-center gap-0.5">
        <Button
          type="button"
          variant="ghost"
          size="icon-sm"
          aria-label="Open panel"
          onClick={onOpenPanel}
        >
          <PanelTop />
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="icon-sm"
          aria-label="Hide quick chat"
          onClick={onClose}
        >
          <X />
        </Button>
      </div>
    </div>
  );
}

export function QuickBubbleWindow() {
  const ready = useFridayReady();
  useAgentEventBridge();

  const session = useActiveSession();
  const timeline = useActiveTimeline();
  const pendingApproval = useActivePendingApproval();

  const [draft, setDraft] = useState("");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<BubbleNotice | null>(null);
  const [clarify, setClarify] = useState<QuickIntentKind | null>(null);
  const apiKeyConfigured = useSettingsStore(
    (s) => s.settings.cursor.apiKeyConfigured ?? false,
  );
  const [mode, setMode] = useState<AgentMode>(
    apiKeyConfigured ? "cloud_agent" : "local_cli",
  );
  const [listening, setListening] = useState(false);

  useEffect(() => {
    document.body.classList.add("bg-transparent");
    return () => document.body.classList.remove("bg-transparent");
  }, []);

  useEffect(() => {
    if (!ready) return;
    void useSessionStore.getState().bootstrap();
    void useSettingsStore.getState().load();
  }, [ready]);

  useEffect(() => {
    if (apiKeyConfigured) {
      setMode("cloud_agent");
    }
  }, [apiKeyConfigured]);

  const running = session ? isRunningStatus(session.status) : false;

  const recentLines = useMemo(() => {
    return timeline
      .filter((item) => item.kind === "message" || item.kind === "status")
      .slice(-12)
      .map((item) => {
        if (item.kind === "message") {
          return {
            key: `${item.timestamp}-${item.role}`,
            text:
              item.role === "user"
                ? item.content
                : `${item.content.slice(0, 400)}${item.content.length > 400 ? "…" : ""}`,
            isUser: item.role === "user",
          };
        }
        return {
          key: item.timestamp,
          text: item.message ?? item.status,
          isUser: false,
        };
      });
  }, [timeline]);

  const hasChatContent =
    recentLines.length > 0 ||
    !!notice ||
    clarify?.kind === "clarify" ||
    !!pendingApproval;

  const dispatchResult = async (
    result: Awaited<ReturnType<typeof submitQuickInput>>,
  ) => {
    if (result.route.intent.kind === "clarify") {
      setClarify(result.route.intent);
      if (result.message) {
        setNotice({ kind: "info", message: result.message });
      }
      return;
    }
    setClarify(null);

    if (result.executed) {
      setNotice(null);
      if (result.message) {
        toast.success(result.message);
      }
      if (result.sessionId) {
        await useSessionStore.getState().selectActiveSession(result.sessionId);
      }
      await useSessionStore.getState().refreshSessions();
      setDraft("");
      return;
    }

    if (result.message) {
      setNotice({ kind: "info", message: result.message });
      toast.message(result.message);
    }
    if (result.sessionId) {
      await useSessionStore.getState().selectActiveSession(result.sessionId);
    }
    await useSessionStore.getState().refreshSessions();
  };

  const handleSend = async () => {
    const text = draft.trim();
    if (!text || busy) return;

    setBusy(true);
    setNotice(null);
    try {
      const result = await submitQuickInput({
        text,
        sessionId: session?.id ?? null,
        projectId: null,
        mode,
      });
      await dispatchResult(result);
    } catch (e) {
      const msg = invokeErrorMessage(e);
      setNotice({ kind: "error", message: msg });
      toast.error(msg);
    } finally {
      setBusy(false);
    }
  };

  const runClarify = async (intent: QuickIntentKind) => {
    setBusy(true);
    setNotice(null);
    try {
      const result = await executeQuickIntent(intent);
      await dispatchResult(result);
    } catch (e) {
      const msg = invokeErrorMessage(e);
      setNotice({ kind: "error", message: msg });
      toast.error(msg);
    } finally {
      setBusy(false);
    }
  };

  return (
    <main className="motion-page-in flex h-screen w-screen flex-col overflow-hidden rounded-xl border border-border/90 bg-background/95 text-foreground shadow-2xl backdrop-blur-md">
      <BubbleHeader
        sessionTitle={session?.title}
        sessionStatus={session?.status}
        onClose={() => void hideQuickBubble()}
        onOpenPanel={() => {
          void hideQuickBubble();
          void openPanel();
        }}
      />

      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <ScrollArea className="min-h-0 flex-1 px-3 pt-2">
          {!hasChatContent && (
            <Empty className="border-0 py-6">
              <EmptyHeader>
                <EmptyMedia variant="icon">
                  <MessageSquare />
                </EmptyMedia>
                <EmptyTitle className="text-xs font-normal text-muted-foreground">
                  No messages yet
                </EmptyTitle>
                <EmptyDescription className="text-xs">
                  Send a prompt below — Friday routes it to your agent.
                </EmptyDescription>
              </EmptyHeader>
            </Empty>
          )}

          {notice?.kind === "error" && (
            <Alert variant="destructive" className="mb-2 py-2">
              <AlertTitle className="text-xs font-medium">Could not send</AlertTitle>
              <AlertDescription className="text-xs leading-relaxed">
                {notice.message}
              </AlertDescription>
            </Alert>
          )}
          {notice?.kind === "info" && (
            <p className="mb-2 text-xs leading-relaxed text-muted-foreground">
              {notice.message}
            </p>
          )}

          <div className="motion-stagger flex flex-col gap-2">
            {recentLines.map((line) => (
              <p
                key={line.key}
                className={
                  line.isUser
                    ? "text-xs text-foreground"
                    : "text-xs leading-relaxed text-muted-foreground"
                }
              >
                {line.text}
              </p>
            ))}
          </div>

          {clarify?.kind === "clarify" && (
            <Alert className="mt-2 shrink-0 py-2">
              <AlertDescription className="text-xs">{clarify.message}</AlertDescription>
              <div className="mt-1.5 flex flex-wrap gap-x-2 gap-y-0.5 text-[11px]">
                {session && (
                  <Button
                    type="button"
                    variant="link"
                    size="xs"
                    className="h-auto p-0"
                    disabled={busy}
                    onClick={() =>
                      void runClarify({
                        kind: "followUp",
                        sessionId: session.id,
                        text: draft,
                      })
                    }
                  >
                    <Reply data-icon="inline-start" />
                    Follow-up
                  </Button>
                )}
                <Button
                  type="button"
                  variant="link"
                  size="xs"
                  className="h-auto p-0"
                  disabled={busy}
                  onClick={() =>
                    void runClarify({
                      kind: "newTask",
                      projectId: "",
                      mode,
                      prompt: draft,
                    })
                  }
                >
                  <ListPlus data-icon="inline-start" />
                  New task
                </Button>
              </div>
            </Alert>
          )}

          {pendingApproval && (
            <div className="mt-2 shrink-0 origin-top scale-[0.92]">
              <ApprovalCard
                approvalId={pendingApproval.approvalId}
                command={pendingApproval.command}
                risk={pendingApproval.risk}
              />
            </div>
          )}
        </ScrollArea>

        <Separator />
        <div className="shrink-0 px-3 py-2.5">
          <QuickComposer
            draft={draft}
            onDraftChange={setDraft}
            mode={mode}
            onModeChange={setMode}
            busy={busy}
            ready={ready}
            running={running}
            listening={listening}
            onListeningChange={setListening}
            onVoiceTranscript={(text) => text.trim() && setDraft(text)}
            onSend={() => void handleSend()}
            showStop={!!session && running}
            onStop={
              session ? () => void closeSession(session.id) : undefined
            }
          />
        </div>
      </div>
    </main>
  );
}
