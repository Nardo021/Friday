import { useState } from "react";

import { cn } from "@/lib/utils";

import { AdaptersPage } from "./AdaptersPage";
import { ApprovalsPage } from "./ApprovalsPage";
import { DashboardPage } from "./DashboardPage";
import { LogsPage } from "./LogsPage";
import { ProjectsPage } from "./ProjectsPage";
import { SessionsPage } from "./SessionsPage";
import { SettingsPage } from "./SettingsPage";

const NAV = [
  { id: "dashboard", label: "Dashboard" },
  { id: "sessions", label: "Sessions" },
  { id: "projects", label: "Projects" },
  { id: "approvals", label: "Approvals" },
  { id: "logs", label: "Logs" },
  { id: "adapters", label: "Adapters" },
  { id: "settings", label: "Settings" },
] as const;

type PageId = (typeof NAV)[number]["id"];

export function CommandCenter() {
  const [page, setPage] = useState<PageId>("dashboard");

  return (
    <div className="flex h-screen bg-zinc-950 text-zinc-100">
      <aside className="w-52 border-r border-zinc-800 p-4">
        <h1 className="mb-4 text-lg font-semibold">Friday</h1>
        <nav className="space-y-1">
          {NAV.map((item) => (
            <button
              key={item.id}
              className={cn(
                "block w-full rounded px-3 py-2 text-left text-sm",
                page === item.id
                  ? "bg-indigo-600/20 text-indigo-200"
                  : "text-zinc-400 hover:bg-zinc-900",
              )}
              onClick={() => setPage(item.id)}
            >
              {item.label}
            </button>
          ))}
        </nav>
      </aside>
      <main className="flex-1 overflow-y-auto p-6">
        {page === "dashboard" && <DashboardPage />}
        {page === "sessions" && <SessionsPage />}
        {page === "projects" && <ProjectsPage />}
        {page === "approvals" && <ApprovalsPage />}
        {page === "logs" && <LogsPage />}
        {page === "adapters" && <AdaptersPage />}
        {page === "settings" && <SettingsPage />}
      </main>
    </div>
  );
}
