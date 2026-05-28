import { useSessionStore } from "@/state/useSessionStore";

export function LogsPage() {
  const sessions = useSessionStore((s) => s.sessions);

  return (
    <div className="space-y-2 text-sm">
      <p className="text-zinc-400">
        Session logs are stored locally in the app data directory.
      </p>
      {sessions.map((s) => (
        <div
          key={s.id}
          className="rounded border border-zinc-800 px-3 py-2 font-mono text-xs"
        >
          {s.id}.log — {s.title}
        </div>
      ))}
    </div>
  );
}
