import { useEffect, useState } from "react";
import { ExternalLink, FileText } from "lucide-react";
import { openPath } from "@tauri-apps/plugin-opener";
import { toast } from "sonner";

import { MotionStagger } from "@/components/friday/Motion";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { getLocalDataPath } from "@/lib/tauri";
import { invokeErrorMessage } from "@/lib/invokeError";
import { UX } from "@/lib/ux";
import { usePanelNavigation } from "@/hooks/usePanelNavigation";
import { useSessionList } from "@/state/useSessionStore";

export function LogsPage() {
  const sessions = useSessionList();
  const [dataPath, setDataPath] = useState<string | null>(null);
  const { goToAgent } = usePanelNavigation();

  useEffect(() => {
    void getLocalDataPath().then(setDataPath);
  }, []);

  const openDataFolder = async () => {
    if (!dataPath) return;
    try {
      await openPath(dataPath);
    } catch (e) {
      toast.error(invokeErrorMessage(e));
    }
  };

  const logsHint = dataPath ? `${dataPath}\\logs` : "…\\logs";

  return (
    <div className={UX.page}>
      <div className="flex flex-wrap items-center gap-2">
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="h-9"
          disabled={!dataPath}
          onClick={() => void openDataFolder()}
        >
          <ExternalLink data-icon="inline-start" />
          Open data folder
        </Button>
        <span className="font-mono text-xs text-muted-foreground break-all">
          {logsHint}
        </span>
      </div>

      {sessions.length === 0 ? (
        <Empty className="rounded-lg border border-dashed py-10">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <FileText />
            </EmptyMedia>
            <EmptyTitle className="text-sm font-normal">No session logs yet</EmptyTitle>
            <EmptyDescription className="text-xs">
              Logs appear after you run an agent. Open a session to inspect live
              output.
            </EmptyDescription>
          </EmptyHeader>
          <Button size="sm" className="mt-2" onClick={() => goToAgent()}>
            Go to agent
          </Button>
        </Empty>
      ) : (
        <MotionStagger className="flex flex-col gap-2">
          {sessions.map((s) => (
            <li key={s.id}>
              <Card>
                <CardContent className="flex items-center justify-between gap-3 py-3">
                  <span className="min-w-0 flex-1 font-mono text-xs">
                    <span className="text-muted-foreground">{s.id}.log</span>
                    <span className="mx-2 text-border">·</span>
                    <span className="text-foreground">{s.title}</span>
                  </span>
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => goToAgent(s.id)}
                  >
                    Open session
                  </Button>
                </CardContent>
              </Card>
            </li>
          ))}
        </MotionStagger>
      )}
    </div>
  );
}
