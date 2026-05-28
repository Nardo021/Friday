import type { FridaySession } from "@friday/agent-core";
import { isRunningStatus } from "@friday/agent-core";

import { StatusPill } from "@/components/friday/StatusPill";
import { formatElapsed } from "@/lib/time";
import {
  useActiveSession,
  useActiveStatusMessage,
  useSessionList,
  useSessionStore,
} from "@/state/useSessionStore";

export function CurrentStatusBar() {
  const session = useActiveSession();
  const message = useActiveStatusMessage();

  if (!session) {
    return (
      <div className="border-b border-zinc-800 px-4 py-3 text-sm text-zinc-400">
        No active session — start one below or select from the list.
      </div>
    );
  }

  return (
    <div className="border-b border-zinc-800 px-4 py-3">
      <div className="flex items-center justify-between gap-3">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <h2 className="truncate font-medium text-zinc-100">{session.title}</h2>
            <StatusPill status={session.status} />
          </div>
          <p className="mt-1 truncate text-sm text-zinc-400">
            {message ?? session.summary ?? session.prompt}
          </p>
        </div>
        <div className="shrink-0 text-right text-xs text-zinc-500">
          <div>{isRunningStatus(session.status) ? "Running" : "Session"}</div>
          <div className="font-mono">{formatElapsed(session.startedAt)}</div>
        </div>
      </div>
      {session.repo?.localPath && (
        <div className="mt-1 truncate font-mono text-xs text-zinc-600">
          {session.repo.localPath}
        </div>
      )}
    </div>
  );
}

export function SessionCard({
  session,
  active,
  onSelect,
}: {
  session: FridaySession;
  active: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onSelect}
      className={`w-full rounded-lg border px-3 py-2 text-left text-sm transition-colors ${
        active
          ? "border-indigo-500/50 bg-indigo-600/10"
          : "border-zinc-800 bg-zinc-900/50 hover:border-zinc-700"
      }`}
    >
      <div className="flex items-center justify-between gap-2">
        <span className="truncate font-medium text-zinc-200">{session.title}</span>
        <StatusPill status={session.status} />
      </div>
      <div className="mt-1 truncate text-xs text-zinc-500">
        {session.ownership === "external"
          ? "External CLI · Observe only"
          : session.adapterId}
      </div>
    </button>
  );
}

export function FridaySessionCard(props: {
  session: FridaySession;
  active: boolean;
  onSelect: () => void;
}) {
  if (props.session.ownership !== "friday") return null;
  return <SessionCard {...props} />;
}

export function ExternalSessionCard(props: {
  session: FridaySession;
  active: boolean;
  onSelect: () => void;
}) {
  if (props.session.ownership !== "external") return null;
  return <SessionCard {...props} />;
}

export function CloudSessionCard(props: {
  session: FridaySession;
  active: boolean;
  onSelect: () => void;
}) {
  if (props.session.type !== "cursor_cloud") return null;
  return (
    <div className="relative">
      <SessionCard {...props} />
      <span className="absolute right-2 top-2 rounded bg-sky-900/60 px-1.5 py-0.5 text-[10px] text-sky-200">
        Cloud
      </span>
    </div>
  );
}

export function ActiveSessionsList() {
  const sessions = useSessionList();
  const activeSessionId = useSessionStore((s) => s.activeSessionId);
  const selectActiveSession = useSessionStore((s) => s.selectActiveSession);
  const loading = useSessionStore((s) => s.loading);

  const sorted = [...sessions].sort(
    (a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime(),
  );

  return (
    <aside className="flex w-64 shrink-0 flex-col border-r border-zinc-800">
      <div className="border-b border-zinc-800 px-3 py-2 text-xs font-medium uppercase tracking-wide text-zinc-500">
        Sessions
      </div>
      <div className="flex-1 space-y-2 overflow-y-auto p-3">
        {loading && sorted.length === 0 && (
          <p className="text-sm text-zinc-500">Loading...</p>
        )}
        {!loading && sorted.length === 0 && (
          <p className="text-sm text-zinc-500">No sessions yet</p>
        )}
        {sorted.map((session) => {
          const props = {
            session,
            active: session.id === activeSessionId,
            onSelect: () => void selectActiveSession(session.id),
          };
          if (session.type === "cursor_cloud") {
            return <CloudSessionCard key={session.id} {...props} />;
          }
          if (session.ownership === "external") {
            return <ExternalSessionCard key={session.id} {...props} />;
          }
          return <FridaySessionCard key={session.id} {...props} />;
        })}
      </div>
    </aside>
  );
}
