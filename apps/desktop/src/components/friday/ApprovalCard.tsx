import { Check, ShieldAlert, X } from "lucide-react";

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
    <Card className="border-destructive/40">
      <CardContent className="flex flex-col gap-3 pt-4">
        <div className="flex items-center justify-between">
          <span className="flex items-center gap-1.5 font-medium">
            <ShieldAlert />
            High-risk command
          </span>
          <Badge variant={risk === "high" ? "destructive" : "outline"}>
            {risk}
          </Badge>
        </div>
        <p className="text-xs text-muted-foreground">
          Cursor may already be running this command. Acknowledge, or stop the
          Friday-owned session.
        </p>
        {command && (
          <pre className="overflow-x-auto rounded-md bg-muted p-2 text-xs">
            {command}
          </pre>
        )}
        <div className="flex gap-2">
          <Button size="sm" onClick={() => void approveCommand(approvalId)}>
            <Check data-icon="inline-start" />
            Acknowledge
          </Button>
          <Button
            size="sm"
            variant="destructive"
            onClick={() => void rejectCommand(approvalId)}
          >
            <X data-icon="inline-start" />
            Stop session
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
