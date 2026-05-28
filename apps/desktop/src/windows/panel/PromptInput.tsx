import { Input } from "@/components/ui/input";

export function PromptInput({
  value,
  onChange,
  disabled,
}: {
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
}) {
  return (
    <div className="space-y-1">
      <label className="text-xs font-medium text-zinc-500">Prompt</label>
      <Input
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="Describe what Friday should do..."
        disabled={disabled}
      />
    </div>
  );
}
