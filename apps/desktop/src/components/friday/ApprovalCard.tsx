import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import type { RiskLevel } from "@friday/agent-core";
import { approveCommand, rejectCommand } from "@/lib/tauri";

export function ApprovalCard({
  approvalId,
  command,
  risk,
}: {
  approvalId: string;
  command?: string;
  risk: RiskLevel;
}) {
  return (
    <Card className="border-amber-700/50">
      <CardContent className="space-y-3 pt-4">
        <div className="flex items-center justify-between">
          <span className="font-medium text-amber-200">Approval required</span>
          <Badge variant={risk === "high" ? "danger" : "warning"}>{risk}</Badge>
        </div>
        {command && (
          <pre className="overflow-x-auto rounded bg-zinc-950 p-2 text-xs">
            {command}
          </pre>
        )}
        <div className="flex gap-2">
          <Button
            size="sm"
            onClick={() => approveCommand(approvalId)}
          >
            Approve
          </Button>
          <Button
            size="sm"
            variant="destructive"
            onClick={() => rejectCommand(approvalId)}
          >
            Reject
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
