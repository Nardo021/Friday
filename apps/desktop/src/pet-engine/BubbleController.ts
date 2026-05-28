import {
  anchorWindow,
  hideStatusBubble,
  showStatusBubble,
} from "@/lib/tauri";

import { statusBubbleText } from "./AgentMoodMapper";
import type { PetActor } from "./PetActor";

export class BubbleController {
  private collapseTimer: number | null = null;
  private lastText = "";

  async onStatusChange(
    actor: PetActor,
    status: Parameters<typeof statusBubbleText>[0],
    message: string | undefined,
    options: {
      showOnChange: boolean;
      autoCollapse: boolean;
    },
  ) {
    const text = statusBubbleText(status, message);
    if (!text || !options.showOnChange) {
      await hideStatusBubble();
      return;
    }

    if (text === this.lastText) {
      await anchorWindow("status-bubble", 0, -88);
      return;
    }

    this.lastText = text;
    actor.setMoodFromStatus(status, message);
    await showStatusBubble();

    if (this.collapseTimer) {
      window.clearTimeout(this.collapseTimer);
    }
    if (options.autoCollapse) {
      this.collapseTimer = window.setTimeout(() => {
        void hideStatusBubble();
        this.lastText = "";
      }, 5000);
    }
  }

  async followPet() {
    await anchorWindow("status-bubble", 0, -88);
  }

  dispose() {
    if (this.collapseTimer) {
      window.clearTimeout(this.collapseTimer);
    }
  }
}
