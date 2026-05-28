import { useEffect, useState } from "react";

import type { AgentMode } from "@friday/agent-core";

import { listAdapters } from "@/lib/tauri";

const MODES: { id: AgentMode; label: string }[] = [
  { id: "local_cli", label: "Local CLI" },
  { id: "sdk_local", label: "SDK Local" },
  { id: "cloud_agent", label: "Cloud Agent" },
];

export function ModeSelector({
  value,
  onChange,
}: {
  value: AgentMode;
  onChange: (mode: AgentMode) => void;
}) {
  const [cloudAvailable, setCloudAvailable] = useState(false);

  useEffect(() => {
    void listAdapters().then((adapters) => {
      const cloud = adapters.find((a) => a.id === "cursor-cloud-agent");
      setCloudAvailable(cloud?.available ?? false);
    });
  }, []);

  return (
    <div className="space-y-1">
      <label className="text-xs font-medium text-zinc-500">Mode</label>
      <div className="flex flex-wrap gap-2">
        {MODES.map((mode) => {
          const enabled =
            mode.id === "local_cli" ||
            (mode.id === "cloud_agent" && cloudAvailable);
          return (
            <button
              key={mode.id}
              type="button"
              disabled={!enabled}
              onClick={() => enabled && onChange(mode.id)}
              className={`rounded-md border px-3 py-1.5 text-xs ${
                value === mode.id
                  ? "border-indigo-500 bg-indigo-600/20 text-indigo-200"
                  : enabled
                    ? "border-zinc-700 text-zinc-300 hover:border-zinc-600"
                    : "cursor-not-allowed border-zinc-800 text-zinc-600"
              }`}
            >
              {mode.label}
              {!enabled && mode.id !== "local_cli" && " (soon)"}
            </button>
          );
        })}
      </div>
      {value === "cloud_agent" && !cloudAvailable && (
        <p className="text-xs text-amber-500/80">
          Configure a Cursor API key in Settings to use Cloud Agent.
        </p>
      )}
    </div>
  );
}
