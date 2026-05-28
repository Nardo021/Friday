import { useEffect, useMemo, useState } from "react";

import { StatusPill } from "@/components/friday/StatusPill";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { deleteSession, exportSessionMarkdown } from "@/lib/tauri";
import { formatTime } from "@/lib/time";
import { useSessionList, useSessionStore } from "@/state/useSessionStore";

export function SessionsPage() {
  const sessions = useSessionList();
  const refreshSessions = useSessionStore((s) => s.refreshSessions);
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState("all");

  useEffect(() => {
    refreshSessions();
  }, [refreshSessions]);

  const filtered = useMemo(() => {
    return sessions.filter((s) => {
      if (statusFilter !== "all" && s.status !== statusFilter) return false;
      if (!query.trim()) return true;
      const q = query.toLowerCase();
      return (
        s.title.toLowerCase().includes(q) ||
        s.type.toLowerCase().includes(q) ||
        (s.repo?.name?.toLowerCase().includes(q) ?? false)
      );
    });
  }, [sessions, query, statusFilter]);

  const exportMd = async (id: string) => {
    const md = await exportSessionMarkdown(id);
    await navigator.clipboard.writeText(md);
    window.alert("Session markdown copied to clipboard.");
  };

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap gap-2">
        <Input
          placeholder="Search sessions…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="max-w-xs"
        />
        <select
          className="rounded-md border border-zinc-700 bg-zinc-900 px-3 py-2 text-sm"
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
        >
          <option value="all">All statuses</option>
          <option value="running_command">Running</option>
          <option value="done">Done</option>
          <option value="stopped">Stopped</option>
          <option value="error">Error</option>
        </select>
      </div>

      <div className="overflow-hidden rounded-lg border border-zinc-800">
        <table className="w-full text-sm">
          <thead className="bg-zinc-900 text-left text-zinc-400">
            <tr>
              <th className="px-4 py-2">Status</th>
              <th className="px-4 py-2">Title</th>
              <th className="px-4 py-2">Type</th>
              <th className="px-4 py-2">Created</th>
              <th className="px-4 py-2">Actions</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((s) => (
              <tr key={s.id} className="border-t border-zinc-800">
                <td className="px-4 py-2">
                  <StatusPill status={s.status} />
                </td>
                <td className="px-4 py-2">{s.title}</td>
                <td className="px-4 py-2 text-zinc-500">{s.type}</td>
                <td className="px-4 py-2 text-zinc-500">
                  {formatTime(s.createdAt)}
                </td>
                <td className="px-4 py-2">
                  <div className="flex gap-1">
                    <Button
                      size="sm"
                      variant="secondary"
                      onClick={() => void exportMd(s.id)}
                    >
                      Export
                    </Button>
                    <Button
                      size="sm"
                      variant="destructive"
                      onClick={() =>
                        void deleteSession(s.id).then(() => refreshSessions())
                      }
                    >
                      Delete
                    </Button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
