import type { AgentStatus } from "@/agent/types";
import { STATUS_LABELS } from "@/agent/status";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

const variantMap: Record<
  AgentStatus,
  "default" | "secondary" | "success" | "warning" | "danger"
> = {
  idle: "secondary",
  starting: "default",
  thinking: "default",
  reading: "default",
  editing: "default",
  running_command: "warning",
  waiting_approval: "warning",
  testing: "default",
  paused: "secondary",
  completed: "success",
  error: "danger",
  cancelled: "secondary",
};

export function StatusPill({
  status,
  className,
}: {
  status: AgentStatus;
  className?: string;
}) {
  return (
    <Badge variant={variantMap[status]} className={cn(className)}>
      {STATUS_LABELS[status]}
    </Badge>
  );
}
