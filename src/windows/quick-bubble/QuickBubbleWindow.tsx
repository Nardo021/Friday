import { useAgentEventBridge } from "@/hooks/useAgentEventBridge";
import { Button } from "@/components/ui/button";
import { StatusPill } from "@/components/friday/StatusPill";
import { openChat, stopAgentSession } from "@/lib/tauri";
import { useAgentStore } from "@/state/useAgentStore";
import { useSessionStore } from "@/state/useSessionStore";
import { ApprovalCard } from "@/components/friday/ApprovalCard";
import { isRunningStatus } from "@/agent/status";

export function QuickBubbleWindow() {
  useAgentEventBridge();

  const status = useAgentStore((s) => s.status);
  const message = useAgentStore((s) => s.statusMessage);
  const session = useAgentStore((s) => s.currentSession);
  const pendingApproval = useAgentStore((s) => s.pendingApproval);
  const projects = useSessionStore((s) => s.projects);
  const project = projects.find((p) => p.id === session?.projectId);

  return (
    <div className="h-screen rounded-xl border border-zinc-700 bg-zinc-900/95 p-4 text-sm text-zinc-100 shadow-2xl backdrop-blur">
      <div className="mb-2 flex items-center justify-between">
        <span className="font-semibold">Friday</span>
        <StatusPill status={status} />
      </div>
      {project && (
        <div className="mb-1 text-zinc-400">Project: {project.name}</div>
      )}
      {session && (
        <div className="mb-2 truncate text-zinc-300">{session.title}</div>
      )}
      {message && <div className="mb-3 text-zinc-400">{message}</div>}

      {pendingApproval && (
        <div className="mb-3">
          <ApprovalCard
            approvalId={pendingApproval.approvalId}
            command={pendingApproval.command}
            risk={pendingApproval.risk}
          />
        </div>
      )}

      <div className="flex flex-wrap gap-2">
        <Button size="sm" onClick={() => openChat()}>
          Open Chat
        </Button>
        {session && isRunningStatus(status) && (
          <Button
            size="sm"
            variant="destructive"
            onClick={() => stopAgentSession(session.id)}
          >
            Stop
          </Button>
        )}
      </div>
    </div>
  );
}
