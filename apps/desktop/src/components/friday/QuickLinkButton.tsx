import type { LucideIcon } from "lucide-react";

import { Button } from "@/components/ui/button";

export function QuickLinkButton({
  icon: Icon,
  label,
  onClick,
}: {
  icon: LucideIcon;
  label: string;
  onClick: () => void;
}) {
  return (
    <Button
      type="button"
      variant="outline"
      size="sm"
      className="motion-hover h-9 gap-1.5"
      onClick={onClick}
    >
      <Icon className="size-3.5" aria-hidden />
      {label}
    </Button>
  );
}
