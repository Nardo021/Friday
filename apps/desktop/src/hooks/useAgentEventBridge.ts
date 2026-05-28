import { useEffect } from "react";

import { listenAgentEvents } from "@/lib/tauri";
import { useSessionStore } from "@/state/useSessionStore";

export function useAgentEventBridge() {
  const handleEvent = useSessionStore((s) => s.handleEvent);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listenAgentEvents(handleEvent).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, [handleEvent]);
}
