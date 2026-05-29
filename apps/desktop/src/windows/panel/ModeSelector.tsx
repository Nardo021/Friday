import type { AgentMode } from "@friday/agent-core";

import { Alert, AlertDescription } from "@/components/ui/alert";
import { Field, FieldLabel, FieldSet, FieldLegend } from "@/components/ui/field";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils";
import { useSettingsStore } from "@/state/useSettingsStore";

const MODES: { id: AgentMode; label: string }[] = [
  { id: "cloud_agent", label: "Cursor API" },
  { id: "local_cli", label: "Local CLI" },
];

export function ModeSelector({
  value,
  onChange,
}: {
  value: AgentMode;
  onChange: (mode: AgentMode) => void;
}) {
  const apiKeyConfigured = useSettingsStore(
    (s) => s.settings.cursor.apiKeyConfigured ?? false,
  );

  return (
    <FieldSet>
      <FieldLegend variant="label">Mode</FieldLegend>
      <p className="text-xs text-muted-foreground">
        <strong className="font-medium text-foreground">Cursor API</strong> uses your
        dashboard key. <strong className="font-medium text-foreground">Local CLI</strong>{" "}
        runs <code className="text-[10px]">cursor-agent</code> on this PC.
      </p>
      <Field>
        <FieldLabel className="sr-only">Mode</FieldLabel>
        <ToggleGroup
          type="single"
          variant="outline"
          size="sm"
          value={value}
          onValueChange={(v) => v && onChange(v as AgentMode)}
          className="flex-wrap"
        >
          {MODES.map((mode) => {
            const enabled =
              mode.id === "local_cli" ||
              (mode.id === "cloud_agent" && apiKeyConfigured);
            return (
              <ToggleGroupItem
                key={mode.id}
                value={mode.id}
                disabled={!enabled}
                className={cn(
                  "text-xs",
                  mode.id === "cloud_agent" &&
                    enabled &&
                    "data-[state=on]:ring-1 data-[state=on]:ring-foreground/20",
                )}
              >
                {mode.label}
                {!enabled && mode.id === "cloud_agent" && " (needs API key)"}
              </ToggleGroupItem>
            );
          })}
        </ToggleGroup>
      </Field>
      {value === "cloud_agent" && !apiKeyConfigured && (
        <Alert variant="default" className="py-2">
          <AlertDescription className="text-xs">
            Save a Cursor API key in Settings first, then select Cursor API mode.
          </AlertDescription>
        </Alert>
      )}
    </FieldSet>
  );
}
