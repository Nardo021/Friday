import { Terminal } from "lucide-react";

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
      <CardContent className="flex flex-col gap-2 pt-4 text-sm">
        <div className="flex items-center justify-between">
          <span className="flex items-center gap-1.5 text-muted-foreground">
            <Terminal />
            Command
          </span>
          <Badge variant={risk === "high" ? "destructive" : "outline"}>
            {risk}
          </Badge>
        </div>
        <pre className="overflow-x-auto rounded-md bg-muted p-2 text-xs">
          {command}
        </pre>
      </CardContent>
    </Card>
  );
}
