import { useEffect } from "react";

import { StatusPill } from "@/components/friday/StatusPill";
import { Card, CardContent } from "@/components/ui/card";
import { useAgentEventBridge } from "@/hooks/useAgentEventBridge";
import { statusBubbleText } from "@/pet-engine/AgentMoodMapper";
import {
  useActiveSession,
  useActiveStatusMessage,
  useSessionStore,
} from "@/state/useSessionStore";

export function StatusBubbleWindow() {
  const session = useActiveSession();
  const message = useActiveStatusMessage();
  const refreshSessions = useSessionStore((s) => s.refreshSessions);

  useAgentEventBridge();

  useEffect(() => {
    document.body.classList.add("bg-transparent");
    void refreshSessions();
    return () => document.body.classList.remove("bg-transparent");
  }, [refreshSessions]);

  const text = session
    ? statusBubbleText(session.status, message ?? session.summary ?? session.prompt)
    : message ?? "";

  if (!text) return null;

  return (
    <div className="flex h-screen w-screen items-center justify-center bg-transparent p-2">
      <Card className="motion-popover-in w-full bg-popover/95 py-0 shadow-xl backdrop-blur">
        <CardContent className="px-3 py-2 text-xs">
          {session && (
            <div className="mb-1">
              <StatusPill status={session.status} />
            </div>
          )}
          <p className="truncate text-foreground">{text}</p>
        </CardContent>
      </Card>
    </div>
  );
}
