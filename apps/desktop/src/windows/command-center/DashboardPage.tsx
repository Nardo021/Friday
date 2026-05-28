import { useEffect, useMemo } from "react";

import { isRunningStatus } from "@friday/agent-core";

import { StatusPill } from "@/components/friday/StatusPill";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  useActiveSession,
  useActiveStatusMessage,
  useSessionList,
  useSessionStore,
} from "@/state/useSessionStore";

export function DashboardPage() {
  const session = useActiveSession();
  const statusMessage = useActiveStatusMessage();
  const sessions = useSessionList();
  const { projects, refreshSessions, refreshProjects } = useSessionStore();

  useEffect(() => {
    refreshSessions();
    refreshProjects();
  }, [refreshSessions, refreshProjects]);

  const recent = sessions.slice(0, 5);
  const external = sessions.filter((s) => s.ownership === "external");
  const completedToday = useMemo(() => {
    const today = new Date().toDateString();
    return sessions.filter(
      (s) =>
        s.status === "done" &&
        s.completedAt &&
        new Date(s.completedAt).toDateString() === today,
    ).length;
  }, [sessions]);

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
              <StatusPill status={session.status} />
              {statusMessage && (
                <p className="text-sm text-zinc-400">{statusMessage}</p>
              )}
            </div>
          ) : (
            <p className="text-zinc-400">No active session</p>
          )}
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>Summary</CardTitle>
        </CardHeader>
        <CardContent className="space-y-1 text-sm text-zinc-400">
          <p>Completed today: {completedToday}</p>
          <p>External CLI sessions: {external.length}</p>
          <p>
            Running:{" "}
            {sessions.filter((s) => isRunningStatus(s.status)).length}
          </p>
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
      <Card>
        <CardHeader>
          <CardTitle>External CLI</CardTitle>
        </CardHeader>
        <CardContent>
          {external.length === 0 ? (
            <p className="text-sm text-zinc-500">None detected</p>
          ) : (
            <ul className="space-y-1 text-sm">
              {external.map((s) => (
                <li key={s.id} className="flex justify-between gap-2">
                  <span className="truncate">{s.title}</span>
                  <StatusPill status={s.status} />
                </li>
              ))}
            </ul>
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
