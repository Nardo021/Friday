import { motion } from "framer-motion";

import { MOOD_EMOJI, MOOD_LABEL } from "@/agent/mood-map";
import type { PetMood } from "@/agent/mood-map";
import { useSettingsStore } from "@/state/useSettingsStore";

const variants = {
  calm: { scale: [1, 1.03, 1], transition: { repeat: Infinity, duration: 3 } },
  awake: { y: [0, -4, 0], transition: { repeat: Infinity, duration: 1.2 } },
  focused: { rotate: [0, 2, -2, 0], transition: { repeat: Infinity, duration: 2 } },
  curious: { x: [0, 3, 0], transition: { repeat: Infinity, duration: 1.5 } },
  working: { scale: [1, 0.96, 1], transition: { repeat: Infinity, duration: 0.8 } },
  intense: { scale: [1, 1.08, 1], transition: { repeat: Infinity, duration: 0.5 } },
  checking: { rotate: [0, 5, 0], transition: { repeat: Infinity, duration: 1 } },
  asking: { y: [0, -6, 0], transition: { repeat: Infinity, duration: 1.2 } },
  satisfied: { scale: [1, 1.1, 1], transition: { duration: 0.6 } },
  stressed: { x: [-2, 2, -2], transition: { repeat: Infinity, duration: 0.3 } },
  neutral: { opacity: [1, 0.85, 1], transition: { repeat: Infinity, duration: 2 } },
  waiting: { opacity: [0.7, 1, 0.7], transition: { repeat: Infinity, duration: 2.5 } },
};

export function PetSprite({ mood }: { mood: PetMood }) {
  const reducedMotion = useSettingsStore((s) => s.settings.appearance.reducedMotion);
  const scale = useSettingsStore((s) => s.settings.appearance.petScale);

  return (
    <motion.div
      className="flex h-24 w-24 select-none flex-col items-center justify-center rounded-full bg-indigo-600/30 text-4xl shadow-lg backdrop-blur-sm"
      style={{ scale }}
      animate={reducedMotion ? undefined : variants[mood]}
    >
      <span>{MOOD_EMOJI[mood]}</span>
      <span className="mt-1 text-[10px] font-medium text-indigo-100">
        {MOOD_LABEL[mood]}
      </span>
    </motion.div>
  );
}
