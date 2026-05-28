import { Badge } from "@/components/ui/badge";

export function AgentBadge({ adapterId }: { adapterId: string }) {
  return <Badge variant="secondary">{adapterId}</Badge>;
}

export function ProjectBadge({ name }: { name: string }) {
  return <Badge>{name}</Badge>;
}
