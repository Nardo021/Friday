import { Bot, ShieldCheck } from "lucide-react";

import { ApprovalCard } from "@/components/friday/ApprovalCard";
import { Button } from "@/components/ui/button";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { usePanelNavigation } from "@/hooks/usePanelNavigation";
import { UX } from "@/lib/ux";
import { useActivePendingApproval } from "@/state/useSessionStore";

export function ApprovalsPage() {
  const pending = useActivePendingApproval();
  const { goToAgent } = usePanelNavigation();

  if (!pending) {
    return (
      <div className={UX.page}>
        <Empty className="rounded-lg border border-dashed py-12">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <ShieldCheck />
            </EmptyMedia>
            <EmptyTitle className="text-sm font-normal">
              No pending approvals
            </EmptyTitle>
            <EmptyDescription className="text-xs max-w-sm">
              When an agent wants to run a high-risk command, you can approve or
              reject it here — or from the banner on the active session in Agent.
            </EmptyDescription>
          </EmptyHeader>
          <Button size="sm" variant="secondary" className="mt-3" onClick={() => goToAgent()}>
            <Bot data-icon="inline-start" />
            Open agent
          </Button>
        </Empty>
      </div>
    );
  }

  return (
    <div className={UX.page}>
      <ApprovalCard
        approvalId={pending.approvalId}
        command={pending.command}
        risk={pending.risk}
      />
      <p className="text-xs text-muted-foreground">
        This approval is tied to your active session. Switch sessions in Agent if
        you do not see the expected command.
      </p>
    </div>
  );
}
