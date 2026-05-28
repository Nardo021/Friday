import type { FridaySessionStatus, PetMood } from "@friday/agent-core";
import { isRunningStatus, STATUS_TO_MOOD } from "@friday/agent-core";

import type { PetBehaviorState } from "./BehaviorStateMachine";

export interface PetPosition {
  x: number;
  y: number;
}

export interface PetVelocity {
  x: number;
  y: number;
}

export class PetActor {
  position: PetPosition = { x: 0, y: 0 };
  velocity: PetVelocity = { x: 0, y: 0 };
  direction = 1;
  mood: PetMood = "calm";
  behaviorState: PetBehaviorState = "idle";
  agentStatus: FridaySessionStatus = "idle";
  isDragging = false;

  setPosition(x: number, y: number) {
    this.position = { x, y };
  }

  setVelocity(x: number, y: number) {
    this.velocity = { x, y };
    if (x !== 0) {
      this.direction = x > 0 ? 1 : -1;
    }
  }

  setMoodFromStatus(status: FridaySessionStatus, message?: string) {
    this.agentStatus = status;
    this.mood = STATUS_TO_MOOD[status];
    if (message) {
      // mood only; bubble text handled by BubbleController
    }
  }

  canPatrol(): boolean {
    if (this.isDragging) return false;
    if (isRunningStatus(this.agentStatus)) return false;
    return this.behaviorState === "idle" || this.behaviorState === "walk";
  }
}
