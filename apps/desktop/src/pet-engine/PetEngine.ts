import type { FridaySessionStatus } from "@friday/agent-core";
import { cursorPosition, getCurrentWindow } from "@tauri-apps/api/window";

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
import { HitTestEngine } from "./HitTestEngine";
import { MotionController } from "./MotionController";
import { PetActor } from "./PetActor";

const TICK_MS = 50;
const HIT_POLL_MS = 32;
const PET_LABEL = "pet";
const BUBBLE_FOLLOW_INTERVAL_MS = 120;

export interface PetEngineOptions {
  onMoodChange?: (mood: PetActor["mood"]) => void;
  onHoverChange?: (hovering: boolean) => void;
}

export class PetEngine {
  readonly actor = new PetActor();
  private readonly bsm = new BehaviorStateMachine();
  private readonly motion = new MotionController();
  private readonly bubble = new BubbleController();
  private readonly hitTest = new HitTestEngine();
  private options: PetEngineOptions;
  private timer: number | null = null;
  private hitPollTimer: number | null = null;
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
  private isHoveringPet = false;
  private interactionLocked = false;
  private petScale = 1;
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

    let pos: Awaited<ReturnType<typeof getPetPosition>>;
    let monitor: Awaited<ReturnType<typeof getMonitorInfo>>;
    let settings: Awaited<ReturnType<typeof getSettings>>;
    let sessions: Awaited<ReturnType<typeof listSessions>>;

    try {
      [pos, monitor, settings, sessions] = await Promise.all([
        getPetPosition(),
        getMonitorInfo(),
        getSettings(),
        listSessions().catch(() => []),
      ]);
    } catch {
      if (!this.isCurrentInit(generation)) return;
      this.notifyMood("calm");
      return;
    }

    if (!this.isCurrentInit(generation)) return;

    this.actor.setPosition(pos.x, pos.y);
    this.patrolEnabled = settings.pet?.patrolEnabled ?? true;
    this.behaviorShowBubble = settings.behavior.showBubbleOnStatusChange;
    this.behaviorAutoCollapse = settings.behavior.autoCollapseBubble;
    this.motion.setMonitor(monitor);
    this.motion.markPositionInitialized();
    this.petScale = Math.max(0.5, settings.appearance?.petScale ?? 1);
    this.hitTest.setScale(this.petScale);
    this.clickThroughEnabled = true;
    void this.setClickThrough(true);
    this.startHitPolling();

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
    if (this.isMovingProgrammatically || this.interactionLocked) return;

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

    if (
      !this.patrolEnabled ||
      !this.bsm.shouldPatrol() ||
      this.actor.isDragging ||
      this.interactionLocked
    ) {
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

  setPetScale(scale: number) {
    const next = Math.max(0.5, scale);
    if (next === this.petScale) return;
    this.petScale = next;
    this.hitTest.setScale(next);
  }

  setInteractionLocked(locked: boolean) {
    if (this.interactionLocked === locked) return;
    this.interactionLocked = locked;
    if (locked) {
      if (this.clickThroughEnabled !== false) {
        this.clickThroughEnabled = false;
        void this.setClickThrough(false);
      }
    } else {
      void this.pollPointerHit();
    }
  }

  setDragging(dragging: boolean) {
    if (this.actor.isDragging === dragging) return;

    this.actor.isDragging = dragging;
    this.bsm.setDragged(dragging);
    this.actor.behaviorState = this.bsm.current;

    if (dragging) {
      void this.applyHitState(true);
    } else {
      void this.pollPointerHit();
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
    if (this.actor.isDragging || this.interactionLocked) return;
    void this.applyHitState(this.hitTest.isSolidPixel(localX, localY));
  }

  private startHitPolling() {
    if (this.hitPollTimer) return;
    this.hitPollTimer = window.setInterval(() => {
      void this.pollPointerHit();
    }, HIT_POLL_MS);
  }

  private async pollPointerHit() {
    if (!this.running || this.actor.isDragging || this.interactionLocked) {
      return;
    }

    try {
      const win = getCurrentWindow();
      const [cursor, origin, size] = await Promise.all([
        cursorPosition(),
        win.outerPosition(),
        win.outerSize(),
      ]);
      this.hitTest.setWindowSize(size.width, size.height);
      const localX = cursor.x - origin.x;
      const localY = cursor.y - origin.y;
      await this.applyHitState(this.hitTest.isSolidPixel(localX, localY));
    } catch {
      // ignore transient window/cursor API errors
    }
  }

  private async applyHitState(hit: boolean) {
    const wantClickThrough = !hit;
    if (this.clickThroughEnabled !== wantClickThrough) {
      this.clickThroughEnabled = wantClickThrough;
      await this.setClickThrough(wantClickThrough);
    }

    if (this.isHoveringPet !== hit) {
      this.isHoveringPet = hit;
      this.options.onHoverChange?.(hit);
    }
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
    if (this.hitPollTimer) {
      window.clearInterval(this.hitPollTimer);
      this.hitPollTimer = null;
    }
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
