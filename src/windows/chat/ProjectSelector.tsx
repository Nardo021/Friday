import { useSessionStore } from "@/state/useSessionStore";

export function ProjectSelector() {
  const projects = useSessionStore((s) => s.projects);
  const selectedProjectId = useSessionStore((s) => s.selectedProjectId);
  const setSelectedProject = useSessionStore((s) => s.setSelectedProject);

  if (projects.length === 0) {
    return (
      <div className="border-b border-zinc-800 px-4 py-2 text-xs text-amber-400">
        Add a project in Command Center → Projects
      </div>
    );
  }

  return (
    <div className="border-b border-zinc-800 px-4 py-2">
      <label className="mb-1 block text-xs text-zinc-500">Project</label>
      <select
        className="w-full rounded-md border border-zinc-700 bg-zinc-900 px-2 py-1 text-sm"
        value={selectedProjectId ?? ""}
        onChange={(e) => setSelectedProject(e.target.value || null)}
      >
        {projects.map((p) => (
          <option key={p.id} value={p.id}>
            {p.name}
          </option>
        ))}
      </select>
    </div>
  );
}
