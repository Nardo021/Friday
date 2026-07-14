import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  EyeOff,
  MessageSquare,
  PanelTop,
  type LucideIcon,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { openPanel, openQuickBubble } from "@/lib/tauri";

const MENU_ITEMS: {
  label: string;
  icon: LucideIcon;
  action: () => void;
}[] = [
  { label: "Quick chat", icon: MessageSquare, action: () => void openQuickBubble() },
  { label: "Open panel", icon: PanelTop, action: () => void openPanel() },
  {
    label: "Hide pet",
    icon: EyeOff,
    action: () => void getCurrentWindow().hide(),
  },
];

export function PetContextMenu({
  x,
  y,
  onClose,
}: {
  x: number;
  y: number;
  onClose: () => void;
}) {
  const menuWidth = 116;
  const rowHeight = 24;
  const menuPad = 4;
  const left = Math.min(x, window.innerWidth - menuWidth - 4);
  const top = Math.min(
    y,
    window.innerHeight - MENU_ITEMS.length * rowHeight - menuPad * 2 - 4,
  );

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
        className={cn(
          "motion-popover-in fixed z-50 w-[116px] overflow-hidden rounded-md border bg-popover p-0.5 text-popover-foreground shadow-md backdrop-blur-sm",
        )}
        style={{ left, top }}
        onContextMenu={(event) => event.preventDefault()}
      >
        {MENU_ITEMS.map((item) => {
          const Icon = item.icon;
          return (
            <Button
              key={item.label}
              type="button"
              variant="ghost"
              size="xs"
              className="motion-hover h-6 w-full justify-start gap-1.5 rounded-sm px-1.5 text-[11px] font-normal [&_svg]:size-3"
              onClick={() => {
                item.action();
                onClose();
              }}
            >
              <Icon aria-hidden />
              {item.label}
            </Button>
          );
        })}
      </div>
    </>
  );
}
