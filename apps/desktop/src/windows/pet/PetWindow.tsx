import { useEffect, useRef, useState } from "react";
import type { PointerEvent as ReactPointerEvent } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

import type { PetMood } from "@friday/agent-core";
import { openPanel, openQuickBubble } from "@/lib/tauri";
import { PetEngine } from "@/pet-engine";

import { PetContextMenu } from "./PetContextMenu";
import { PetSprite } from "./PetSprite";

const DRAG_THRESHOLD_PX = 6;

export function PetWindow() {
  const [mood, setMood] = useState<PetMood>("calm");
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);
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
    const engine = PetEngine.start({
      onMoodChange: (nextMood) => {
        setMoodRef.current((prev) => (prev === nextMood ? prev : nextMood));
      },
    });
    engineRef.current = engine;
    return () => {
      engine.stop();
      engineRef.current = null;
    };
  }, []);

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

  const handlePointerMove = (event: ReactPointerEvent<HTMLDivElement>) => {
    engineRef.current?.handlePointerMove(event.clientX, event.clientY);

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
    <div className="relative h-screen w-screen overflow-hidden bg-transparent">
      <div className="flex h-full w-full items-center justify-center">
        <div
          className="relative cursor-grab active:cursor-grabbing"
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
