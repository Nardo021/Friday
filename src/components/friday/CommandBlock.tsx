import type { RiskLevel } from "@/agent/types";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";

const riskVariant: Record<
  RiskLevel,
  "secondary" | "warning" | "danger"
> = {
  low: "secondary",
  medium: "warning",
  high: "danger",
};

export function CommandBlock({
  command,
  risk,
}: {
  command: string;
  risk: RiskLevel;
}) {
  return (
    <Card>
      <CardContent className="space-y-2 pt-4">
        <div className="flex items-center justify-between">
          <span className="text-xs uppercase tracking-wide text-zinc-400">
            Command
          </span>
          <Badge variant={riskVariant[risk]}>{risk}</Badge>
        </div>
        <pre className="overflow-x-auto rounded bg-zinc-950 p-2 text-xs text-zinc-200">
          {command}
        </pre>
      </CardContent>
    </Card>
  );
}
