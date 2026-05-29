import { KeyRound } from "lucide-react";

import { MotionStagger } from "@/components/friday/Motion";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { usePanelNavigation } from "@/hooks/usePanelNavigation";
import { UX } from "@/lib/ux";
import { useSettingsStore } from "@/state/useSettingsStore";

export function AdaptersPage() {
  const adapters = useSettingsStore((s) => s.adapters);
  const apiKeyConfigured = useSettingsStore(
    (s) => s.settings.cursor.apiKeyConfigured,
  );
  const { goToPage } = usePanelNavigation();

  return (
    <div className={UX.page}>
      {!apiKeyConfigured && (
        <div className="motion-item-in flex flex-wrap items-center justify-between gap-3 rounded-lg border border-dashed border-amber-500/40 bg-amber-500/5 px-4 py-3">
          <p className="text-sm text-muted-foreground">
            Cloud agent adapter needs a Cursor API key in Settings.
          </p>
          <Button size="sm" variant="secondary" onClick={() => goToPage("settings")}>
            <KeyRound data-icon="inline-start" />
            Add API key
          </Button>
        </div>
      )}

      <MotionStagger className="grid gap-3 md:grid-cols-2">
        {adapters.map((a) => (
          <li key={a.id}>
            <Card>
              <CardHeader className="flex flex-row items-center justify-between gap-2 pb-2">
                <CardTitle className="text-base">{a.name}</CardTitle>
                <Badge variant={a.available ? "default" : "secondary"}>
                  {a.available ? "Ready" : "Unavailable"}
                </Badge>
              </CardHeader>
              <CardContent className="pt-0">
                <p className="font-mono text-xs text-muted-foreground">{a.id}</p>
                {!a.available && a.id.includes("cloud") && !apiKeyConfigured && (
                  <p className="mt-2 text-xs text-muted-foreground">
                    Configure API key in Settings to enable.
                  </p>
                )}
              </CardContent>
            </Card>
          </li>
        ))}
      </MotionStagger>
    </div>
  );
}
