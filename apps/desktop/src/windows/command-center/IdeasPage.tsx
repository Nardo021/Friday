import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import { deleteIdea, listIdeas, openPanel, type Idea } from "@/lib/tauri";
import { useSessionStore } from "@/state/useSessionStore";

export function IdeasPage() {
  const [ideas, setIdeas] = useState<Idea[]>([]);
  const setSelectedProject = useSessionStore((s) => s.setSelectedProject);

  const load = () => void listIdeas().then(setIdeas);

  useEffect(() => {
    load();
  }, []);

  const convertToTask = (idea: Idea) => {
    if (idea.projectId) setSelectedProject(idea.projectId);
    void openPanel();
  };

  return (
    <div className="space-y-4">
      <h2 className="text-lg font-medium">Ideas</h2>
      <p className="text-sm text-zinc-400">
        Quick captures from Quick Bubble (prefix with 记一下 or save idea).
      </p>
      <div className="space-y-2">
        {ideas.length === 0 && (
          <p className="text-sm text-zinc-500">No ideas yet.</p>
        )}
        {ideas.map((idea) => (
          <div
            key={idea.id}
            className="rounded-lg border border-zinc-800 px-4 py-3"
          >
            <div className="font-medium text-zinc-100">{idea.title}</div>
            <p className="mt-1 text-sm text-zinc-400">{idea.body}</p>
            <div className="mt-2 flex gap-2">
              <Button size="sm" onClick={() => convertToTask(idea)}>
                Convert to task
              </Button>
              <Button
                size="sm"
                variant="secondary"
                onClick={() => void deleteIdea(idea.id).then(load)}
              >
                Delete
              </Button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
