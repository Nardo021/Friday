import { StatusPill } from "@/components/friday/StatusPill";
import { useAgentStore } from "@/state/useAgentStore";

export function PetBubble() {
  const status = useAgentStore((s) => s.status);
  const message = useAgentStore((s) => s.statusMessage);
  const bubbleVisible = useAgentStore((s) => s.statusMessage);

  if (!bubbleVisible && status === "idle") return null;

  return (
    <div className="absolute -top-16 left-1/2 w-48 -translate-x-1/2 rounded-lg border border-zinc-700 bg-zinc-900/95 px-3 py-2 text-xs shadow-xl backdrop-blur">
      <div className="mb-1">
        <StatusPill status={status} />
      </div>
      {message && <p className="text-zinc-300">{message}</p>}
    </div>
  );
}
