import type { PetMood } from "@friday/agent-core";

import { PET_MOOD_ICON } from "@/lib/pet-mood-icons";
import { cn } from "@/lib/utils";
import { useSettingsStore } from "@/state/useSettingsStore";

const MOOD_LABEL: Record<PetMood, string> = {
  calm: "Resting",
  focused: "Thinking",
  working: "Working",
  asking: "Needs approval",
  stressed: "Error",
  satisfied: "Done",
};

/** Static affordance — no ambient pulse/bounce (b.md productivity canvas). */
const MOOD_RING: Partial<Record<PetMood, string>> = {
  focused: "ring-2 ring-primary/40",
  working: "ring-2 ring-primary/60",
  asking: "ring-2 ring-amber-500/60",
  stressed: "ring-2 ring-destructive/70",
  satisfied: "ring-2 ring-emerald-500/50",
};

export function PetSprite({ mood }: { mood: PetMood }) {
  const petScale = Math.max(
    0.5,
    useSettingsStore((s) => s.settings.appearance.petScale) || 1,
  );
  const MoodIcon = PET_MOOD_ICON[mood];

  return (
    <div
      className="flex items-center justify-center"
      style={{ transform: `scale(${petScale})` }}
    >
      <div
        className={cn(
          "flex size-24 select-none flex-col items-center justify-center gap-1 rounded-full bg-primary/25 shadow-lg backdrop-blur-sm",
          MOOD_RING[mood],
        )}
      >
        <MoodIcon className="size-10 text-primary" strokeWidth={1.5} />
        <span className="text-[10px] font-medium text-primary-foreground">
          {MOOD_LABEL[mood]}
        </span>
      </div>
    </div>
  );
}
