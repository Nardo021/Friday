import { PET_MOOD_LUCIDE_ICON, type PetMood } from "@friday/agent-core";
import {
  Brain,
  CircleCheck,
  Hand,
  Keyboard,
  Moon,
  TriangleAlert,
  type LucideIcon,
} from "lucide-react";

/** Desktop pet mood icons — names aligned with {@link PET_MOOD_LUCIDE_ICON}. */
export const PET_MOOD_ICON: Record<PetMood, LucideIcon> = {
  calm: Moon,
  focused: Brain,
  working: Keyboard,
  asking: Hand,
  stressed: TriangleAlert,
  satisfied: CircleCheck,
};

export { PET_MOOD_LUCIDE_ICON };
