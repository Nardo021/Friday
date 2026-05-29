import { useState } from "react";

import {
  CURSOR_CLOUD_CAPABILITIES,
  EXTERNAL_CURSOR_OBSERVER_CAPABILITIES,
  FRIDAY_OWNED_CLI_CAPABILITIES,
  isRunningStatus,
  type AgentCapabilities,
  type FridaySession,
} from "@friday/agent-core";

import { ScrollText, Square } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Field, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { closeSession, followUp, openPanel } from "@/lib/tauri";
import { usePanelPageStore } from "@/state/usePanelPageStore";
import {
  useActivePendingApproval,
  useActiveSession,
} from "@/state/useSessionStore";

function capabilitiesFor(session: FridaySession): AgentCapabilities {
  switch (session.type) {
    case "external_cli":
      return EXTERNAL_CURSOR_OBSERVER_CAPABILITIES;
    case "cursor_cloud":
      return CURSOR_CLOUD_CAPABILITIES;
    case "friday_owned_cli":
    case "cursor_sdk_local":
    default:
      return FRIDAY_OWNED_CLI_CAPABILITIES;
  }
}

export function ActionsBar() {
  const session = useActiveSession();
  const pendingApproval = useActivePendingApproval();
  const [followUpText, setFollowUpText] = useState("");
  const [busy, setBusy] = useState(false);

  if (!session) return null;

  const caps = capabilitiesFor(session);
  const running = isRunningStatus(session.status);

  const handleFollowUp = async () => {
    if (!followUpText.trim() || busy) return;
    setBusy(true);
    try {
      await followUp(session.id, followUpText.trim());
      setFollowUpText("");
    } finally {
      setBusy(false);
    }
  };

  const handleStop = async () => {
    await closeSession(session.id);
  };

  return (
    <div className="border-t border-border p-4">
      {pendingApproval && (
        <p className="mb-2 text-xs text-destructive">
          Pending approval: {pendingApproval.title ?? pendingApproval.command}
        </p>
      )}
      {caps.canSendFollowUp && (
        <div className="mb-3 flex items-end gap-2">
          <Field className="min-w-0 flex-1">
            <FieldLabel htmlFor="panel-follow-up">Follow up</FieldLabel>
            <Input
              id="panel-follow-up"
              value={followUpText}
              onChange={(e) => setFollowUpText(e.target.value)}
              disabled={!running || busy}
            />
          </Field>
          <Button
            size="sm"
            onClick={() => void handleFollowUp()}
            disabled={!running || busy || !followUpText.trim()}
          >
            Follow up
          </Button>
        </div>
      )}
      <div className="flex flex-wrap gap-2">
        {caps.canStop && running && (
          <Button size="sm" variant="destructive" onClick={() => void handleStop()}>
            <Square data-icon="inline-start" className="fill-current" />
            Stop
          </Button>
        )}
        {caps.canObserve && (
          <Button
            size="sm"
            variant="secondary"
            onClick={() => {
              usePanelPageStore.getState().setPage("logs");
              void openPanel();
            }}
          >
            <ScrollText data-icon="inline-start" />
            Logs
          </Button>
        )}
      </div>
    </div>
  );
}
