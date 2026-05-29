import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/sonner";

import { useReducedMotionClass } from "@/hooks/useReducedMotionClass";

export function AppProviders({ children }: { children: React.ReactNode }) {
  useReducedMotionClass();

  return (
    <TooltipProvider>
      {children}
      <Toaster richColors closeButton position="top-right" />
    </TooltipProvider>
  );
}
