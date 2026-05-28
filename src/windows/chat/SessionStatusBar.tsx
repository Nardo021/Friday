import { formatElapsed } from "@/lib/time";
import { useAgentStore } from "@/state/useAgentStore";
import { isRunningStatus } from "@/agent/status";

export function SessionStatusBar() {
  const session = useAgentStore((s) => s.currentSession);
  const status = useAgentStore((s) => s.status);
  const message = useAgentStore((s) => s.statusMessage);

  if (!session) {
    return (
      <div className="border-b border-zinc-800 px-4 py-2 text-sm text-zinc-400">
        No active session
      </div>
    );
  }

  return (
    <div className="border-b border-zinc-800 px-4 py-2 text-sm">
      <div className="flex items-center justify-between text-zinc-300">
        <span>
          {isRunningStatus(status) ? "Running" : "Status"}: {message ?? status}
        </span>
        <span className="font-mono text-zinc-500">
          {formatElapsed(session.startedAt)}
        </span>
      </div>
      <div className="mt-1 truncate text-xs text-zinc-500">{session.cwd}</div>
    </div>
  );
}
