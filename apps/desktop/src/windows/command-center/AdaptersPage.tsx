import { useSettingsStore } from "@/state/useSettingsStore";
import { Badge } from "@/components/ui/badge";

export function AdaptersPage() {
  const adapters = useSettingsStore((s) => s.adapters);

  return (
    <div className="grid gap-3 md:grid-cols-2">
      {adapters.map((a) => (
        <div
          key={a.id}
          className="rounded-lg border border-zinc-800 p-4"
        >
          <div className="flex items-center justify-between">
            <span className="font-medium">{a.name}</span>
            <Badge variant={a.available ? "success" : "secondary"}>
              {a.available ? "Available" : "Coming soon"}
            </Badge>
          </div>
          <div className="mt-2 text-xs text-zinc-500">{a.id}</div>
        </div>
      ))}
    </div>
  );
}
