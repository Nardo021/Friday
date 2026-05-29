import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import { getSettings } from "@/lib/tauri";

export const FRIDAY_READY_EVENT = "friday://ready";

/** Wait until the Tauri backend finished setup before invoking agent commands. */
export function useFridayReady(): boolean {
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const markReady = () => {
      if (!cancelled) setReady(true);
    };

    void (async () => {
      for (let attempt = 0; attempt < 40; attempt++) {
        if (cancelled) return;
        try {
          await getSettings();
          markReady();
          return;
        } catch {
          await new Promise((resolve) => setTimeout(resolve, 100));
        }
      }
      markReady();
    })();

    void listen(FRIDAY_READY_EVENT, markReady).then((unlisten) => {
      if (cancelled) {
        unlisten();
        return;
      }
      return () => unlisten();
    });

    return () => {
      cancelled = true;
    };
  }, []);

  return ready;
}
