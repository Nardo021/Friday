import { useCallback } from "react";

import { usePanelPageStore } from "@/state/usePanelPageStore";
import { useSessionStore } from "@/state/useSessionStore";
import type { PanelPageId } from "@/windows/panel/PanelNav";

/** Switch panel tab and optionally focus a session in the agent workspace. */
export function usePanelNavigation() {
  const setPage = usePanelPageStore((s) => s.setPage);
  const selectActiveSession = useSessionStore((s) => s.selectActiveSession);

  const goToPage = useCallback(
    (page: PanelPageId) => {
      setPage(page);
    },
    [setPage],
  );

  const goToAgent = useCallback(
    (sessionId?: string) => {
      if (sessionId) void selectActiveSession(sessionId);
      setPage("agent");
    },
    [selectActiveSession, setPage],
  );

  return { goToPage, goToAgent };
}
