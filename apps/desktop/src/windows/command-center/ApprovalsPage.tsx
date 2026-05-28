import { ApprovalCard } from "@/components/friday/ApprovalCard";
import { useActivePendingApproval } from "@/state/useSessionStore";

export function ApprovalsPage() {
  const pending = useActivePendingApproval();

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
