import { ApprovalCard } from "@/components/friday/ApprovalCard";
import { CommandBlock } from "@/components/friday/CommandBlock";
import { FileChangeCard } from "@/components/friday/FileChangeCard";
import { ToolEventCard } from "@/components/friday/ToolEventCard";
import { formatTime } from "@/lib/time";
import {
  useActivePendingApproval,
  useActiveTimeline,
} from "@/state/useSessionStore";

export function SessionTimeline() {
  const timeline = useActiveTimeline();
  const pendingApproval = useActivePendingApproval();

  return (
    <div className="flex-1 space-y-3 overflow-y-auto px-4 py-3">
      {timeline.map((item, idx) => {
        const key = `${item.kind}-${item.timestamp}-${idx}`;
        switch (item.kind) {
          case "message":
            return (
              <div key={key} className="text-sm">
                <div className="mb-1 text-xs text-zinc-500">
                  {item.role} · {formatTime(item.timestamp)}
                </div>
                <div
                  className={
                    item.role === "user"
                      ? "rounded-lg bg-indigo-600/20 px-3 py-2 text-zinc-100"
                      : "rounded-lg bg-zinc-800/80 px-3 py-2 text-zinc-200"
                  }
                >
                  {item.content}
                </div>
              </div>
            );
          case "tool":
            return (
              <ToolEventCard
                key={key}
                toolName={item.toolName}
                title={item.title}
              />
            );
          case "command":
            return (
              <CommandBlock
                key={key}
                command={item.command}
                risk={item.risk}
              />
            );
          case "file":
            return (
              <FileChangeCard
                key={key}
                path={item.path}
                action={item.action}
              />
            );
          case "approval":
            return (
              <ApprovalCard
                key={key}
                approvalId={item.approvalId}
                command={item.command}
                risk={item.risk}
              />
            );
          case "status":
            if (!item.message) return null;
            return (
              <div key={key} className="text-xs italic text-zinc-500">
                {item.message}
              </div>
            );
          case "artifact":
            return (
              <div key={key} className="rounded border border-zinc-800 px-3 py-2 text-sm">
                <span className="text-zinc-400">Artifact</span> {item.title}
              </div>
            );
          case "pr":
            return (
              <div key={key} className="rounded border border-zinc-800 px-3 py-2 text-sm">
                <a
                  href={item.prUrl}
                  className="text-indigo-300 hover:underline"
                  target="_blank"
                  rel="noreferrer"
                >
                  {item.prUrl}
                </a>
              </div>
            );
          default:
            return null;
        }
      })}

      {pendingApproval && (
        <ApprovalCard
          approvalId={pendingApproval.approvalId}
          command={pendingApproval.command}
          risk={pendingApproval.risk}
        />
      )}
    </div>
  );
}
