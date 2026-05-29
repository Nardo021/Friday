import type { LucideIcon } from "lucide-react";
import {
  Bot,
  Cable,
  FileText,
  FolderKanban,
  LayoutDashboard,
  Lightbulb,
  List,
  Settings,
  Shield,
} from "lucide-react";

import { TabsList, TabsTrigger } from "@/components/ui/tabs";
import { cn } from "@/lib/utils";
import { UX } from "@/lib/ux";

export type PanelPageId =
  | "agent"
  | "dashboard"
  | "sessions"
  | "projects"
  | "ideas"
  | "approvals"
  | "logs"
  | "adapters"
  | "settings";

type NavItem = {
  id: PanelPageId;
  label: string;
  icon: LucideIcon;
};

const NAV_GROUPS: { label: string; items: NavItem[] }[] = [
  {
    label: "Agent",
    items: [{ id: "agent", label: "Agent", icon: Bot }],
  },
  {
    label: "Overview",
    items: [
      { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
      { id: "sessions", label: "All sessions", icon: List },
    ],
  },
  {
    label: "Work",
    items: [
      { id: "projects", label: "Projects", icon: FolderKanban },
      { id: "ideas", label: "Ideas", icon: Lightbulb },
      { id: "approvals", label: "Approvals", icon: Shield },
    ],
  },
  {
    label: "System",
    items: [
      { id: "logs", label: "Logs", icon: FileText },
      { id: "adapters", label: "Adapters", icon: Cable },
    ],
  },
];

export const SETTINGS_NAV: NavItem = {
  id: "settings",
  label: "Settings",
  icon: Settings,
};

export function PanelNav() {
  return (
    <div className={cn("flex min-h-0 flex-1 flex-col", UX.betweenGroups)}>
      {NAV_GROUPS.map((group) => (
        <div key={group.label} className={cn("flex flex-col", UX.section)}>
          <p className="px-3 text-[10px] font-medium uppercase tracking-wider text-muted-foreground">
            {group.label}
          </p>
          <TabsList
            variant="line"
            className="h-auto w-full flex-col items-stretch gap-0.5 bg-transparent p-0"
          >
            {group.items.map((item) => {
              const Icon = item.icon;
              return (
                <TabsTrigger
                  key={item.id}
                  value={item.id}
                  className="h-9 w-full justify-start gap-2 px-3 text-sm data-[state=active]:bg-accent data-[state=active]:font-medium"
                >
                  <Icon aria-hidden />
                  {item.label}
                </TabsTrigger>
              );
            })}
          </TabsList>
        </div>
      ))}
      <div className="mt-auto border-t border-border pt-4">
        <TabsList
          variant="line"
          className="h-auto w-full flex-col items-stretch bg-transparent p-0"
        >
          <TabsTrigger
            value={SETTINGS_NAV.id}
            className="h-9 w-full justify-start gap-2 px-3 text-sm data-[state=active]:bg-accent data-[state=active]:font-medium"
          >
            <SETTINGS_NAV.icon aria-hidden />
            {SETTINGS_NAV.label}
          </TabsTrigger>
        </TabsList>
      </div>
    </div>
  );
}
