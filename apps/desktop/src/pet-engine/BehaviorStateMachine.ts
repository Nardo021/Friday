import type { FridaySessionStatus } from "@friday/agent-core";
import { isRunningStatus } from "@friday/agent-core";

export type PetBehaviorState =
  | "idle"
  | "walk"
  | "dragged"
  | "thinking"
  | "editing"
  | "waitingApproval"
  | "error"
  | "done";

export class BehaviorStateMachine {
  private state: PetBehaviorState = "idle";

  get current(): PetBehaviorState {
    return this.state;
  }

  setDragged(dragging: boolean) {
    this.state = dragging ? "dragged" : "idle";
  }

  updateFromAgentStatus(status: FridaySessionStatus) {
    if (this.state === "dragged") return;

    if (status === "waiting_permission") {
      this.state = "waitingApproval";
      return;
    }
    if (status === "error") {
      this.state = "error";
      return;
    }
    if (status === "done") {
      this.state = "done";
      return;
    }
    if (isRunningStatus(status)) {
      if (["editing", "running_command"].includes(status)) {
        this.state = "editing";
      } else {
        this.state = "thinking";
      }
      return;
    }
    this.state = "idle";
  }

  shouldPatrol(): boolean {
    return this.state === "idle" || this.state === "walk";
  }

  setWalk() {
    if (this.state === "idle" || this.state === "walk") {
      this.state = "walk";
    }
  }

  setIdle() {
    if (this.state === "walk") {
      this.state = "idle";
    }
  }
}
