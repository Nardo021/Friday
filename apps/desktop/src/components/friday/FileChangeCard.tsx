import { FilePenLine } from "lucide-react";

import { Card, CardContent } from "@/components/ui/card";
import type { FileAction } from "@friday/agent-core";

export function FileChangeCard({
  path,
  action,
}: {
  path: string;
  action: FileAction;
}) {
  return (
    <Card>
      <CardContent className="flex items-start gap-2 pt-4 text-sm">
        <FilePenLine className="mt-0.5 shrink-0 text-muted-foreground" />
        <span>
          <span className="text-muted-foreground">{action}</span>{" "}
          <span className="font-mono text-foreground">{path}</span>
        </span>
      </CardContent>
    </Card>
  );
}
