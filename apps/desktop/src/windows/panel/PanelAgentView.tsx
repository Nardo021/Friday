import { ActionsBar } from "./ActionsBar";
import { ActiveSessionsList } from "./ActiveSessionsList";
import { ChatPortal } from "./ChatPortal";
import { CurrentStatusBar } from "./CurrentStatusBar";
import { SessionTimeline } from "./SessionTimeline";

/** Live agent work: sessions, timeline, new task, follow-up. */
export function PanelAgentView() {
  return (
    <>
      <CurrentStatusBar />
      <div className="flex min-h-0 flex-1">
        <ActiveSessionsList />
        <main id="panel-main" className="flex min-w-0 flex-1 flex-col">
          <SessionTimeline />
          <ChatPortal />
          <ActionsBar />
        </main>
      </div>
    </>
  );
}
