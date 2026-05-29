import { getCurrentWindow } from "@tauri-apps/api/window";

import { OnboardingWindow } from "@/windows/onboarding/OnboardingWindow";
import { FridayPanel } from "@/windows/panel/FridayPanel";
import { PetWindow } from "@/windows/pet/PetWindow";
import { QuickBubbleWindow } from "@/windows/quick-bubble/QuickBubbleWindow";
import { StatusBubbleWindow } from "@/windows/status-bubble/StatusBubbleWindow";

function resolveWindowLabel(): string {
  try {
    return getCurrentWindow().label;
  } catch {
    return "panel";
  }
}

export function WindowRouter() {
  const label = resolveWindowLabel();

  switch (label) {
    case "pet":
      return <PetWindow />;
    case "quick-bubble":
      return <QuickBubbleWindow />;
    case "status-bubble":
      return <StatusBubbleWindow />;
    case "onboarding":
      return <OnboardingWindow />;
    case "panel":
    case "chat":
      return <FridayPanel />;
    default:
      return <FridayPanel />;
  }
}