import { Wrench } from "lucide-react";

import { Card, CardContent } from "@/components/ui/card";

export function ToolEventCard({
  toolName,
  title,
}: {
  toolName: string;
  title: string;
}) {
  return (
    <Card>
      <CardContent className="flex gap-2 pt-4 text-sm">
        <Wrench className="mt-0.5 shrink-0 text-muted-foreground" />
        <div>
          <span className="text-muted-foreground">Tool</span>{" "}
          <span className="font-medium text-primary">{toolName}</span>
          <div className="mt-1 text-foreground">{title}</div>
        </div>
      </CardContent>
    </Card>
  );
}
