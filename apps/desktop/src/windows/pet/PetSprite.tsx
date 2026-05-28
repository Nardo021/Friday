import { MOOD_EMOJI, type PetMood } from "@friday/agent-core";
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

const MOOD_MOTION: Partial<Record<PetMood, string>> = {
  calm: "animate-pulse",
  working: "animate-bounce",
  stressed: "animate-pulse",
};

export function PetSprite({ mood }: { mood: PetMood }) {
  const reducedMotion = useSettingsStore(
    (s) => s.settings.appearance.reducedMotion,
  );
  const petScale = useSettingsStore((s) => s.settings.appearance.petScale);

  return (
    <div
      className="flex items-center justify-center"
      style={{ transform: `scale(${petScale})` }}
    >
      <div
        className={cn(
          "flex h-24 w-24 select-none flex-col items-center justify-center rounded-full bg-indigo-600/30 text-4xl shadow-lg backdrop-blur-sm",
          !reducedMotion && MOOD_MOTION[mood],
        )}
      >
        <span>{MOOD_EMOJI[mood]}</span>
        <span className="mt-1 text-[10px] font-medium text-indigo-100">
          {MOOD_LABEL[mood]}
        </span>
      </div>
    </div>
  );
}
