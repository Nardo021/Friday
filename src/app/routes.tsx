import { getCurrentWindow } from "@tauri-apps/api/window";

import { ChatWindow } from "@/windows/chat/ChatWindow";
import { CommandCenter } from "@/windows/command-center/CommandCenter";
import { PetWindow } from "@/windows/pet/PetWindow";
import { QuickBubbleWindow } from "@/windows/quick-bubble/QuickBubbleWindow";

export function WindowRouter() {
  const label = getCurrentWindow().label;

  switch (label) {
    case "pet":
      return <PetWindow />;
    case "quick-bubble":
      return <QuickBubbleWindow />;
    case "command-center":
      return <CommandCenter />;
    case "chat":
    default:
      return <ChatWindow />;
  }
}
