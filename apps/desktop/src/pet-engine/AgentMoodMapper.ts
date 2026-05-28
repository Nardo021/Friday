import type { FridaySessionStatus } from "@friday/agent-core";
import { STATUS_TO_MOOD, type PetMood } from "@friday/agent-core";

import type { PetBehaviorState } from "./BehaviorStateMachine";

export function moodFromStatus(status: FridaySessionStatus): PetMood {
  return STATUS_TO_MOOD[status];
}

export function behaviorFromStatus(status: FridaySessionStatus): PetBehaviorState {
  if (status === "waiting_permission") return "waitingApproval";
  if (status === "error") return "error";
  if (status === "done") return "done";
  if (["editing", "running_command"].includes(status)) return "editing";
  if (
    [
      "starting",
      "thinking",
      "reading",
      "testing",
    ].includes(status)
  ) {
    return "thinking";
  }
  return "idle";
}

export function statusBubbleText(
  status: FridaySessionStatus,
  message?: string,
): string {
  if (message) return message;
  switch (status) {
    case "thinking":
      return "Friday is thinking...";
    case "reading":
      return "Reading files...";
    case "editing":
      return "Editing code...";
    case "running_command":
      return "Running command...";
    case "waiting_permission":
      return "Cursor needs approval";
    case "error":
      return "Something went wrong";
    case "done":
      return "Task completed";
    case "starting":
      return "Starting session...";
    default:
      return "";
  }
}
