import { useEffect, useRef, useState } from "react";
import type { PointerEvent as ReactPointerEvent } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

import type { PetMood } from "@friday/agent-core";
import { useDismissOnWindowBlur } from "@/hooks/useDismissOnWindowBlur";
import { useFridayReady } from "@/hooks/useFridayReady";
import { openPanel, openQuickBubble, petSurfaceReady } from "@/lib/tauri";
import { PET_HIT_RADIUS_BASE, PetEngine } from "@/pet-engine";
import { cn } from "@/lib/utils";
import { useSettingsStore } from "@/state/useSettingsStore";

import { PetContextMenu } from "./PetContextMenu";
import { PetSprite } from "./PetSprite";

const DRAG_THRESHOLD_PX = 6;

export function PetWindow() {
  const ready = useFridayReady();
  const loadSettings = useSettingsStore((s) => s.load);
  const [mood, setMood] = useState<PetMood>("calm");
  const [hoveringPet, setHoveringPet] = useState(false);
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);
  const rootRef = useRef<HTMLDivElement>(null);
  const petScale = useSettingsStore((s) =>
    Math.max(0.5, s.settings.appearance.petScale || 1),
  );
  const clickTimer = useRef<number | null>(null);
  const engineRef = useRef<PetEngine | null>(null);
  const setMoodRef = useRef(setMood);
  const dragRef = useRef<{
    pointerId: number;
    startX: number;
    startY: number;
    dragging: boolean;
  } | null>(null);
  setMoodRef.current = setMood;

  useEffect(() => {
    document.body.classList.add("bg-transparent");
    return () => document.body.classList.remove("bg-transparent");
  }, []);

  useEffect(() => {
    if (!ready) return;
    void loadSettings();
    void petSurfaceReady();
  }, [ready, loadSettings]);

  useEffect(() => {
    if (!ready) return;

    const engine = PetEngine.start({
      onMoodChange: (nextMood) => {
        setMoodRef.current((prev) => (prev === nextMood ? prev : nextMood));
      },
      onHoverChange: setHoveringPet,
    });
    engineRef.current = engine;
    return () => {
      engine.stop();
      engineRef.current = null;
    };
  }, [ready]);

  useEffect(() => {
    engineRef.current?.setPetScale(petScale);
  }, [petScale]);

  useEffect(() => {
    engineRef.current?.setInteractionLocked(menu !== null);
  }, [menu]);

  useDismissOnWindowBlur(menu !== null, () => setMenu(null));

  const clearClickTimer = () => {
    if (clickTimer.current) {
      window.clearTimeout(clickTimer.current);
      clickTimer.current = null;
    }
  };

  const handleClick = () => {
    if (dragRef.current?.dragging) return;

    if (clickTimer.current) {
      clearClickTimer();
      void openPanel();
      return;
    }

    clickTimer.current = window.setTimeout(() => {
      void openQuickBubble();
      clickTimer.current = null;
    }, 220);
  };

  const handlePointerDown = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) return;

    dragRef.current = {
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      dragging: false,
    };
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const toWindowLocal = (clientX: number, clientY: number) => {
    const root = rootRef.current;
    if (!root) return null;
    const rect = root.getBoundingClientRect();
    return { x: clientX - rect.left, y: clientY - rect.top };
  };

  const handlePointerMove = (event: ReactPointerEvent<HTMLDivElement>) => {
    const local = toWindowLocal(event.clientX, event.clientY);
    if (local) {
      engineRef.current?.handlePointerMove(local.x, local.y);
    }

    const drag = dragRef.current;
    if (!drag || drag.pointerId !== event.pointerId || drag.dragging) return;

    const dx = event.clientX - drag.startX;
    const dy = event.clientY - drag.startY;
    if (Math.hypot(dx, dy) < DRAG_THRESHOLD_PX) return;

    drag.dragging = true;
    clearClickTimer();
    engineRef.current?.setDragging(true);
    void getCurrentWindow().startDragging();
  };

  const endPointer = (event: ReactPointerEvent<HTMLDivElement>) => {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== event.pointerId) return;

    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }

    if (drag.dragging) {
      clearClickTimer();
    }

    dragRef.current = null;
    engineRef.current?.setDragging(false);
  };

  const handleContextMenu = (event: ReactPointerEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();
    clearClickTimer();
    setMenu({ x: event.clientX, y: event.clientY });
  };

  return (
    <div
      ref={rootRef}
      className={cn(
        "relative h-screen w-screen overflow-hidden bg-transparent",
        menu && "pointer-events-auto",
      )}
    >
      <div className="pointer-events-none flex h-full w-full items-center justify-center">
        <div
          className={cn(
            "relative flex items-center justify-center rounded-full",
            hoveringPet && "pointer-events-auto cursor-grab active:cursor-grabbing",
          )}
          style={{
            width: PET_HIT_RADIUS_BASE * 2 * petScale,
            height: PET_HIT_RADIUS_BASE * 2 * petScale,
          }}
          onPointerDown={handlePointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={endPointer}
          onPointerCancel={endPointer}
          onClick={handleClick}
          onContextMenu={handleContextMenu}
        >
          <PetSprite mood={mood} />
        </div>
      </div>
      {menu && (
        <PetContextMenu
          x={menu.x}
          y={menu.y}
          onClose={() => setMenu(null)}
        />
      )}
    </div>
  );
}
