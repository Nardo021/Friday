import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { useAgentEventBridge } from "@/hooks/useAgentEventBridge";
import { openChat, openQuickBubble } from "@/lib/tauri";
import { useAgentStore } from "@/state/useAgentStore";
import { usePetStore } from "@/state/usePetStore";

import { PetBubble } from "./PetBubble";
import { PetContextMenu } from "./PetContextMenu";
import { PetSprite } from "./PetSprite";

export function PetWindow() {
  const mood = usePetStore((s) => s.mood);
  const status = useAgentStore((s) => s.status);
  const statusMessage = useAgentStore((s) => s.statusMessage);
  const setFromStatus = usePetStore((s) => s.setFromStatus);
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);
  const clickTimer = useRef<number | null>(null);

  useAgentEventBridge();

  useEffect(() => {
    setFromStatus(status, statusMessage);
  }, [status, statusMessage, setFromStatus]);

  useEffect(() => {
    document.body.classList.add("bg-transparent");
    return () => document.body.classList.remove("bg-transparent");
  }, []);

  const handleClick = () => {
    if (clickTimer.current) {
      window.clearTimeout(clickTimer.current);
      clickTimer.current = null;
      openChat();
      return;
    }
    clickTimer.current = window.setTimeout(() => {
      openQuickBubble();
      clickTimer.current = null;
    }, 220);
  };

  return (
    <div
      className="relative h-screen w-screen overflow-hidden bg-transparent p-2"
      onContextMenu={(e) => {
        e.preventDefault();
        setMenu({ x: e.clientX, y: e.clientY });
      }}
    >
      <div
        className="relative cursor-grab active:cursor-grabbing"
        onMouseDown={() => getCurrentWindow().startDragging()}
        onClick={handleClick}
      >
        <PetBubble />
        <PetSprite mood={mood} />
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
