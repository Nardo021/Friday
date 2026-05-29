import { Inbox } from "lucide-react";

import type { FridaySession } from "@friday/agent-core";

import { isRunningStatus } from "@friday/agent-core";



import { StatusPill } from "@/components/friday/StatusPill";

import { Badge } from "@/components/ui/badge";

import { Button } from "@/components/ui/button";

import {

  Empty,

  EmptyDescription,

  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";

import { Skeleton } from "@/components/ui/skeleton";

import { MotionStagger } from "@/components/friday/Motion";
import { cn } from "@/lib/utils";

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

      <div className="motion-feedback border-b border-border px-4 py-3 text-sm text-muted-foreground">

        No active session — start one below or select from the list.

      </div>

    );

  }



  return (

    <div className="motion-feedback border-b border-border px-4 py-3">

      <div className="flex items-center justify-between gap-3">

        <div className="min-w-0 flex-1">

          <div className="flex items-center gap-2">

            <h2 className="truncate font-medium text-foreground">{session.title}</h2>

            <StatusPill status={session.status} />

          </div>

          <p className="mt-1 truncate text-sm text-muted-foreground">

            {message ?? session.summary ?? session.prompt}

          </p>

        </div>

        <div className="shrink-0 text-right text-xs text-muted-foreground">

          <div>{isRunningStatus(session.status) ? "Running" : "Session"}</div>

          <div className="font-mono">{formatElapsed(session.startedAt)}</div>

        </div>

      </div>

      {session.repo?.localPath && (

        <div className="mt-1 truncate font-mono text-xs text-muted-foreground">

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

    <Button

      type="button"

      variant={active ? "secondary" : "outline"}

      aria-current={active ? "true" : undefined}

      onClick={onSelect}

      className={cn(

        "motion-hover h-auto w-full flex-col items-stretch gap-1 px-3 py-2 text-left text-sm font-normal",

        active && "border-foreground/25 bg-accent/50",

      )}

    >

      <div className="flex w-full items-center justify-between gap-2">

        <span className="truncate font-medium">{session.title}</span>

        <StatusPill status={session.status} />

      </div>

      <span className="w-full truncate text-xs text-muted-foreground">

        {session.ownership === "external"

          ? "External CLI · Observe only"

          : session.adapterId}

      </span>

    </Button>

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

      <Badge variant="outline" className="absolute top-2 right-2 text-[10px]">

        Cloud

      </Badge>

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

    <aside
      aria-label="Sessions"
      className="flex w-64 shrink-0 flex-col border-r border-border"
    >

      <div className="border-b border-border px-3 py-2 text-xs font-medium uppercase tracking-wide text-muted-foreground">

        Sessions

      </div>

      <MotionStagger className="flex flex-1 flex-col gap-2 overflow-y-auto p-3">

        {loading && sorted.length === 0 && (

          <div className="flex flex-col gap-2">

            <Skeleton className="h-14 w-full" />

            <Skeleton className="h-14 w-full" />

          </div>

        )}

        {!loading && sorted.length === 0 && (

          <Empty className="border-0 p-0">

            <EmptyHeader>
              <EmptyMedia variant="icon">
                <Inbox />
              </EmptyMedia>
              <EmptyTitle className="text-sm font-normal">No sessions yet</EmptyTitle>

              <EmptyDescription className="text-xs">
                Use <strong className="font-medium text-foreground">New task</strong> below
                to run your first agent, or open Quick Chat from the pet.
              </EmptyDescription>

            </EmptyHeader>

          </Empty>

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

      </MotionStagger>

    </aside>

  );

}

