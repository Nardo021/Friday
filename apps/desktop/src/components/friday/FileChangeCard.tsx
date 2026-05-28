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
      <CardContent className="pt-4 text-sm">
        <span className="text-zinc-400">{action}</span>{" "}
        <span className="font-mono text-zinc-200">{path}</span>
      </CardContent>
    </Card>
  );
}
