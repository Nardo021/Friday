import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { addProject } from "@/lib/tauri";
import { useSessionStore } from "@/state/useSessionStore";

export function ProjectsPage() {
  const { projects, refreshProjects } = useSessionStore();
  const [name, setName] = useState("");
  const [path, setPath] = useState("");

  useEffect(() => {
    refreshProjects();
  }, [refreshProjects]);

  const handleAdd = async () => {
    if (!name.trim() || !path.trim()) return;
    await addProject(name.trim(), path.trim(), true);
    setName("");
    setPath("");
    refreshProjects();
  };

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap gap-2">
        <Input
          placeholder="Project name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <Input
          placeholder="Project path"
          value={path}
          onChange={(e) => setPath(e.target.value)}
          className="min-w-64 flex-1"
        />
        <Button onClick={handleAdd}>Add Project</Button>
      </div>
      <div className="space-y-2">
        {projects.map((p) => (
          <div
            key={p.id}
            className="rounded border border-zinc-800 px-4 py-3"
          >
            <div className="font-medium">{p.name}</div>
            <div className="text-xs text-zinc-500">{p.path}</div>
            <div className="mt-1 text-xs text-zinc-400">
              Trusted: {p.trusted ? "yes" : "no"} · Adapter:{" "}
              {p.defaultAdapterId}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
