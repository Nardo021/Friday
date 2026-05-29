import { useEffect, useMemo } from "react";
import {
  Bot,
  FolderKanban,
  MessageSquare,
  Settings,
} from "lucide-react";

import { isRunningStatus } from "@friday/agent-core";

import { QuickLinkButton } from "@/components/friday/QuickLinkButton";
import { StatusPill } from "@/components/friday/StatusPill";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { usePanelNavigation } from "@/hooks/usePanelNavigation";
import { openQuickBubble } from "@/lib/tauri";
import { UX } from "@/lib/ux";
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
  const { projects, selectedProjectId, refreshSessions, refreshProjects } =
    useSessionStore();
  const { goToAgent, goToPage } = usePanelNavigation();

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

  const runningCount = sessions.filter((s) => isRunningStatus(s.status)).length;
  const activeProject = selectedProjectId
    ? projects.find((p) => p.id === selectedProjectId)
    : undefined;

  return (
    <div className={UX.page} data-od-id="dashboard">
      <div className="motion-stagger flex flex-wrap gap-2">
        <QuickLinkButton
          icon={Bot}
          label="New task"
          onClick={() => goToAgent()}
        />
        <QuickLinkButton
          icon={MessageSquare}
          label="Quick chat"
          onClick={() => void openQuickBubble()}
        />
        <QuickLinkButton
          icon={FolderKanban}
          label="Projects"
          onClick={() => goToPage("projects")}
        />
        <QuickLinkButton
          icon={Settings}
          label="Settings"
          onClick={() => goToPage("settings")}
        />
      </div>

      <section
        data-od-id="dashboard-live"
        className="flex flex-wrap items-baseline gap-x-8 gap-y-2 border-b border-border pb-6"
      >
        <p className="text-sm text-muted-foreground">
          <span className="font-mono text-2xl text-foreground tabular-nums">
            {runningCount}
          </span>{" "}
          running
        </p>
        <p className="text-sm text-muted-foreground">
          <span className="font-mono text-foreground tabular-nums">
            {completedToday}
          </span>{" "}
          done today
        </p>
        <p className="text-sm text-muted-foreground">
          <span className="font-mono text-foreground tabular-nums">
            {external.length}
          </span>{" "}
          external CLI
        </p>
      </section>

      <div className="motion-stagger grid gap-4 md:grid-cols-2">
        <Card data-od-id="dashboard-current-session">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Current session</CardTitle>
          </CardHeader>
          <CardContent>
            {session ? (
              <div className="flex flex-col gap-3">
                <div className="flex flex-col gap-2">
                  <p className="font-medium leading-snug">{session.title}</p>
                  <StatusPill status={session.status} />
                  {statusMessage && (
                    <p className="text-sm text-muted-foreground">{statusMessage}</p>
                  )}
                </div>
                <Button
                  size="sm"
                  variant="secondary"
                  className="w-fit"
                  onClick={() => goToAgent(session.id)}
                >
                  Open in agent
                </Button>
              </div>
            ) : (
              <div className="flex flex-col gap-2">
                <p className="text-sm text-muted-foreground">
                  No active session — start one from Agent or Quick Chat.
                </p>
                <Button size="sm" onClick={() => goToAgent()}>
                  Go to agent
                </Button>
              </div>
            )}
          </CardContent>
        </Card>

        <Card data-od-id="dashboard-project">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Working folder</CardTitle>
          </CardHeader>
          <CardContent>
            {activeProject ? (
              <div className="flex flex-col gap-3">
                <div>
                  <p className="font-medium">{activeProject.name}</p>
                  <p className="font-mono text-xs text-muted-foreground break-all">
                    {activeProject.path}
                  </p>
                </div>
                <Button
                  size="sm"
                  variant="secondary"
                  className="w-fit"
                  onClick={() => goToPage("projects")}
                >
                  Manage projects
                </Button>
              </div>
            ) : (
              <div className="flex flex-col gap-2">
                <p className="text-sm text-muted-foreground">
                  Chat works without a repo. Link a folder here when you want
                  agents scoped to a codebase.
                </p>
                <Button size="sm" variant="secondary" onClick={() => goToPage("projects")}>
                  Link a repo (optional)
                </Button>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <section data-od-id="dashboard-recent" className={UX.section}>
        <div className="flex items-center justify-between gap-2">
          <h3 className="text-sm font-medium">Recent sessions</h3>
          {sessions.length > 0 && (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="h-8 text-xs"
              onClick={() => goToPage("sessions")}
            >
              View all
            </Button>
          )}
        </div>
        {recent.length === 0 ? (
          <p className="text-sm text-muted-foreground">Nothing recent yet.</p>
        ) : (
          <ul className="divide-y divide-border rounded-lg border">
            {recent.map((s) => (
              <li key={s.id}>
                <button
                  type="button"
                  className="flex w-full items-center justify-between gap-3 px-4 py-3 text-left text-sm transition-colors hover:bg-accent/50"
                  onClick={() => goToAgent(s.id)}
                >
                  <span className="min-w-0 truncate">{s.title}</span>
                  <StatusPill status={s.status} />
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>

      {external.length > 0 && (
        <section data-od-id="dashboard-external" className={UX.section}>
          <h3 className="text-sm font-medium">External CLI</h3>
          <ul className="motion-stagger divide-y divide-border rounded-lg border">
            {external.map((s) => (
              <li key={s.id}>
                <button
                  type="button"
                  className="flex w-full items-center justify-between gap-3 px-4 py-3 text-left text-sm transition-colors hover:bg-accent/50"
                  onClick={() => goToAgent(s.id)}
                >
                  <span className="min-w-0 truncate">{s.title}</span>
                  <StatusPill status={s.status} />
                </button>
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}
