import { useEffect, useState } from "react";
import { Lightbulb, ListTodo, Trash2 } from "lucide-react";
import { toast } from "sonner";

import { MotionStagger } from "@/components/friday/Motion";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { usePanelNavigation } from "@/hooks/usePanelNavigation";
import { invokeErrorMessage } from "@/lib/invokeError";
import { deleteIdea, listIdeas, type Idea } from "@/lib/tauri";
import { UX } from "@/lib/ux";
import { useSessionStore } from "@/state/useSessionStore";

export function IdeasPage() {
  const [ideas, setIdeas] = useState<Idea[]>([]);
  const setSelectedProject = useSessionStore((s) => s.setSelectedProject);
  const { goToAgent } = usePanelNavigation();

  const load = () =>
    void listIdeas()
      .then(setIdeas)
      .catch((e) => toast.error(invokeErrorMessage(e)));

  useEffect(() => {
    load();
  }, []);

  const convertToTask = (idea: Idea) => {
    if (idea.projectId) setSelectedProject(idea.projectId);
    goToAgent();
    toast.success("Opened agent — paste or edit your prompt");
  };

  return (
    <div className={UX.page}>
      {ideas.length === 0 ? (
        <Empty className="rounded-lg border border-dashed py-10">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <Lightbulb />
            </EmptyMedia>
            <EmptyTitle className="text-sm font-normal">No ideas yet</EmptyTitle>
            <EmptyDescription className="text-xs">
              In Quick Chat, prefix a note with 记一下 or use “save idea” to capture
              it here.
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      ) : (
        <MotionStagger className="flex flex-col gap-3">
          {ideas.map((idea) => (
            <li key={idea.id}>
              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-base">{idea.title}</CardTitle>
                </CardHeader>
                <CardContent className="pt-0">
                  <p className="text-sm text-muted-foreground whitespace-pre-wrap">
                    {idea.body}
                  </p>
                </CardContent>
                <CardFooter className="gap-2 pt-0">
                  <Button size="sm" onClick={() => convertToTask(idea)}>
                    <ListTodo data-icon="inline-start" />
                    Open in agent
                  </Button>
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() =>
                      void deleteIdea(idea.id)
                        .then(load)
                        .catch((e) => toast.error(invokeErrorMessage(e)))
                    }
                  >
                    <Trash2 data-icon="inline-start" />
                    Delete
                  </Button>
                </CardFooter>
              </Card>
            </li>
          ))}
        </MotionStagger>
      )}
    </div>
  );
}
