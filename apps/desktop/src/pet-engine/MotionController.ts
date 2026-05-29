import type { MonitorInfo } from "@/lib/tauri";

import type { PetActor } from "./PetActor";

const PATROL_SPEED = 45;
const BOTTOM_MARGIN = 16;

export class MotionController {
  private monitor: MonitorInfo | null = null;
  private lastAppliedX = NaN;
  private lastAppliedY = NaN;
  private patrolDirection = 1;
  private idleTimer = 0;
  private positionInitialized = false;

  setMonitor(monitor: MonitorInfo) {
    this.monitor = monitor;
  }

  markPositionInitialized() {
    this.positionInitialized = true;
  }

  tick(
    actor: PetActor,
    behaviorState: "idle" | "walk",
    dt: number,
  ): { x: number; y: number; nextBehavior: "idle" | "walk" } | null {
    if (!this.monitor) {
      return null;
    }

    const { workAreaX, workAreaY, workAreaWidth, workAreaHeight } =
      this.monitor;
    const windowSize = 160;
    const minX = workAreaX;
    const maxX = workAreaX + workAreaWidth - windowSize;
    const patrolY = workAreaY + workAreaHeight - windowSize - BOTTOM_MARGIN;

    let { x, y } = actor.position;
    if (!this.positionInitialized && x === 0 && y === 0) {
      x = maxX - 24;
      y = patrolY;
      this.positionInitialized = true;
    }

    // Keep user-dragged height; only snap Y while the pet is still on the patrol line.
    if (Math.abs(y - patrolY) <= 2) {
      y = patrolY;
    }
    let nextBehavior = behaviorState;

    if (behaviorState === "idle") {
      this.idleTimer += dt;
      if (this.idleTimer > 2.5) {
        nextBehavior = "walk";
        this.idleTimer = 0;
      }
      actor.setPosition(x, y);
      const changed = this.changed(x, y);
      return changed ? { ...changed, nextBehavior } : null;
    }

    x += this.patrolDirection * PATROL_SPEED * dt;

    if (x <= minX) {
      x = minX;
      this.patrolDirection = 1;
      nextBehavior = "idle";
      this.idleTimer = 0;
    } else if (x >= maxX) {
      x = maxX;
      this.patrolDirection = -1;
      nextBehavior = "idle";
      this.idleTimer = 0;
    }

    actor.setVelocity(this.patrolDirection * PATROL_SPEED, 0);
    actor.setPosition(x, y);
    const changed = this.changed(x, y);
    return changed ? { ...changed, nextBehavior } : null;
  }

  avoidScreenEdge(actor: PetActor) {
    if (!this.monitor) return;
    const windowSize = 160;
    const minX = this.monitor.workAreaX;
    const maxX =
      this.monitor.workAreaX + this.monitor.workAreaWidth - windowSize;
    const minY = this.monitor.workAreaY;
    const maxY =
      this.monitor.workAreaY +
      this.monitor.workAreaHeight -
      windowSize -
      BOTTOM_MARGIN;

    actor.setPosition(
      Math.min(maxX, Math.max(minX, actor.position.x)),
      Math.min(maxY, Math.max(minY, actor.position.y)),
    );
  }

  settleNearTaskbar(actor: PetActor) {
    if (!this.monitor) return;
    const windowSize = 160;
    const targetY =
      this.monitor.workAreaY +
      this.monitor.workAreaHeight -
      windowSize -
      BOTTOM_MARGIN;
    actor.setPosition(actor.position.x, targetY);
  }

  private changed(x: number, y: number): { x: number; y: number } | null {
    if (
      Math.abs(x - this.lastAppliedX) < 1 &&
      Math.abs(y - this.lastAppliedY) < 1
    ) {
      return null;
    }
    this.lastAppliedX = x;
    this.lastAppliedY = y;
    return { x, y };
  }
}
