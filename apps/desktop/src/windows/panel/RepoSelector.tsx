import { Field, FieldDescription, FieldLabel } from "@/components/ui/field";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useSessionStore } from "@/state/useSessionStore";

export function RepoSelector({ optional = false }: { optional?: boolean }) {
  const projects = useSessionStore((s) => s.projects);
  const selectedProjectId = useSessionStore((s) => s.selectedProjectId);
  const setSelectedProject = useSessionStore((s) => s.setSelectedProject);

  return (
    <Field>
      <FieldLabel htmlFor="repo-select">
        {optional ? "Working folder" : "Repository"}
      </FieldLabel>
      {optional && (
        <FieldDescription>
          Leave unset to use your home directory. Pick a repo when the agent
          should run inside a specific codebase.
        </FieldDescription>
      )}
      <Select
        value={selectedProjectId ?? "__general__"}
        onValueChange={(v) =>
          setSelectedProject(v === "__general__" ? null : v || null)
        }
      >
        <SelectTrigger id="repo-select" className="w-full">
          <SelectValue
            placeholder={optional ? "General (no repo)" : "Select project…"}
          />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            {optional && (
              <SelectItem value="__general__">General — no repo</SelectItem>
            )}
            {projects.map((p) => (
              <SelectItem key={p.id} value={p.id}>
                {p.name}
              </SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>
    </Field>
  );
}
