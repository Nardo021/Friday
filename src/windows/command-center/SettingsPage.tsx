import { useEffect } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useSettingsStore } from "@/state/useSettingsStore";

export function SettingsPage() {
  const { settings, loaded, load, update } = useSettingsStore();

  useEffect(() => {
    if (!loaded) load();
  }, [loaded, load]);

  if (!loaded) return <div className="text-zinc-400">Loading settings...</div>;

  return (
    <div className="max-w-xl space-y-6">
      <section>
        <h3 className="mb-2 font-medium">Appearance</h3>
        <label className="mb-2 block text-sm text-zinc-400">Pet scale</label>
        <Input
          type="number"
          step="0.1"
          min="0.5"
          max="2"
          value={settings.appearance.petScale}
          onChange={(e) =>
            update({
              ...settings,
              appearance: {
                ...settings.appearance,
                petScale: Number(e.target.value),
              },
            })
          }
        />
      </section>

      <section>
        <h3 className="mb-2 font-medium">Security</h3>
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={settings.security.requireApprovalForHighRiskCommands}
            onChange={(e) =>
              update({
                ...settings,
                security: {
                  ...settings.security,
                  requireApprovalForHighRiskCommands: e.target.checked,
                },
              })
            }
          />
          Require approval for high-risk commands
        </label>
        <label className="mt-2 flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={settings.security.redactSecrets}
            onChange={(e) =>
              update({
                ...settings,
                security: {
                  ...settings.security,
                  redactSecrets: e.target.checked,
                },
              })
            }
          />
          Redact secrets in logs
        </label>
      </section>

      <section>
        <h3 className="mb-2 font-medium">Cursor CLI</h3>
        <Input
          placeholder="Executable path (optional)"
          value={settings.cursor.executablePath ?? ""}
          onChange={(e) =>
            update({
              ...settings,
              cursor: {
                ...settings.cursor,
                executablePath: e.target.value || undefined,
              },
            })
          }
        />
      </section>

      <Button onClick={() => update(settings)}>Save Settings</Button>
    </div>
  );
}
