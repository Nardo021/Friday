import type { FridaySessionStatus } from "@friday/agent-core";
import { getCurrentWindow } from "@tauri-apps/api/window";

import type { AgentEvent } from "@friday/agent-core";
import { isRunningStatus } from "@friday/agent-core";
import { listenAgentEvents } from "@/lib/tauri";
import {
  getMonitorInfo,
  getPetPosition,
  getSettings,
  listSessions,
  saveSettings,
  setPetPosition,
  setWindowClickThrough,
} from "@/lib/tauri";

import { BehaviorStateMachine } from "./BehaviorStateMachine";
import { BubbleController } from "./BubbleController";
import { MotionController } from "./MotionController";
import { PetActor } from "./PetActor";

const TICK_MS = 50;
const PET_LABEL = "pet";
const BUBBLE_FOLLOW_INTERVAL_MS = 120;

export interface PetEngineOptions {
  onMoodChange?: (mood: PetActor["mood"]) => void;
}

export class PetEngine {
  readonly actor = new PetActor();
  private readonly bsm = new BehaviorStateMachine();
  private readonly motion = new MotionController();
  private readonly bubble = new BubbleController();
  private options: PetEngineOptions;
  private timer: number | null = null;
  private lastTick = 0;
  private patrolEnabled = true;
  private saveTimer: number | null = null;
  private moveUnlisten: (() => void) | null = null;
  private eventUnlisten: (() => void) | null = null;
  private running = false;
  private isMovingProgrammatically = false;
  private lastNotifiedMood: PetActor["mood"] | null = null;
  private lastAgentStatusKey = "";
  private clickThroughEnabled: boolean | null = null;
  private lastSavedPosition = { x: NaN, y: NaN };
  private initGeneration = 0;
  private behaviorShowBubble = true;
  private behaviorAutoCollapse = true;
  private lastBubbleFollowAt = 0;

  private constructor(options: PetEngineOptions) {
    this.options = options;
  }

  static start(options: PetEngineOptions = {}): PetEngine {
    const engine = new PetEngine(options);
    void engine.init();
    return engine;
  }

  private async init() {
    const generation = ++this.initGeneration;
    this.running = true;

    const [pos, monitor, settings, sessions] = await Promise.all([
      getPetPosition(),
      getMonitorInfo(),
      getSettings(),
      listSessions().catch(() => []),
    ]);

    if (!this.isCurrentInit(generation)) return;

    this.actor.setPosition(pos.x, pos.y);
    this.patrolEnabled = settings.pet?.patrolEnabled ?? true;
    this.behaviorShowBubble = settings.behavior.showBubbleOnStatusChange;
    this.behaviorAutoCollapse = settings.behavior.autoCollapseBubble;
    this.motion.setMonitor(monitor);
    this.motion.markPositionInitialized();
    this.clickThroughEnabled = false;
    void this.setClickThrough(false);

    const win = getCurrentWindow();
    this.moveUnlisten = await win.onMoved(() => {
      void this.syncPositionFromWindow();
    });
    if (!this.isCurrentInit(generation)) {
      this.moveUnlisten?.();
      this.moveUnlisten = null;
      return;
    }

    this.eventUnlisten = await listenAgentEvents((event) => {
      this.handleAgentEvent(event);
    });
    if (!this.isCurrentInit(generation)) {
      this.moveUnlisten?.();
      this.moveUnlisten = null;
      this.eventUnlisten?.();
      this.eventUnlisten = null;
      return;
    }

    const active =
      sessions.find((s) => isRunningStatus(s.status)) ?? sessions[0];
    if (active) {
      await this.updateAgentStatus(active.status, active.summary ?? active.prompt);
    } else {
      await this.updateAgentStatus("idle");
    }
    if (!this.isCurrentInit(generation)) return;

    this.lastTick = performance.now();
    this.timer = window.setInterval(() => void this.tick(), TICK_MS);
  }

  private isCurrentInit(generation: number): boolean {
    return this.running && generation === this.initGeneration;
  }

  private handleAgentEvent(event: AgentEvent) {
    switch (event.type) {
      case "agent.status":
        void this.updateAgentStatus(event.status, event.message);
        break;
      case "session.completed":
        void this.updateAgentStatus("done", event.summary);
        break;
      case "session.error":
        void this.updateAgentStatus("error", event.error);
        break;
      case "session.started":
      case "session.discovered":
        void this.updateAgentStatus("starting");
        break;
      default:
        break;
    }
  }

  private async syncPositionFromWindow() {
    if (this.isMovingProgrammatically) return;

    const pos = await getPetPosition();
    this.actor.setPosition(pos.x, pos.y);
    this.scheduleSave(pos.x, pos.y);
    await this.bubble.followPet();
  }

  private notifyMood(mood: PetActor["mood"]) {
    if (mood === this.lastNotifiedMood) return;
    this.lastNotifiedMood = mood;
    this.options.onMoodChange?.(mood);
  }

  private async tick() {
    if (!this.running) return;

    const now = performance.now();
    const dt = Math.min(0.1, (now - this.lastTick) / 1000);
    this.lastTick = now;

    if (!this.patrolEnabled || !this.bsm.shouldPatrol() || this.actor.isDragging) {
      return;
    }

    const behavior = this.bsm.current === "walk" ? "walk" : "idle";
    const delta = this.motion.tick(this.actor, behavior, dt);
    if (!delta) return;

    if (delta.nextBehavior === "walk") {
      this.bsm.setWalk();
    } else {
      this.bsm.setIdle();
    }
    this.actor.behaviorState = this.bsm.current;

    this.isMovingProgrammatically = true;
    try {
      await setPetPosition(delta.x, delta.y);
      this.actor.setPosition(delta.x, delta.y);
    } finally {
      this.isMovingProgrammatically = false;
    }
    if (now - this.lastBubbleFollowAt >= BUBBLE_FOLLOW_INTERVAL_MS) {
      this.lastBubbleFollowAt = now;
      await this.bubble.followPet();
    }
  }

  setDragging(dragging: boolean) {
    if (this.actor.isDragging === dragging) return;

    this.actor.isDragging = dragging;
    this.bsm.setDragged(dragging);
    this.actor.behaviorState = this.bsm.current;

    if (dragging) {
      void this.setClickThrough(false);
    }
  }

  async updateAgentStatus(status: FridaySessionStatus, message?: string) {
    const statusKey = `${status}:${message ?? ""}`;
    if (statusKey === this.lastAgentStatusKey) return;
    this.lastAgentStatusKey = statusKey;

    this.bsm.updateFromAgentStatus(status);
    this.actor.behaviorState = this.bsm.current;
    this.actor.setMoodFromStatus(status, message);
    this.notifyMood(this.actor.mood);

    await this.bubble.onStatusChange(this.actor, status, message, {
      showOnChange: this.behaviorShowBubble,
      autoCollapse: this.behaviorAutoCollapse,
    });
  }

  handlePointerMove(localX: number, localY: number) {
    if (this.actor.isDragging) return;
    // Keep the pet window fully interactive; corner click-through caused missed clicks.
    if (this.clickThroughEnabled !== false) {
      this.clickThroughEnabled = false;
      void this.setClickThrough(false);
    }
    void localX;
    void localY;
  }

  private async setClickThrough(enabled: boolean) {
    await setWindowClickThrough(PET_LABEL, enabled);
  }

  scheduleSave(x: number, y: number) {
    if (
      Math.abs(x - this.lastSavedPosition.x) < 1 &&
      Math.abs(y - this.lastSavedPosition.y) < 1
    ) {
      return;
    }
    this.lastSavedPosition = { x, y };

    if (this.saveTimer) {
      window.clearTimeout(this.saveTimer);
    }
    this.saveTimer = window.setTimeout(() => {
      void this.persistPosition(x, y);
    }, 800);
  }

  private async persistPosition(x: number, y: number) {
    const settings = await getSettings();
    await saveSettings({
      ...settings,
      pet: {
        ...settings.pet,
        lastX: x,
        lastY: y,
        patrolEnabled: this.patrolEnabled,
      },
    });
  }

  stop() {
    this.running = false;
    this.initGeneration += 1;
    if (this.timer) {
      window.clearInterval(this.timer);
      this.timer = null;
    }
    if (this.saveTimer) {
      window.clearTimeout(this.saveTimer);
      this.saveTimer = null;
    }
    this.bubble.dispose();
    this.moveUnlisten?.();
    this.moveUnlisten = null;
    this.eventUnlisten?.();
    this.eventUnlisten = null;
  }
}
