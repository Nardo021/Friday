import { useAgentStore } from "@/state/useAgentStore";
import { ApprovalCard } from "@/components/friday/ApprovalCard";

export function ApprovalsPage() {
  const pending = useAgentStore((s) => s.pendingApproval);

  if (!pending) {
    return <p className="text-zinc-400">No pending approvals</p>;
  }

  return (
    <ApprovalCard
      approvalId={pending.approvalId}
      command={pending.command}
      risk={pending.risk}
    />
  );
}
