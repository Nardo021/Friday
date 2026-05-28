import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import type { RiskLevel } from "@friday/agent-core";

export function CommandBlock({
  command,
  risk,
}: {
  command: string;
  risk: RiskLevel;
}) {
  return (
    <Card>
      <CardContent className="space-y-2 pt-4 text-sm">
        <div className="flex items-center justify-between">
          <span className="text-zinc-400">Command</span>
          <Badge variant={risk === "high" ? "danger" : "warning"}>{risk}</Badge>
        </div>
        <pre className="overflow-x-auto rounded bg-zinc-950 p-2 text-xs text-zinc-200">
          {command}
        </pre>
      </CardContent>
    </Card>
  );
}
