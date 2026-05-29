import type { ReactNode } from "react";

import { MOTION } from "@/lib/motion";
import { cn } from "@/lib/utils";

/** Fade-in page body on mount / tab switch (opacity only — safe for reduced motion). */
export function MotionPage({
  children,
  className,
  pageKey,
}: {
  children: ReactNode;
  className?: string;
  /** Change when navigating to re-run enter animation. */
  pageKey: string;
}) {
  return (
    <div key={pageKey} className={cn(MOTION.pageIn, className)}>
      {children}
    </div>
  );
}

/** Staggered list enter for cards / rows (max 8 children get delay). */
export function MotionStagger({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return <div className={cn(MOTION.stagger, className)}>{children}</div>;
}

/** Single timeline / list item enter. */
export function MotionItem({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return <div className={cn(MOTION.itemIn, className)}>{children}</div>;
}
