import type { FridaySessionStatus } from "./sessions.js";

export type PetMood =
  | "calm"
  | "focused"
  | "working"
  | "asking"
  | "stressed"
  | "satisfied";

export const STATUS_TO_MOOD: Record<FridaySessionStatus, PetMood> = {
  discovered: "calm",
  idle: "calm",
  starting: "focused",
  thinking: "focused",
  reading: "focused",
  editing: "working",
  running_command: "working",
  waiting_permission: "asking",
  testing: "focused",
  done: "satisfied",
  error: "stressed",
  stopped: "calm",
};

/** Lucide icon names for pet mood UI — import matching icons from `lucide-react`. */
export const PET_MOOD_LUCIDE_ICON: Record<PetMood, string> = {
  calm: "moon",
  focused: "brain",
  working: "keyboard",
  asking: "hand",
  stressed: "triangle-alert",
  satisfied: "circle-check",
};

export const STATUS_LABELS: Record<FridaySessionStatus, string> = {
  discovered: "Discovered",
  idle: "Idle",
  starting: "Starting",
  thinking: "Thinking",
  reading: "Reading",
  editing: "Editing",
  running_command: "Running command",
  waiting_permission: "Waiting permission",
  testing: "Testing",
  done: "Done",
  error: "Error",
  stopped: "Stopped",
};
