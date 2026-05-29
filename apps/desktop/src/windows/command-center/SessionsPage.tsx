import { useEffect, useMemo, useState, type MouseEvent } from "react";
import { ClipboardCopy, Inbox, RefreshCw, Trash2 } from "lucide-react";
import { toast } from "sonner";

import { MotionItem } from "@/components/friday/Motion";
import { PageToolbar } from "@/components/friday/PageToolbar";
import { StatusPill } from "@/components/friday/StatusPill";
import { Button } from "@/components/ui/button";
import { Field, FieldLabel } from "@/components/ui/field";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { usePanelNavigation } from "@/hooks/usePanelNavigation";
import { invokeErrorMessage } from "@/lib/invokeError";
import { deleteSession, exportSessionMarkdown } from "@/lib/tauri";
import { formatTime } from "@/lib/time";
import { UX } from "@/lib/ux";
import { useSessionList, useSessionStore } from "@/state/useSessionStore";

export function SessionsPage() {
  const sessions = useSessionList();
  const refreshSessions = useSessionStore((s) => s.refreshSessions);
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState("all");
  const [busyId, setBusyId] = useState<string | null>(null);
  const { goToAgent } = usePanelNavigation();

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

  const exportMd = async (id: string, e: MouseEvent) => {
    e.stopPropagation();
    try {
      const md = await exportSessionMarkdown(id);
      await navigator.clipboard.writeText(md);
      toast.success("Markdown copied to clipboard");
    } catch (err) {
      toast.error(invokeErrorMessage(err));
    }
  };

  const remove = async (id: string, e: MouseEvent) => {
    e.stopPropagation();
    setBusyId(id);
    try {
      await deleteSession(id);
      await refreshSessions();
      toast.success("Session deleted");
    } catch (err) {
      toast.error(invokeErrorMessage(err));
    } finally {
      setBusyId(null);
    }
  };

  return (
    <div className={UX.page}>
      <PageToolbar>
        <div className="flex flex-wrap items-end gap-2">
          <Field className="min-w-[200px] flex-1">
            <FieldLabel htmlFor="session-search">Search</FieldLabel>
            <Input
              id="session-search"
              placeholder="Title, type, repo…"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
          </Field>
          <Field>
            <FieldLabel htmlFor="session-status-filter">Status</FieldLabel>
            <Select value={statusFilter} onValueChange={setStatusFilter}>
              <SelectTrigger id="session-status-filter" className="w-[160px]">
                <SelectValue placeholder="All" />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  <SelectItem value="all">All</SelectItem>
                  <SelectItem value="running_command">Running</SelectItem>
                  <SelectItem value="done">Done</SelectItem>
                  <SelectItem value="stopped">Stopped</SelectItem>
                  <SelectItem value="error">Error</SelectItem>
                </SelectGroup>
              </SelectContent>
            </Select>
          </Field>
        </div>
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="h-9"
          onClick={() => void refreshSessions()}
        >
          <RefreshCw data-icon="inline-start" />
          Refresh
        </Button>
      </PageToolbar>

      <p className="text-xs text-muted-foreground">
        {filtered.length} of {sessions.length} sessions · click a row to open in
        agent
      </p>

      {filtered.length === 0 ? (
        <Empty className="rounded-lg border border-dashed py-10">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <Inbox />
            </EmptyMedia>
            <EmptyTitle className="text-sm font-normal">
              {sessions.length === 0 ? "No sessions yet" : "No matches"}
            </EmptyTitle>
            <EmptyDescription className="text-xs">
              {sessions.length === 0
                ? "Start a task from the Agent tab or Quick Chat."
                : "Try a different search or status filter."}
            </EmptyDescription>
          </EmptyHeader>
          {sessions.length === 0 && (
            <Button size="sm" className="mt-2" onClick={() => goToAgent()}>
              Go to agent
            </Button>
          )}
        </Empty>
      ) : (
        <MotionItem>
        <div className="overflow-hidden rounded-lg border border-border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Status</TableHead>
                <TableHead>Title</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>Created</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filtered.map((s) => (
                <TableRow
                  key={s.id}
                  className="cursor-pointer"
                  onClick={() => goToAgent(s.id)}
                >
                  <TableCell>
                    <StatusPill status={s.status} />
                  </TableCell>
                  <TableCell className="max-w-[240px] truncate font-medium">
                    {s.title}
                  </TableCell>
                  <TableCell className="text-muted-foreground">{s.type}</TableCell>
                  <TableCell className="text-muted-foreground whitespace-nowrap">
                    {formatTime(s.createdAt)}
                  </TableCell>
                  <TableCell className="text-right">
                    <div
                      className="flex justify-end gap-1"
                      onClick={(e) => e.stopPropagation()}
                      onKeyDown={(e) => e.stopPropagation()}
                    >
                      <Button
                        size="sm"
                        variant="secondary"
                        disabled={busyId === s.id}
                        onClick={(e) => void exportMd(s.id, e)}
                      >
                        <ClipboardCopy data-icon="inline-start" />
                        Export
                      </Button>
                      <Button
                        size="sm"
                        variant="destructive"
                        disabled={busyId === s.id}
                        onClick={(e) => void remove(s.id, e)}
                      >
                        <Trash2 data-icon="inline-start" />
                        Delete
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
        </MotionItem>
      )}
    </div>
  );
}
