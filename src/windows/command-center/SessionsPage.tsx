import { useEffect } from "react";

import { StatusPill } from "@/components/friday/StatusPill";
import { useSessionStore } from "@/state/useSessionStore";
import { formatTime } from "@/lib/time";

export function SessionsPage() {
  const { sessions, refreshSessions } = useSessionStore();

  useEffect(() => {
    refreshSessions();
  }, [refreshSessions]);

  return (
    <div className="overflow-hidden rounded-lg border border-zinc-800">
      <table className="w-full text-sm">
        <thead className="bg-zinc-900 text-left text-zinc-400">
          <tr>
            <th className="px-4 py-2">Status</th>
            <th className="px-4 py-2">Title</th>
            <th className="px-4 py-2">Created</th>
          </tr>
        </thead>
        <tbody>
          {sessions.map((s) => (
            <tr key={s.id} className="border-t border-zinc-800">
              <td className="px-4 py-2">
                <StatusPill status={s.status} />
              </td>
              <td className="px-4 py-2">{s.title}</td>
              <td className="px-4 py-2 text-zinc-500">
                {formatTime(s.createdAt)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
