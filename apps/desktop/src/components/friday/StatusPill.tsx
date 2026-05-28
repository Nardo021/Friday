import type { FridaySessionStatus } from "@friday/agent-core";
import { STATUS_LABELS } from "@friday/agent-core";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

const variantMap: Record<
  FridaySessionStatus,
  "default" | "secondary" | "success" | "warning" | "danger"
> = {
  discovered: "secondary",
  idle: "secondary",
  starting: "default",
  thinking: "default",
  reading: "default",
  editing: "default",
  running_command: "warning",
  waiting_permission: "warning",
  testing: "default",
  done: "success",
  error: "danger",
  stopped: "secondary",
};

export function StatusPill({
  status,
  className,
}: {
  status: FridaySessionStatus;
  className?: string;
}) {
  return (
    <Badge variant={variantMap[status]} className={cn(className)}>
      {STATUS_LABELS[status]}
    </Badge>
  );
}
