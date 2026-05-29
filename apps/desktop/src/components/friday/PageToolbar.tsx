import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

/** Filter row / actions aligned to the end of a page (Fitts: primary actions together). */
export function PageToolbar({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "flex flex-wrap items-end justify-between gap-3",
        className,
      )}
    >
      {children}
    </div>
  );
}
