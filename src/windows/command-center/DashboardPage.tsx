import { useEffect } from "react";

import { StatusPill } from "@/components/friday/StatusPill";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useAgentStore } from "@/state/useAgentStore";
import { useSessionStore } from "@/state/useSessionStore";

export function DashboardPage() {
  const session = useAgentStore((s) => s.currentSession);
  const status = useAgentStore((s) => s.status);
  const { sessions, projects, refreshSessions, refreshProjects } =
    useSessionStore();

  useEffect(() => {
    refreshSessions();
    refreshProjects();
  }, [refreshSessions, refreshProjects]);

  const recent = sessions.slice(0, 5);
  const activeProject = projects[0];

  return (
    <div className="grid gap-4 md:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>Current Session</CardTitle>
        </CardHeader>
        <CardContent>
          {session ? (
            <div className="space-y-2">
              <div>{session.title}</div>
              <StatusPill status={status} />
            </div>
          ) : (
            <p className="text-zinc-400">No active session</p>
          )}
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>Active Project</CardTitle>
        </CardHeader>
        <CardContent>
          {activeProject ? (
            <div>
              <div className="font-medium">{activeProject.name}</div>
              <div className="text-xs text-zinc-500">{activeProject.path}</div>
            </div>
          ) : (
            <p className="text-zinc-400">Add a project to get started</p>
          )}
        </CardContent>
      </Card>
      <Card className="md:col-span-2">
        <CardHeader>
          <CardTitle>Recent Sessions</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {recent.map((s) => (
              <div
                key={s.id}
                className="flex items-center justify-between rounded border border-zinc-800 px-3 py-2"
              >
                <span>{s.title}</span>
                <StatusPill status={s.status} />
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
