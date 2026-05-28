import type { AgentStatus } from "./types";

export type PetMood =
  | "calm"
  | "awake"
  | "focused"
  | "curious"
  | "working"
  | "intense"
  | "checking"
  | "asking"
  | "satisfied"
  | "stressed"
  | "neutral"
  | "waiting";

export const STATUS_TO_MOOD: Record<AgentStatus, PetMood> = {
  idle: "calm",
  starting: "awake",
  thinking: "focused",
  reading: "curious",
  editing: "working",
  running_command: "intense",
  testing: "checking",
  waiting_approval: "asking",
  completed: "satisfied",
  error: "stressed",
  cancelled: "neutral",
  paused: "waiting",
};

export const MOOD_EMOJI: Record<PetMood, string> = {
  calm: "😌",
  awake: "🐣",
  focused: "🤔",
  curious: "🔍",
  working: "⌨️",
  intense: "⚡",
  checking: "✅",
  asking: "🙋",
  satisfied: "🎉",
  stressed: "😰",
  neutral: "😐",
  waiting: "💤",
};

export const MOOD_LABEL: Record<PetMood, string> = {
  calm: "Resting",
  awake: "Booting up",
  focused: "Thinking",
  curious: "Reading",
  working: "Editing",
  intense: "Running command",
  checking: "Testing",
  asking: "Needs approval",
  satisfied: "Done",
  stressed: "Error",
  neutral: "Cancelled",
  waiting: "Paused",
};
