import { useEffect } from "react";

import { useSettingsStore } from "@/state/useSettingsStore";

/** Syncs `reduce-motion` on `<html>` from settings + `prefers-reduced-motion`. */
export function useReducedMotionClass() {
  const reducedMotion = useSettingsStore(
    (s) => s.settings.appearance.reducedMotion,
  );

  useEffect(() => {
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");

    const apply = () => {
      document.documentElement.classList.toggle(
        "reduce-motion",
        reducedMotion || mq.matches,
      );
    };

    apply();
    mq.addEventListener("change", apply);
    return () => mq.removeEventListener("change", apply);
  }, [reducedMotion]);
}
