import { useEffect } from "react";
import { Cat, LayoutDashboard, Settings } from "lucide-react";

import { AgentBadge } from "@/components/friday/AgentBadge";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent } from "@/components/ui/tabs";
import { useAgentEventBridge } from "@/hooks/useAgentEventBridge";
import { useFridayReady } from "@/hooks/useFridayReady";
import { usePanelNavigation } from "@/hooks/usePanelNavigation";
import { listenAgentEvents } from "@/lib/tauri";
import { cn } from "@/lib/utils";
import { usePanelPageStore } from "@/state/usePanelPageStore";
import { useActiveSession, useSessionStore } from "@/state/useSessionStore";
import { useSettingsStore } from "@/state/useSettingsStore";

import { AdaptersPage } from "@/windows/command-center/AdaptersPage";
import { ApprovalsPage } from "@/windows/command-center/ApprovalsPage";
import { DashboardPage } from "@/windows/command-center/DashboardPage";
import { IdeasPage } from "@/windows/command-center/IdeasPage";
import { LogsPage } from "@/windows/command-center/LogsPage";
import { ProjectsPage } from "@/windows/command-center/ProjectsPage";
import { SessionsPage } from "@/windows/command-center/SessionsPage";
import { SettingsPage } from "@/windows/command-center/SettingsPage";

import { MotionPage } from "@/components/friday/Motion";

import { PanelAgentView } from "./PanelAgentView";
import { PanelNav, type PanelPageId } from "./PanelNav";
import { panelPageDescription, panelPageTitle } from "./pageMeta";

function PanelPageContent({ page }: { page: PanelPageId }) {
  switch (page) {
    case "agent":
      return null;
    case "dashboard":
      return <DashboardPage />;
    case "sessions":
      return <SessionsPage />;
    case "projects":
      return <ProjectsPage />;
    case "ideas":
      return <IdeasPage />;
    case "approvals":
      return <ApprovalsPage />;
    case "logs":
      return <LogsPage />;
    case "adapters":
      return <AdaptersPage />;
    case "settings":
      return <SettingsPage />;
  }
}

export function FridayPanel() {
  const ready = useFridayReady();
  const page = usePanelPageStore((s) => s.page);
  const setPage = usePanelPageStore((s) => s.setPage);
  const bootstrap = useSessionStore((s) => s.bootstrap);
  const refreshSessions = useSessionStore((s) => s.refreshSessions);
  const loadSettings = useSettingsStore((s) => s.load);
  const session = useActiveSession();
  const { goToPage } = usePanelNavigation();

  useAgentEventBridge();

  useEffect(() => {
    if (!ready) return;
    void bootstrap();
    void loadSettings();
  }, [ready, bootstrap, loadSettings]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listenAgentEvents((event) => {
      if (
        event.type === "session.completed" ||
        event.type === "session.error"
      ) {
        refreshSessions();
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, [refreshSessions]);

  const headerTitle =
    page === "agent" && session ? session.title : panelPageTitle(page);
  const headerDescription =
    page === "agent" && session
      ? "Follow up, stop, or inspect output below."
      : panelPageDescription(page);

  return (
    <div className="flex h-screen bg-background text-foreground">
      <Tabs
        value={page}
        onValueChange={(v) => setPage(v as PanelPageId)}
        orientation="vertical"
        className="flex min-h-0 flex-1"
      >
        <aside
          aria-label="Friday panel navigation"
          className="flex w-48 shrink-0 flex-col border-r border-border p-3"
        >
          <h1 className="mb-0.5 flex items-center gap-2 px-2 text-base font-semibold">
            <Cat className="size-4 shrink-0" strokeWidth={1.6} aria-hidden />
            Friday
          </h1>
          <p className="mb-3 px-2 text-[11px] text-muted-foreground">Panel</p>
          <PanelNav />
        </aside>

        <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
          <header className="flex shrink-0 items-start justify-between gap-4 border-b border-border px-5 py-3">
            <div className="min-w-0 flex-1">
              <h2 className="truncate text-lg font-medium leading-snug">
                {headerTitle}
              </h2>
              <p className="mt-0.5 text-sm text-muted-foreground">
                {headerDescription}
              </p>
            </div>
            <div className="flex shrink-0 items-center gap-1">
              {page === "agent" && session && (
                <AgentBadge adapterId={session.adapterId} />
              )}
              {page === "agent" && (
                <>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="h-8"
                    onClick={() => goToPage("dashboard")}
                  >
                    <LayoutDashboard className="size-3.5" aria-hidden />
                    Dashboard
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="h-8"
                    onClick={() => goToPage("settings")}
                  >
                    <Settings className="size-3.5" aria-hidden />
                    Settings
                  </Button>
                </>
              )}
            </div>
          </header>

          <TabsContent
            value="agent"
            className="motion-tab-in mt-0 flex min-h-0 flex-1 flex-col overflow-hidden"
          >
            <PanelAgentView />
          </TabsContent>

          <div
            className={cn(
              "min-h-0 flex-1 overflow-y-auto px-5 py-5",
              page === "agent" && "hidden",
            )}
          >
            <MotionPage pageKey={page}>
              <PanelPageContent page={page} />
            </MotionPage>
          </div>
        </div>
      </Tabs>
    </div>
  );
}
