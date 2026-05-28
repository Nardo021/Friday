import { getCurrentWindow } from "@tauri-apps/api/window";

import {
  openChat,
  openCommandCenter,
  openQuickBubble,
} from "@/lib/tauri";

export function PetContextMenu({
  x,
  y,
  onClose,
}: {
  x: number;
  y: number;
  onClose: () => void;
}) {
  const items = [
    { label: "New Cursor Task", action: () => openChat() },
    { label: "Open Chat", action: () => openChat() },
    { label: "Quick Bubble", action: () => openQuickBubble() },
    { label: "Command Center", action: () => openCommandCenter() },
    { label: "Hide Pet", action: () => getCurrentWindow().hide() },
  ];

  return (
    <div
      className="fixed z-50 min-w-40 rounded-md border border-zinc-700 bg-zinc-900 py-1 shadow-xl"
      style={{ left: x, top: y }}
      onMouseLeave={onClose}
    >
      {items.map((item) => (
        <button
          key={item.label}
          className="block w-full px-3 py-2 text-left text-sm text-zinc-200 hover:bg-zinc-800"
          onClick={() => {
            item.action();
            onClose();
          }}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}
