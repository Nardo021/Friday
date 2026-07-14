import { useEffect, useState } from "react";
import { Cat, CheckCircle2, CircleAlert } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Field,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Spinner } from "@/components/ui/spinner";
import { useFridayReady } from "@/hooks/useFridayReady";
import { invokeErrorMessage } from "@/lib/invokeError";
import {
  finishOnboarding,
  getSettings,
  probeCursorCli,
  saveCursorApiKey,
  saveSettings,
  type CursorCliProbe,
} from "@/lib/tauri";
import type { FridaySettings } from "@friday/agent-core";
import { useSettingsStore } from "@/state/useSettingsStore";

const defaultOnboardingSettings = useSettingsStore.getState().settings;

export function OnboardingWindow() {
  const ready = useFridayReady();
  const [apiKey, setApiKey] = useState("");
  const [settings, setSettings] = useState<FridaySettings | null>(null);
  const [cliProbe, setCliProbe] = useState<CursorCliProbe | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!ready) return;
    void getSettings()
      .then(setSettings)
      .catch(() => setSettings(defaultOnboardingSettings));
    void probeCursorCli()
      .then(setCliProbe)
      .catch(() =>
        setCliProbe({
          found: false,
          path: "cursor-agent",
          error: "Could not probe Cursor CLI",
        }),
      );
  }, [ready]);

  const complete = async (skipKey: boolean) => {
    if (!settings || busy) return;
    setBusy(true);
    setError(null);
    try {
      if (!skipKey && apiKey.trim()) {
        await saveCursorApiKey(apiKey.trim());
      }
      const next: FridaySettings = {
        ...settings,
        onboarding: { completed: true },
      };
      await saveSettings(next);
      await finishOnboarding();
    } catch (e) {
      setError(invokeErrorMessage(e));
    } finally {
      setBusy(false);
    }
  };

  if (!settings) {
    return (
      <div className="flex h-screen items-center justify-center bg-background text-muted-foreground">
        <Spinner />
      </div>
    );
  }

  const hintId = "onboarding-api-key-hint";
  const errorId = "onboarding-api-key-error";

  return (
    <main className="motion-page-in flex h-screen flex-col bg-background px-8 py-10 text-foreground">
      <div className="mb-8 flex flex-col gap-3">
        <p className="text-xs font-medium text-muted-foreground">
          Step 1 of 1 · Setup
        </p>
        <p className="flex items-center gap-1.5 text-xs font-medium uppercase tracking-wide text-muted-foreground">
          <Cat className="size-3.5" strokeWidth={1.6} aria-hidden />
          First launch
        </p>
        <h1 className="text-2xl">Welcome to Friday</h1>
        <p className="text-sm leading-relaxed text-muted-foreground">
          Friday is your desktop companion for Cursor agents. Paste your{" "}
          <strong className="font-medium text-foreground">Cursor API key</strong>{" "}
          from the Cursor dashboard to enable Cloud Agent. Local CLI uses{" "}
          <code className="text-foreground">cursor-agent</code> on your PATH
          (no API key required).
        </p>
      </div>

      {cliProbe && (
        <div
          className={`mb-4 flex items-start gap-2 rounded-md border px-3 py-2 text-sm ${
            cliProbe.found
              ? "border-border bg-muted/40 text-foreground"
              : "border-destructive/40 bg-destructive/5 text-foreground"
          }`}
        >
          {cliProbe.found ? (
            <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-emerald-600" aria-hidden />
          ) : (
            <CircleAlert className="mt-0.5 size-4 shrink-0 text-destructive" aria-hidden />
          )}
          <div className="min-w-0">
            <p className="font-medium">
              {cliProbe.found
                ? "Cursor CLI detected"
                : "Cursor CLI not found"}
            </p>
            <p className="break-all text-xs text-muted-foreground">
              {cliProbe.found
                ? cliProbe.path
                : (cliProbe.error ??
                  "Install Cursor Agent CLI and ensure cursor-agent is on PATH, or set the path in Settings.")}
            </p>
          </div>
        </div>
      )}

      <FieldGroup className="gap-4">
        <Field data-invalid={!!error}>
          <FieldLabel htmlFor="api-key">Cursor API key (optional)</FieldLabel>
          <Input
            id="api-key"
            type="password"
            autoComplete="off"
            placeholder="crsr_… (from Cursor dashboard)"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            disabled={busy}
            aria-invalid={!!error}
            aria-describedby={
              error ? `${hintId} ${errorId}` : hintId
            }
          />
          <FieldDescription id={hintId}>
            Create at{" "}
            <a
              href="https://cursor.com/dashboard"
              className="text-foreground underline underline-offset-2 hover:text-foreground/80"
              target="_blank"
              rel="noreferrer"
            >
              cursor.com/dashboard
            </a>{" "}
            → Integrations → API Keys. Needed for Cloud Agent only. Stored in
            your OS credential manager.
          </FieldDescription>
        </Field>
      </FieldGroup>

      {error && (
        <FieldError id={errorId} className="mt-2">
          {error}
        </FieldError>
      )}

      <div className="mt-auto flex flex-col gap-2 pt-8">
        <Button disabled={busy || !apiKey.trim()} onClick={() => void complete(false)}>
          {busy && <Spinner data-icon="inline-start" />}
          Save key &amp; show pet
        </Button>
        <Button variant="secondary" disabled={busy} onClick={() => void complete(true)}>
          {cliProbe && !cliProbe.found
            ? "Continue without Cloud (Local CLI later)"
            : "Skip for now — use Local CLI"}
        </Button>
      </div>
    </main>
  );
}
