import type {
  AgentEvent,
  FridaySession,
  FridaySessionStatus,
  TimelineItem,
} from "@friday/agent-core";

import type { PendingApproval } from "@/state/useSessionStore";
import {
  useActivePendingApproval,
  useActiveSession,
  useActiveStatusMessage,
  useActiveTimeline,
  useSessionStore,
} from "@/state/useSessionStore";

interface DerivedAgentState {
  currentSession: FridaySession | null;
  status: FridaySessionStatus;
  statusMessage?: string;
  timeline: TimelineItem[];
  pendingApproval?: PendingApproval;
  setSession: (session: FridaySession | null) => void;
  clearTimeline: () => void;
  handleEvent: (event: AgentEvent) => void;
}

export function useAgentStore<T>(
  selector: (state: DerivedAgentState) => T,
): T {
  const session = useActiveSession();
  const timeline = useActiveTimeline();
  const pendingApproval = useActivePendingApproval();
  const statusMessage = useActiveStatusMessage();
  const handleEvent = useSessionStore((s) => s.handleEvent);
  const selectActiveSession = useSessionStore((s) => s.selectActiveSession);
  const clearTimelineFn = useSessionStore((s) => s.clearTimeline);
  const activeSessionId = useSessionStore((s) => s.activeSessionId);

  const status: FridaySessionStatus = session?.status ?? "idle";

  const derived: DerivedAgentState = {
    currentSession: session,
    status,
    statusMessage,
    timeline,
    pendingApproval,
    setSession: (s) => {
      void selectActiveSession(s?.id ?? null);
    },
    clearTimeline: () => {
      if (activeSessionId) clearTimelineFn(activeSessionId);
    },
    handleEvent,
  };

  return selector(derived);
}
