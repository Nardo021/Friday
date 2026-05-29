import { MotionItem } from "@/components/friday/Motion";
import { ApprovalCard } from "@/components/friday/ApprovalCard";
import { CommandBlock } from "@/components/friday/CommandBlock";
import { FileChangeCard } from "@/components/friday/FileChangeCard";
import { ToolEventCard } from "@/components/friday/ToolEventCard";
import { Card, CardContent } from "@/components/ui/card";
import { formatTime } from "@/lib/time";
import {
  useActivePendingApproval,
  useActiveTimeline,
} from "@/state/useSessionStore";

export function SessionTimeline() {
  const timeline = useActiveTimeline();
  const pendingApproval = useActivePendingApproval();

  return (
    <div className="motion-stagger flex flex-1 flex-col gap-3 overflow-y-auto px-4 py-3">
      {timeline.map((item, idx) => {
        const key = `${item.kind}-${item.timestamp}-${idx}`;
        switch (item.kind) {
          case "message":
            return (
              <MotionItem key={key} className="text-sm">
                <div className="mb-1 text-xs text-muted-foreground">
                  {item.role} · {formatTime(item.timestamp)}
                </div>
                <div
                  className={
                    item.role === "user"
                      ? "rounded-lg border border-border bg-muted/80 px-3 py-2 text-foreground"
                      : "rounded-lg bg-muted px-3 py-2 text-foreground"
                  }
                >
                  {item.content}
                </div>
              </MotionItem>
            );
          case "tool":
            return (
              <MotionItem key={key}>
              <ToolEventCard
                toolName={item.toolName}
                title={item.title}
              />
              </MotionItem>
            );
          case "command":
            return (
              <MotionItem key={key}>
              <CommandBlock
                command={item.command}
                risk={item.risk}
              />
              </MotionItem>
            );
          case "file":
            return (
              <MotionItem key={key}>
              <FileChangeCard
                path={item.path}
                action={item.action}
              />
              </MotionItem>
            );
          case "approval":
            return (
              <MotionItem key={key}>
              <ApprovalCard
                approvalId={item.approvalId}
                command={item.command}
                risk={item.risk}
              />
              </MotionItem>
            );
          case "status":
            if (!item.message) return null;
            return (
              <MotionItem key={key} className="text-xs italic text-muted-foreground">
                {item.message}
              </MotionItem>
            );
          case "artifact":
            return (
              <MotionItem key={key}>
              <Card>
                <CardContent className="py-3 text-sm">
                  <span className="text-muted-foreground">Artifact</span> {item.title}
                </CardContent>
              </Card>
              </MotionItem>
            );
          case "pr":
            return (
              <MotionItem key={key}>
              <Card>
                <CardContent className="py-3 text-sm">
                  <a
                    href={item.prUrl}
                    className="text-foreground underline underline-offset-2"
                    target="_blank"
                    rel="noreferrer"
                  >
                    {item.prUrl}
                  </a>
                </CardContent>
              </Card>
              </MotionItem>
            );
          default:
            return null;
        }
      })}

      {pendingApproval && (
        <MotionItem>
          <ApprovalCard
            approvalId={pendingApproval.approvalId}
            command={pendingApproval.command}
            risk={pendingApproval.risk}
          />
        </MotionItem>
      )}
    </div>
  );
}
