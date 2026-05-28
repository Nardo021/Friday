import { AgentBadge, ProjectBadge } from "@/components/friday/AgentBadge";
import { StatusPill } from "@/components/friday/StatusPill";
import { useAgentStore } from "@/state/useAgentStore";
import { useSessionStore } from "@/state/useSessionStore";

export function ChatHeader() {
  const session = useAgentStore((s) => s.currentSession);
  const projects = useSessionStore((s) => s.projects);
  const project = projects.find((p) => p.id === session?.projectId);

  return (
    <div className="border-b border-zinc-800 px-4 py-3">
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold text-zinc-100">Friday</h1>
        {session && <StatusPill status={session.status} />}
      </div>
      <div className="mt-2 flex flex-wrap gap-2">
        {session && <AgentBadge adapterId={session.adapterId} />}
        {project && <ProjectBadge name={project.name} />}
      </div>
    </div>
  );
}
