import { useEffect, useState } from "react";
import { Check, FolderKanban, FolderPlus } from "lucide-react";
import { toast } from "sonner";

import { MotionStagger } from "@/components/friday/Motion";
import { PageToolbar } from "@/components/friday/PageToolbar";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, FieldDescription, FieldLabel } from "@/components/ui/field";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { Input } from "@/components/ui/input";
import { invokeErrorMessage } from "@/lib/invokeError";
import { addProject } from "@/lib/tauri";
import { cn } from "@/lib/utils";
import { UX } from "@/lib/ux";
import { usePanelNavigation } from "@/hooks/usePanelNavigation";
import { useSessionStore } from "@/state/useSessionStore";

export function ProjectsPage() {
  const { projects, selectedProjectId, refreshProjects, setSelectedProject } =
    useSessionStore();
  const { goToAgent } = usePanelNavigation();
  const [name, setName] = useState("");
  const [path, setPath] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    refreshProjects();
  }, [refreshProjects]);

  const handleAdd = async () => {
    if (!name.trim() || !path.trim() || busy) return;
    setBusy(true);
    try {
      const project = await addProject(name.trim(), path.trim(), true);
      setName("");
      setPath("");
      setSelectedProject(project.id);
      await refreshProjects();
      toast.success(`Added ${project.name}`);
    } catch (e) {
      toast.error(invokeErrorMessage(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className={UX.page}>
      <PageToolbar className="items-end">
        <div className="flex flex-1 flex-wrap items-end gap-2">
          <Field className="min-w-[140px]">
            <FieldLabel htmlFor="project-name">Name</FieldLabel>
            <Input
              id="project-name"
              placeholder="My app"
              value={name}
              onChange={(e) => setName(e.target.value)}
              disabled={busy}
            />
          </Field>
          <Field className="min-w-[220px] flex-1">
            <FieldLabel htmlFor="project-path">Folder path</FieldLabel>
            <Input
              id="project-path"
              placeholder="D:\Dev\projects\my-repo"
              value={path}
              onChange={(e) => setPath(e.target.value)}
              disabled={busy}
            />
            <FieldDescription>Absolute path to the git repo root.</FieldDescription>
          </Field>
        </div>
        <Button disabled={busy || !name.trim() || !path.trim()} onClick={() => void handleAdd()}>
          <FolderPlus data-icon="inline-start" />
          Add project
        </Button>
      </PageToolbar>

      {projects.length === 0 ? (
        <Empty className="rounded-lg border border-dashed py-10">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <FolderKanban />
            </EmptyMedia>
            <EmptyTitle className="text-sm font-normal">No projects yet</EmptyTitle>
            <EmptyDescription className="text-xs">
              Add a repo path above, then start an agent task.
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      ) : (
        <MotionStagger className="flex flex-col gap-2">
          {projects.map((p) => {
            const selected = p.id === selectedProjectId;
            return (
              <li key={p.id}>
                <Card
                  className={cn(
                    "transition-colors",
                    selected && "border-foreground/20 bg-accent/30",
                  )}
                >
                  <CardHeader className="flex flex-row items-start justify-between gap-2 pb-2">
                    <CardTitle className="text-base">{p.name}</CardTitle>
                    {selected && (
                      <span className="flex items-center gap-1 text-xs text-muted-foreground">
                        <Check className="size-3.5" aria-hidden />
                        Default for new tasks
                      </span>
                    )}
                  </CardHeader>
                  <CardContent className="flex flex-col gap-3 pt-0">
                    <span className="break-all font-mono text-xs text-muted-foreground">
                      {p.path}
                    </span>
                    <div className="flex flex-wrap gap-2">
                      <Button
                        size="sm"
                        variant={selected ? "secondary" : "default"}
                        disabled={selected}
                        onClick={() => {
                          setSelectedProject(p.id);
                          toast.success(`${p.name} selected for new tasks`);
                        }}
                      >
                        {selected ? "Selected" : "Use for new tasks"}
                      </Button>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => goToAgent()}
                      >
                        Start task
                      </Button>
                    </div>
                  </CardContent>
                </Card>
              </li>
            );
          })}
        </MotionStagger>
      )}
    </div>
  );
}
