import { useEffect, useState } from "react";
import {
  ClipboardCopy,
  Link,
  RefreshCw,
  Save,
  Smartphone,
  Trash2,
} from "lucide-react";

import type { FridaySettings, MobileBridgeSettingsView } from "@friday/agent-core";

import { ConfirmDialog } from "@/components/friday/ConfirmDialog";
import { Button } from "@/components/ui/button";
import { Field, FieldDescription, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Spinner } from "@/components/ui/spinner";
import { toast } from "sonner";
import { invokeErrorMessage } from "@/lib/invokeError";
import {
  saveCursorApiKey,
  clearCursorApiKey,
  clearLocalData,
  getLocalDataPath,
  saveSttApiKey,
  clearSttApiKey,
  getMobileBridgeSettings,
  updateMobileBridgeSettings,
  regenerateMobileBridgeToken,
  probeCursorCli,
  type CursorCliProbe,
} from "@/lib/tauri";
import { cn } from "@/lib/utils";
import { UX } from "@/lib/ux";
import { useSettingsStore } from "@/state/useSettingsStore";

function CursorApiKeyField({ configured }: { configured: boolean }) {
  const load = useSettingsStore((s) => s.load);
  const [value, setValue] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [confirmRemove, setConfirmRemove] = useState(false);

  const save = async () => {
    if (!value.trim() || busy) return;
    setBusy(true);
    setMessage(null);
    try {
      await saveCursorApiKey(value.trim());
      setValue("");
      await load();
      const ok = useSettingsStore.getState().settings.cursor.apiKeyConfigured;
      if (!ok) {
        const msg =
          "Key was not stored on this device. Restart Friday and try again.";
        setMessage(msg);
        toast.error(msg);
        return;
      }
      setMessage("Saved and verified with Cursor API.");
      toast.success("Cursor API key saved. Use Cursor API mode in Quick Chat or Agent.");
    } catch (e) {
      const msg = invokeErrorMessage(e);
      setMessage(msg);
      toast.error(msg);
    } finally {
      setBusy(false);
    }
  };

  const remove = async () => {
    if (busy || !configured) return;
    setBusy(true);
    setMessage(null);
    try {
      await clearCursorApiKey();
      await load();
      setMessage("Removed.");
      toast.success("Cursor API key removed");
    } catch (e) {
      const msg = invokeErrorMessage(e);
      setMessage(msg);
      toast.error(msg);
    } finally {
      setBusy(false);
    }
  };

  return (
    <>
      <Field>
        <FieldLabel htmlFor="cursor-api-key">API key</FieldLabel>
        <Input
          id="cursor-api-key"
          type="password"
          placeholder={configured ? "Replace Cursor API key…" : "crsr_… from Cursor dashboard"}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          disabled={busy}
        />
        <FieldDescription>
          From{" "}
          <a
            href="https://cursor.com/dashboard"
            className="text-foreground underline underline-offset-2 hover:text-foreground/80"
            target="_blank"
            rel="noreferrer"
          >
            cursor.com/dashboard
          </a>{" "}
          → Integrations → API Keys. Saved keys are verified against Cursor API
          before storing. Use <strong className="font-medium text-foreground">Cursor API</strong>{" "}
          mode in chat; Local CLI does not need this key.
        </FieldDescription>
      </Field>
      <div className="flex gap-2">
        <Button size="sm" disabled={busy || !value.trim()} onClick={() => void save()}>
          <Save data-icon="inline-start" />
          Save API key
        </Button>
        {configured && (
          <Button
            size="sm"
            variant="secondary"
            disabled={busy}
            onClick={() => setConfirmRemove(true)}
          >
            <Trash2 data-icon="inline-start" />
            Remove API key
          </Button>
        )}
      </div>
      {message && <p className="text-xs text-muted-foreground">{message}</p>}
      <ConfirmDialog
        open={confirmRemove}
        onOpenChange={setConfirmRemove}
        title="Remove Cursor API key?"
        description="The stored Cursor API key will be removed from this device. Cloud Agent features will stop working until you add a key again."
        confirmLabel="Remove"
        destructive
        onConfirm={() => void remove()}
      />
    </>
  );
}

function MobileRemoteSection() {
  const [bridge, setBridge] = useState<MobileBridgeSettingsView | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    void getMobileBridgeSettings().then(setBridge);
  }, []);

  const toggle = async (enabled: boolean) => {
    if (!bridge || busy) return;
    setBusy(true);
    setMessage(null);
    try {
      const next = await updateMobileBridgeSettings({
        enabled,
        port: bridge.port,
        authToken: bridge.authToken,
      });
      setBridge(next);
      setMessage(enabled ? "Mobile bridge enabled." : "Mobile bridge disabled.");
    } catch (e) {
      setMessage(e instanceof Error ? e.message : "Failed to update bridge");
    } finally {
      setBusy(false);
    }
  };

  const savePort = async (port: number) => {
    if (!bridge || busy) return;
    setBusy(true);
    try {
      const next = await updateMobileBridgeSettings({
        enabled: bridge.enabled,
        port,
        authToken: bridge.authToken,
      });
      setBridge(next);
    } finally {
      setBusy(false);
    }
  };

  const regenToken = async () => {
    if (busy) return;
    setBusy(true);
    setMessage(null);
    try {
      const next = await regenerateMobileBridgeToken();
      setBridge(next);
      setMessage("Token regenerated. Update paired mobile devices.");
    } catch (e) {
      setMessage(e instanceof Error ? e.message : "Failed to regenerate token");
    } finally {
      setBusy(false);
    }
  };

  const copyToken = async () => {
    if (!bridge?.authToken) return;
    await navigator.clipboard.writeText(bridge.authToken);
    setMessage("Token copied.");
  };

  const copyUrl = async () => {
    if (!bridge?.localUrl) return;
    await navigator.clipboard.writeText(bridge.localUrl);
    setMessage("URL copied.");
  };

  if (!bridge) {
    return (
      <section>
        <h3 className="mb-2 font-medium">Mobile Remote</h3>
        <p className="text-sm text-muted-foreground">Loading…</p>
      </section>
    );
  }

  return (
    <section>
      <h3 className="mb-2 flex items-center gap-2 font-medium">
        <Smartphone />
        Mobile Remote
      </h3>
      <p className="mb-3 text-sm text-muted-foreground">
        Allow the Friday iOS companion on your LAN to observe sessions, approve commands, and stop agents.
      </p>
      <div className="mb-3 flex items-center gap-2">
        <Switch
          id="mobile-bridge"
          checked={bridge.enabled}
          disabled={busy}
          onCheckedChange={(v) => void toggle(v)}
        />
        <Label htmlFor="mobile-bridge">Enable mobile bridge</Label>
      </div>
      <Label className="mb-1 block text-sm text-muted-foreground">Port</Label>
      <Input
        type="number"
        className="mb-3 max-w-[8rem]"
        value={bridge.port}
        disabled={busy}
        onChange={(e) => void savePort(Number(e.target.value))}
      />
      <p className="mb-1 text-sm text-muted-foreground">Local URL</p>
      <div className="mb-3 flex flex-wrap items-center gap-2">
        <code className="break-all rounded bg-muted px-2 py-1 text-xs">{bridge.localUrl}</code>
        <Button size="sm" variant="secondary" disabled={busy} onClick={() => void copyUrl()}>
          <Link data-icon="inline-start" />
          Copy URL
        </Button>
      </div>
      <p className="mb-1 text-sm text-muted-foreground">Auth token</p>
      <div className="mb-3 flex flex-wrap items-center gap-2">
        <code className="break-all rounded bg-muted px-2 py-1 text-xs">{bridge.authToken}</code>
        <Button size="sm" variant="secondary" disabled={busy} onClick={() => void copyToken()}>
          <ClipboardCopy data-icon="inline-start" />
          Copy token
        </Button>
        <Button size="sm" variant="secondary" disabled={busy} onClick={() => void regenToken()}>
          <RefreshCw data-icon="inline-start" />
          Regenerate
        </Button>
      </div>
      <p className="text-xs text-muted-foreground">
        In the mobile app, enter the URL and token above. Phone and desktop must be on the same Wi‑Fi.
        See <span className="text-muted-foreground">docs/MOBILE_REMOTE.md</span> for API details.
      </p>
      {message && <p className="mt-2 text-xs text-muted-foreground">{message}</p>}
    </section>
  );
}

function LocalDataSection() {
  const [path, setPath] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [confirmWipe, setConfirmWipe] = useState(false);

  useEffect(() => {
    void getLocalDataPath().then(setPath);
  }, []);

  const wipe = async () => {
    if (busy) return;
    setBusy(true);
    try {
      await clearLocalData();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to clear data");
      setBusy(false);
    }
  };

  return (
    <section>
      <h3 className="mb-2 font-medium">Local data</h3>
      <p className="mb-2 text-sm text-muted-foreground">
        Sessions, projects, and settings are stored on this device only.
      </p>
      {path && (
        <p className="mb-3 break-all font-mono text-xs text-muted-foreground">{path}</p>
      )}
      <Button
        size="sm"
        variant="destructive"
        disabled={busy}
        onClick={() => setConfirmWipe(true)}
      >
        <Trash2 data-icon="inline-start" />
        Delete all local data
      </Button>
      <p className="mt-2 text-xs text-muted-foreground">
        Uninstalling via Windows installer also offers a checkbox to remove local
        data.
      </p>
      <ConfirmDialog
        open={confirmWipe}
        onOpenChange={setConfirmWipe}
        title="Delete all local data?"
        description="This removes settings, sessions, projects, logs, and stored API keys on this device. The app will restart."
        confirmLabel="Delete everything"
        destructive
        onConfirm={() => void wipe()}
      />
    </section>
  );
}

function SttApiKeyField({ configured }: { configured: boolean }) {
  const load = useSettingsStore((s) => s.load);
  const [value, setValue] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const save = async () => {
    if (!value.trim() || busy) return;
    setBusy(true);
    setMessage(null);
    try {
      await saveSttApiKey(value.trim());
      setValue("");
      await load();
      setMessage("Saved.");
      toast.success("OpenAI key saved for voice");
    } catch (e) {
      const msg = invokeErrorMessage(e);
      setMessage(msg);
      toast.error(msg);
    } finally {
      setBusy(false);
    }
  };

  const remove = async () => {
    if (busy || !configured) return;
    setBusy(true);
    setMessage(null);
    try {
      await clearSttApiKey();
      await load();
      setMessage("Removed.");
      toast.success("OpenAI key removed");
    } catch (e) {
      const msg = invokeErrorMessage(e);
      setMessage(msg);
      toast.error(msg);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex flex-wrap gap-2">
      <Input
        type="password"
        placeholder={configured ? "Replace OpenAI key…" : "sk-… (OpenAI, voice only)"}
        value={value}
        onChange={(e) => setValue(e.target.value)}
        disabled={busy}
        className="max-w-xs"
      />
      <Button size="sm" disabled={busy || !value.trim()} onClick={() => void save()}>
        <Save data-icon="inline-start" />
        Save STT key
      </Button>
      {configured && (
        <Button size="sm" variant="secondary" disabled={busy} onClick={() => void remove()}>
          <Trash2 data-icon="inline-start" />
          Remove
        </Button>
      )}
      {message && (
        <p className="w-full text-xs text-muted-foreground">{message}</p>
      )}
    </div>
  );
}

function formatArgTemplates(templates: string[]): string {
  return JSON.stringify(templates);
}

function CursorCliProbeRow() {
  const [probe, setProbe] = useState<CursorCliProbe | null>(null);
  const [busy, setBusy] = useState(false);

  const run = async () => {
    setBusy(true);
    try {
      const result = await probeCursorCli();
      setProbe(result);
      if (result.found) {
        toast.success(`Found Cursor CLI: ${result.path}`);
      } else {
        toast.error(result.error ?? "cursor-agent not found");
      }
    } catch (e) {
      toast.error(invokeErrorMessage(e));
    } finally {
      setBusy(false);
    }
  };

  useEffect(() => {
    void run();
  }, []);

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
        <Button size="sm" variant="secondary" disabled={busy} onClick={() => void run()}>
          {busy ? <Spinner data-icon="inline-start" /> : <RefreshCw data-icon="inline-start" />}
          Check Cursor CLI
        </Button>
        {probe && (
          <span className="text-xs text-muted-foreground">
            {probe.found ? `OK · ${probe.path}` : (probe.error ?? "Not found")}
          </span>
        )}
      </div>
      <FieldDescription>
        Local Agent mode runs <code className="text-foreground">cursor-agent --print --output-format stream-json</code>.
        Install the Cursor CLI and ensure it is on PATH, or set an absolute path below.
      </FieldDescription>
    </div>
  );
}

function parseArgTemplates(raw: string): string[] {
  const trimmed = raw.trim();
  if (!trimmed) return [];
  if (trimmed.startsWith("[")) {
    const parsed = JSON.parse(trimmed) as unknown;
    if (!Array.isArray(parsed) || !parsed.every((v) => typeof v === "string")) {
      throw new Error("JSON array must contain only strings");
    }
    return parsed;
  }
  return trimmed.split(",").map((s) => s.trim()).filter(Boolean);
}

export function SettingsPage() {
  const { settings, loaded, load, update } = useSettingsStore();
  const [argTemplateText, setArgTemplateText] = useState("");
  const [argError, setArgError] = useState<string | null>(null);

  useEffect(() => {
    if (!loaded) load();
  }, [loaded, load]);

  const headlessArgTemplates = Array.isArray(settings.cursor.argTemplates)
    ? settings.cursor.argTemplates
    : (settings.cursor.argTemplates?.headlessStream ?? []);

  useEffect(() => {
    if (loaded) {
      setArgTemplateText(formatArgTemplates(headlessArgTemplates));
    }
  }, [loaded, headlessArgTemplates]);

  if (!loaded) {
    return (
      <div className="flex items-center gap-2 text-muted-foreground">
        <Spinner />
        Loading settings…
      </div>
    );
  }

  const patchCursor = (patch: Partial<FridaySettings["cursor"]>) =>
    update({
      ...settings,
      cursor: { ...settings.cursor, ...patch },
    });

  const saveArgTemplates = () => {
    try {
      const headlessStream = parseArgTemplates(argTemplateText);
      setArgError(null);
      void patchCursor({
        argTemplates: { ...settings.cursor.argTemplates, headlessStream },
      });
    } catch (e) {
      setArgError(e instanceof Error ? e.message : "Invalid arg templates");
    }
  };

  return (
    <div className={cn("flex max-w-xl flex-col", UX.page)}>
      <section className={cn(UX.section, "motion-item-in")} data-od-id="settings-workspace">
        <h3 className="text-base font-medium">Workspace</h3>
        <p className="text-sm text-muted-foreground">
          Pet, motion, and how Friday starts with your desktop.
        </p>
        <div className={cn("flex flex-col", UX.withinGroup)}>
        <div>
        <Label className="mb-2 block text-sm">Pet scale</Label>
        <Input
          type="number"
          step="0.1"
          min="0.5"
          max="2"
          value={settings.appearance.petScale}
          onChange={(e) =>
            update({
              ...settings,
              appearance: {
                ...settings.appearance,
                petScale: Number(e.target.value),
              },
            })
          }
        />
        <Field orientation="horizontal" className="mt-4">
          <Switch
            id="reduced-motion"
            checked={settings.appearance.reducedMotion}
            onCheckedChange={(v) =>
              update({
                ...settings,
                appearance: {
                  ...settings.appearance,
                  reducedMotion: v,
                },
              })
            }
          />
          <FieldLabel htmlFor="reduced-motion" className="font-normal">
            Reduce motion
          </FieldLabel>
        </Field>
        <FieldDescription>
          Minimizes movement and uses color-only state cues (matches system
          preference when enabled).
        </FieldDescription>
        <div className="mt-4 flex items-center gap-2">
          <Switch
            id="patrol-enabled"
            checked={settings.pet.patrolEnabled}
            onCheckedChange={(v) =>
              update({
                ...settings,
                pet: { ...settings.pet, patrolEnabled: v },
              })
            }
          />
          <Label htmlFor="patrol-enabled">Pet patrol (walk along screen edge)</Label>
        </div>
        <div className="mt-2 flex items-center gap-2">
          <Switch
            id="show-bubble"
            checked={settings.behavior.showBubbleOnStatusChange}
            onCheckedChange={(v) =>
              update({
                ...settings,
                behavior: {
                  ...settings.behavior,
                  showBubbleOnStatusChange: v,
                },
              })
            }
          />
          <Label htmlFor="show-bubble">Show status bubble on agent updates</Label>
        </div>
        <div className="flex items-center gap-2 pt-2">
          <Switch
            id="launch-startup"
            checked={settings.behavior.launchAtStartup}
            onCheckedChange={(v) =>
              update({
                ...settings,
                behavior: {
                  ...settings.behavior,
                  launchAtStartup: v,
                },
              })
            }
          />
          <Label htmlFor="launch-startup">Launch Friday at startup</Label>
        </div>
        <div className="text-sm text-muted-foreground">
          <p className="mb-1 font-medium text-foreground">Shortcuts</p>
          <ul className="flex flex-col gap-1">
            <li>
              Quick chat:{" "}
              {navigator.userAgent.includes("Windows")
                ? "Alt+Shift+Space"
                : settings.shortcuts.quickBubble}
            </li>
            <li>Panel: {settings.shortcuts.openPanel}</li>
            <li>Voice: {settings.shortcuts.voiceInput}</li>
            <li>Stop session: {settings.shortcuts.stopSession}</li>
          </ul>
        </div>
        </div>
        </div>
      </section>

      <section className={cn(UX.section, "motion-item-in")} data-od-id="settings-agents">
        <h3 className="text-base font-medium">Agents &amp; API</h3>
        <p className="text-sm text-muted-foreground">
          Keys and cloud behavior for Cursor agents.
        </p>
        <div className={cn("flex flex-col", UX.withinGroup)}>
        <div>
        <p className="text-sm text-muted-foreground">
          {settings.cursor.apiKeyConfigured
            ? "Cursor API key saved in OS secure storage on this device."
            : "No Cursor API key configured yet."}
        </p>
        <CursorApiKeyField configured={settings.cursor.apiKeyConfigured ?? false} />
        </div>
        <div>
        <p className="mb-2 text-sm text-muted-foreground">
          Cloud agents run on Cursor infrastructure and can open pull requests.
        </p>
        <div className="flex items-center gap-2">
          <Switch
            id="auto-pr"
            checked={settings.cloud?.autoCreatePr ?? true}
            onCheckedChange={(v) =>
              update({
                ...settings,
                cloud: {
                  autoCreatePr: v,
                  model: settings.cloud?.model,
                },
              })
            }
          />
          <Label htmlFor="auto-pr">Auto-create pull request when cloud run completes</Label>
        </div>
        <Label className="mt-2 block text-sm text-muted-foreground">Model (optional)</Label>
        <Input
          className="mt-1 max-w-xs"
          placeholder="e.g. composer-2.5"
          value={settings.cloud?.model ?? ""}
          onChange={(e) =>
            update({
              ...settings,
              cloud: {
                autoCreatePr: settings.cloud?.autoCreatePr ?? true,
                model: e.target.value || undefined,
              },
            })
          }
        />
        </div>
        <details className="rounded-md border border-border px-4 py-3">
          <summary className="cursor-pointer text-sm font-medium">
            Advanced: Cursor CLI
          </summary>
          <div className={cn("mt-3 flex flex-col", UX.withinGroup)}>
        <CursorCliProbeRow />
        <Input
          placeholder="Executable path (optional)"
          value={settings.cursor.executablePath ?? ""}
          onChange={(e) =>
            patchCursor({
              executablePath: e.target.value || undefined,
            })
          }
        />
        <div className="flex items-center gap-2">
          <Switch
            id="use-pty"
            checked={settings.cursor.usePty}
            onCheckedChange={(v) => patchCursor({ usePty: v })}
          />
          <Label htmlFor="use-pty">Use PTY for agent process</Label>
        </div>
        <div className="grid grid-cols-2 gap-3">
          <div>
            <Label className="mb-1 block text-sm text-muted-foreground">Terminal cols</Label>
            <Input
              type="number"
              min={40}
              max={300}
              value={settings.cursor.terminalCols}
              onChange={(e) =>
                patchCursor({ terminalCols: Number(e.target.value) })
              }
            />
          </div>
          <div>
            <Label className="mb-1 block text-sm text-muted-foreground">Terminal rows</Label>
            <Input
              type="number"
              min={10}
              max={100}
              value={settings.cursor.terminalRows}
              onChange={(e) =>
                patchCursor({ terminalRows: Number(e.target.value) })
              }
            />
          </div>
        </div>
        <div>
        <Label className="mb-1 block text-sm text-muted-foreground">
          Headless stream arg templates (comma-separated or JSON array)
        </Label>
        <Input
          className="font-mono text-xs"
          value={argTemplateText}
          onChange={(e) => setArgTemplateText(e.target.value)}
        />
        {argError && <p className="text-xs text-destructive">{argError}</p>}
        <Button size="sm" variant="secondary" className="mt-2" onClick={saveArgTemplates}>
          <Save data-icon="inline-start" />
          Apply arg templates
        </Button>
        </div>
          </div>
        </details>
        </div>
      </section>

      <section className={cn(UX.section, "motion-item-in")} data-od-id="settings-security">
        <h3 className="text-base font-medium">Security</h3>
        <div className={cn("flex flex-col", UX.withinGroup)}>
        <div className="flex items-center gap-2">
          <Switch
            id="require-approval"
            checked={settings.security.requireApprovalForHighRiskCommands}
            onCheckedChange={(v) =>
              update({
                ...settings,
                security: {
                  ...settings.security,
                  requireApprovalForHighRiskCommands: v,
                },
              })
            }
          />
          <Label htmlFor="require-approval">Require approval for high-risk commands</Label>
        </div>
        <div className="mt-2 flex items-center gap-2">
          <Switch
            id="redact-secrets"
            checked={settings.security.redactSecrets}
            onCheckedChange={(v) =>
              update({
                ...settings,
                security: {
                  ...settings.security,
                  redactSecrets: v,
                },
              })
            }
          />
          <Label htmlFor="redact-secrets">Redact secrets in logs</Label>
        </div>
        </div>
      </section>

      <section className={cn(UX.section, "motion-item-in")} data-od-id="settings-voice">
        <h3 className="text-base font-medium">Voice</h3>
        <p className="text-sm text-muted-foreground">
          Optional OpenAI key for push-to-talk transcription in Quick Chat.
        </p>
        <div className={cn("flex flex-col", UX.withinGroup)}>
        <p className="mb-2 text-sm text-muted-foreground">
          {settings.voice.sttApiKeyConfigured
            ? "OpenAI API key stored for voice transcription only."
            : "Optional: add an OpenAI API key (sk-…) for push-to-talk transcription. Separate from the Cursor API key above."}
        </p>
        <SttApiKeyField configured={settings.voice.sttApiKeyConfigured ?? false} />
        <div className="mt-2 flex items-center gap-2">
          <Switch
            id="voice-confirm"
            checked={settings.voice.confirmBeforeSend}
            onCheckedChange={(v) =>
              update({
                ...settings,
                voice: { ...settings.voice, confirmBeforeSend: v },
              })
            }
          />
          <Label htmlFor="voice-confirm">Confirm before sending transcription</Label>
        </div>
        </div>
      </section>

      <section className={cn(UX.section, "motion-item-in")} data-od-id="settings-data">
        <h3 className="text-base font-medium">Data &amp; devices</h3>
        <div className={cn("flex flex-col", UX.withinGroup)}>
      <LocalDataSection />
      <MobileRemoteSection />
        </div>
      </section>

      <p className="border-t border-border pt-4 text-xs text-muted-foreground">
        Toggles and fields save automatically when you change them.
      </p>
    </div>
  );
}
