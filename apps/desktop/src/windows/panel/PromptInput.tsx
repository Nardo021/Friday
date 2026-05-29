import { Field, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";

export function PromptInput({
  value,
  onChange,
  disabled,
  onSubmit,
}: {
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  onSubmit?: () => void;
}) {
  return (
    <Field>
      <FieldLabel htmlFor="panel-prompt">Message</FieldLabel>
      <Input
        id="panel-prompt"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="Ask Friday anything…"
        disabled={disabled}
        onKeyDown={(e) => {
          if (e.key === "Enter" && !e.shiftKey && onSubmit) {
            e.preventDefault();
            onSubmit();
          }
        }}
      />
    </Field>
  );
}
