import { useEffect } from "react";

import { listenAgentEvents } from "@/lib/tauri";
import { useAgentStore } from "@/state/useAgentStore";

export function useAgentEventBridge() {
  const handleEvent = useAgentStore((s) => s.handleEvent);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listenAgentEvents(handleEvent).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, [handleEvent]);
}
