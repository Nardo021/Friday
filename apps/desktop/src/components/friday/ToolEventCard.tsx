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
      <CardContent className="pt-4 text-sm">
        <span className="text-zinc-400">Tool</span>{" "}
        <span className="font-medium text-indigo-300">{toolName}</span>
        <div className="mt-1 text-zinc-300">{title}</div>
      </CardContent>
    </Card>
  );
}
