import { useEffect, useRef, useState } from "react";
import { ArrowUp, ChevronDown, Square } from "lucide-react";

import type { AgentMode } from "@friday/agent-core";

import { VoiceRecorder } from "@/components/friday/VoiceRecorder";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Field, FieldDescription, FieldLabel } from "@/components/ui/field";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupText,
  InputGroupTextarea,
} from "@/components/ui/input-group";
import { Spinner } from "@/components/ui/spinner";
import { useSettingsStore } from "@/state/useSettingsStore";

const MODES: { id: AgentMode; label: string; short: string }[] = [
  { id: "cloud_agent", label: "Cursor API", short: "API" },
  { id: "local_cli", label: "Local CLI", short: "CLI" },
];

function ModeMenu({
  value,
  onChange,
  apiKeyConfigured,
}: {
  value: AgentMode;
  onChange: (mode: AgentMode) => void;
  apiKeyConfigured: boolean;
}) {
  const current = MODES.find((m) => m.id === value) ?? MODES[0]!;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <InputGroupButton
          size="sm"
          className="gap-0.5 px-2 text-xs"
          aria-label={`Mode: ${current.label}`}
        >
          <span>{current.short}</span>
          <ChevronDown data-icon="inline-end" />
        </InputGroupButton>
      </DropdownMenuTrigger>
      <DropdownMenuContent side="top" align="start" className="min-w-[180px]">
        <DropdownMenuGroup>
          {MODES.map((mode) => {
            const enabled =
              mode.id === "local_cli" ||
              (mode.id === "cloud_agent" && apiKeyConfigured);
            return (
              <DropdownMenuItem
                key={mode.id}
                disabled={!enabled}
                onSelect={() => enabled && onChange(mode.id)}
              >
                {mode.label}
                {!enabled && mode.id === "cloud_agent" && (
                  <span className="text-muted-foreground"> · add key in Settings</span>
                )}
              </DropdownMenuItem>
            );
          })}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export function QuickComposer({
  draft,
  onDraftChange,
  mode,
  onModeChange,
  busy,
  ready,
  running,
  listening,
  onListeningChange,
  onVoiceTranscript,
  onSend,
  onStop,
  showStop,
}: {
  draft: string;
  onDraftChange: (value: string) => void;
  mode: AgentMode;
  onModeChange: (mode: AgentMode) => void;
  busy: boolean;
  ready: boolean;
  running: boolean;
  listening: boolean;
  onListeningChange: (listening: boolean) => void;
  onVoiceTranscript: (text: string) => void;
  onSend: () => void;
  onStop?: () => void;
  showStop?: boolean;
}) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const apiKeyConfigured = useSettingsStore(
    (s) => s.settings.cursor.apiKeyConfigured ?? false,
  );
  useEffect(() => {
    if (mode === "cloud_agent" && !apiKeyConfigured) {
      onModeChange("local_cli");
    }
  }, [mode, apiKeyConfigured, onModeChange]);

  useEffect(() => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${Math.min(el.scrollHeight, 120)}px`;
  }, [draft]);

  const canSend = draft.trim().length > 0 && !busy && ready;

  const hintId = "quick-composer-hint";

  return (
    <Field className="gap-1.5">
      <FieldLabel htmlFor="quick-composer-input" className="sr-only">
        Message to Friday
      </FieldLabel>
      {!apiKeyConfigured && (
        <FieldDescription className="text-[11px] leading-snug">
          Add a Cursor API key in Settings to use API mode. Local CLI uses{" "}
          <code className="text-[10px]">cursor-agent</code> on this machine.
        </FieldDescription>
      )}
      {mode === "cloud_agent" && apiKeyConfigured && (
        <FieldDescription className="text-[11px] leading-snug">
          Cursor API — uses your saved dashboard key (Cloud Agents).
        </FieldDescription>
      )}
      <InputGroup className="shrink-0 rounded-xl bg-card/90 has-[>textarea]:h-auto">
        {listening && (
          <InputGroupAddon align="block-start">
            <InputGroupText id={hintId} className="text-xs text-muted-foreground">
              Listening…
            </InputGroupText>
          </InputGroupAddon>
        )}
        {!ready && (
          <InputGroupAddon align="block-start">
            <InputGroupText className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <Spinner className="size-3" />
              Connecting…
            </InputGroupText>
          </InputGroupAddon>
        )}
        <InputGroupTextarea
          id="quick-composer-input"
          ref={textareaRef}
          rows={1}
          className="max-h-[120px] min-h-0 py-2.5"
          placeholder={running ? "Follow up on this task…" : "Ask Friday anything…"}
          aria-describedby={listening ? hintId : undefined}
          value={draft}
          onChange={(e) => onDraftChange(e.target.value)}
          disabled={busy}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              if (canSend) onSend();
            }
          }}
        />
        <InputGroupAddon align="block-end" className="w-full justify-between gap-2 pb-1.5">
          <ModeMenu
            value={mode}
            onChange={onModeChange}
            apiKeyConfigured={apiKeyConfigured}
          />
          <div className="flex items-center gap-0.5">
            {showStop && onStop && (
              <InputGroupButton
                size="icon-sm"
                aria-label="Stop session"
                className="text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                onClick={onStop}
                disabled={busy}
              >
                <Square data-icon="inline-start" className="fill-current" />
              </InputGroupButton>
            )}
            <VoiceRecorder
              variant="icon"
              disabled={busy || !ready}
              onTranscript={onVoiceTranscript}
              onListeningChange={onListeningChange}
            />
            <InputGroupButton
              size="icon-sm"
              aria-label="Send message"
              variant="default"
              disabled={!canSend}
              onClick={onSend}
            >
              <ArrowUp data-icon="inline-start" strokeWidth={2.5} />
            </InputGroupButton>
          </div>
        </InputGroupAddon>
      </InputGroup>
    </Field>
  );
}
