import { getCurrentWindow } from "@tauri-apps/api/window";

import {
  openCommandCenter,
  openPanel,
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
    { label: "Quick chat", action: () => void openQuickBubble() },
    { label: "Open panel", action: () => void openPanel() },
    { label: "Command center", action: () => void openCommandCenter() },
    { label: "Hide pet", action: () => void getCurrentWindow().hide() },
  ];

  const menuWidth = 132;
  const left = Math.min(x, window.innerWidth - menuWidth - 4);
  const top = Math.min(y, window.innerHeight - items.length * 28 - 8);

  return (
    <>
      <button
        type="button"
        aria-label="Close menu"
        className="fixed inset-0 z-40 cursor-default bg-transparent"
        onClick={onClose}
        onContextMenu={(event) => {
          event.preventDefault();
          onClose();
        }}
      />
      <div
        className="fixed z-50 w-[132px] overflow-hidden rounded-md border border-zinc-700/90 bg-zinc-900/95 py-0.5 shadow-lg backdrop-blur-sm"
        style={{ left, top }}
        onContextMenu={(event) => event.preventDefault()}
      >
        {items.map((item) => (
          <button
            key={item.label}
            type="button"
            className="block w-full px-2.5 py-1 text-left text-[11px] leading-tight text-zinc-200 hover:bg-zinc-800"
            onClick={() => {
              item.action();
              onClose();
            }}
          >
            {item.label}
          </button>
        ))}
      </div>
    </>
  );
}
