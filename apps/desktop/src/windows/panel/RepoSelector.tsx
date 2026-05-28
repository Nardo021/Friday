import { useSessionStore } from "@/state/useSessionStore";

export function RepoSelector() {
  const projects = useSessionStore((s) => s.projects);
  const selectedProjectId = useSessionStore((s) => s.selectedProjectId);
  const setSelectedProject = useSessionStore((s) => s.setSelectedProject);

  return (
    <div className="space-y-1">
      <label className="text-xs font-medium text-zinc-500">Repository</label>
      <select
        className="w-full rounded-md border border-zinc-700 bg-zinc-900 px-3 py-2 text-sm text-zinc-100"
        value={selectedProjectId ?? ""}
        onChange={(e) => setSelectedProject(e.target.value || null)}
      >
        <option value="">Select project...</option>
        {projects.map((p) => (
          <option key={p.id} value={p.id}>
            {p.name}
          </option>
        ))}
      </select>
    </div>
  );
}
