import { useEffect, useState } from "react";

import type { FridaySettings, MobileBridgeSettingsView } from "@friday/agent-core";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { saveCursorApiKey, clearCursorApiKey, clearLocalData, getLocalDataPath, saveSttApiKey, clearSttApiKey, getMobileBridgeSettings, updateMobileBridgeSettings, regenerateMobileBridgeToken } from "@/lib/tauri";
import { useSettingsStore } from "@/state/useSettingsStore";

function CursorApiKeyField({ configured }: { configured: boolean }) {
  const load = useSettingsStore((s) => s.load);
  const [value, setValue] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const save = async () => {
    if (!value.trim() || busy) return;
    setBusy(true);
    setMessage(null);
    try {
      await saveCursorApiKey(value.trim());
      setValue("");
      await load();
      setMessage("Saved.");
    } catch (e) {
      setMessage(e instanceof Error ? e.message : "Failed to save key");
    } finally {
      setBusy(false);
    }
  };

  const remove = async () => {
    if (busy || !configured) return;
    if (!window.confirm("Remove the stored Cursor API key from this device?")) return;
    setBusy(true);
    setMessage(null);
    try {
      await clearCursorApiKey();
      await load();
      setMessage("Removed.");
    } catch (e) {
      setMessage(e instanceof Error ? e.message : "Failed to remove key");
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-2">
      <Input
        type="password"
        placeholder={configured ? "Replace API key…" : "Paste Cursor API key…"}
        value={value}
        onChange={(e) => setValue(e.target.value)}
        disabled={busy}
      />
      <div className="flex gap-2">
        <Button size="sm" disabled={busy || !value.trim()} onClick={() => void save()}>
          Save API key
        </Button>
        {configured && (
          <Button size="sm" variant="secondary" disabled={busy} onClick={() => void remove()}>
            Remove API key
          </Button>
        )}
      </div>
      {message && <p className="text-xs text-zinc-400">{message}</p>}
    </div>
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
        <p className="text-sm text-zinc-400">Loading…</p>
      </section>
    );
  }

  return (
    <section>
      <h3 className="mb-2 font-medium">Mobile Remote</h3>
      <p className="mb-3 text-sm text-zinc-400">
        Allow the Friday iOS companion on your LAN to observe sessions, approve commands, and stop agents.
      </p>
      <label className="mb-3 flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={bridge.enabled}
          disabled={busy}
          onChange={(e) => void toggle(e.target.checked)}
        />
        Enable mobile bridge
      </label>
      <label className="mb-1 block text-sm text-zinc-400">Port</label>
      <Input
        type="number"
        className="mb-3 max-w-[8rem]"
        value={bridge.port}
        disabled={busy}
        onChange={(e) => void savePort(Number(e.target.value))}
      />
      <p className="mb-1 text-sm text-zinc-400">Local URL</p>
      <div className="mb-3 flex flex-wrap items-center gap-2">
        <code className="break-all rounded bg-zinc-900 px-2 py-1 text-xs">{bridge.localUrl}</code>
        <Button size="sm" variant="secondary" disabled={busy} onClick={() => void copyUrl()}>
          Copy URL
        </Button>
      </div>
      <p className="mb-1 text-sm text-zinc-400">Auth token</p>
      <div className="mb-3 flex flex-wrap items-center gap-2">
        <code className="break-all rounded bg-zinc-900 px-2 py-1 text-xs">{bridge.authToken}</code>
        <Button size="sm" variant="secondary" disabled={busy} onClick={() => void copyToken()}>
          Copy token
        </Button>
        <Button size="sm" variant="secondary" disabled={busy} onClick={() => void regenToken()}>
          Regenerate
        </Button>
      </div>
      <p className="text-xs text-zinc-500">
        In the mobile app, enter the URL and token above. Phone and desktop must be on the same Wi‑Fi.
        See <span className="text-zinc-400">docs/MOBILE_REMOTE.md</span> for API details.
      </p>
      {message && <p className="mt-2 text-xs text-zinc-400">{message}</p>}
    </section>
  );
}

function LocalDataSection() {
  const [path, setPath] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void getLocalDataPath().then(setPath);
  }, []);

  const wipe = async () => {
    if (
      busy ||
      !window.confirm(
        "Delete all local Friday data? This removes settings, sessions, projects, logs, and stored API keys. The app will restart.",
      )
    ) {
      return;
    }
    setBusy(true);
    try {
      await clearLocalData();
    } catch (e) {
      window.alert(e instanceof Error ? e.message : "Failed to clear data");
      setBusy(false);
    }
  };

  return (
    <section>
      <h3 className="mb-2 font-medium">Local data</h3>
      <p className="mb-2 text-sm text-zinc-400">
        Sessions, projects, and settings are stored on this device only.
      </p>
      {path && (
        <p className="mb-3 break-all font-mono text-xs text-zinc-500">{path}</p>
      )}
      <Button size="sm" variant="destructive" disabled={busy} onClick={() => void wipe()}>
        Delete all local data
      </Button>
      <p className="mt-2 text-xs text-zinc-500">
        Uninstalling via Windows installer also offers a checkbox to remove local
        data.
      </p>
    </section>
  );
}

function SttApiKeyField({ configured }: { configured: boolean }) {
  const load = useSettingsStore((s) => s.load);
  const [value, setValue] = useState("");
  const [busy, setBusy] = useState(false);

  const save = async () => {
    if (!value.trim() || busy) return;
    setBusy(true);
    try {
      await saveSttApiKey(value.trim());
      setValue("");
      await load();
    } finally {
      setBusy(false);
    }
  };

  const remove = async () => {
    if (busy || !configured) return;
    setBusy(true);
    try {
      await clearSttApiKey();
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex flex-wrap gap-2">
      <Input
        type="password"
        placeholder={configured ? "Replace STT key…" : "OpenAI API key…"}
        value={value}
        onChange={(e) => setValue(e.target.value)}
        disabled={busy}
        className="max-w-xs"
      />
      <Button size="sm" disabled={busy || !value.trim()} onClick={() => void save()}>
        Save STT key
      </Button>
      {configured && (
        <Button size="sm" variant="secondary" disabled={busy} onClick={() => void remove()}>
          Remove
        </Button>
      )}
    </div>
  );
}

function formatArgTemplates(templates: string[]): string {
  return JSON.stringify(templates);
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

  useEffect(() => {
    if (loaded) {
      setArgTemplateText(
        formatArgTemplates(settings.cursor.argTemplates.headlessStream),
      );
    }
  }, [loaded, settings.cursor.argTemplates.headlessStream]);

  if (!loaded) return <div className="text-zinc-400">Loading settings...</div>;

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
    <div className="max-w-xl space-y-6">
      <section>
        <h3 className="mb-2 font-medium">Appearance</h3>
        <label className="mb-2 block text-sm text-zinc-400">Pet scale</label>
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
      </section>

      <section>
        <h3 className="mb-2 font-medium">Security</h3>
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={settings.security.requireApprovalForHighRiskCommands}
            onChange={(e) =>
              update({
                ...settings,
                security: {
                  ...settings.security,
                  requireApprovalForHighRiskCommands: e.target.checked,
                },
              })
            }
          />
          Require approval for high-risk commands
        </label>
        <label className="mt-2 flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={settings.security.redactSecrets}
            onChange={(e) =>
              update({
                ...settings,
                security: {
                  ...settings.security,
                  redactSecrets: e.target.checked,
                },
              })
            }
          />
          Redact secrets in logs
        </label>
      </section>

      <section>
        <h3 className="mb-2 font-medium">Cursor API</h3>
        <p className="mb-2 text-sm text-zinc-400">
          {settings.cursor.apiKeyConfigured
            ? "API key saved in OS secure storage on this device."
            : "No API key configured yet."}
        </p>
        <p className="mb-2 text-xs text-zinc-500">
          Session messages and event logs are encrypted at rest with a key stored in the same OS
          credential store.
        </p>
        <CursorApiKeyField configured={settings.cursor.apiKeyConfigured ?? false} />
      </section>

      <section>
        <h3 className="mb-2 font-medium">Cloud Agent</h3>
        <p className="mb-2 text-sm text-zinc-400">
          Cloud agents run on Cursor infrastructure and can open pull requests.
        </p>
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={settings.cloud?.autoCreatePr ?? true}
            onChange={(e) =>
              update({
                ...settings,
                cloud: {
                  autoCreatePr: e.target.checked,
                  model: settings.cloud?.model,
                },
              })
            }
          />
          Auto-create pull request when cloud run completes
        </label>
        <label className="mt-2 block text-sm text-zinc-400">Model (optional)</label>
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
      </section>

      <section>
        <h3 className="mb-2 font-medium">General</h3>
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={settings.behavior.launchAtStartup}
            onChange={(e) =>
              update({
                ...settings,
                behavior: {
                  ...settings.behavior,
                  launchAtStartup: e.target.checked,
                },
              })
            }
          />
          Launch Friday at startup
        </label>
      </section>

      <section>
        <h3 className="mb-2 font-medium">Voice (Cloud STT)</h3>
        <p className="mb-2 text-sm text-zinc-400">
          {settings.voice.sttApiKeyConfigured
            ? "OpenAI Whisper API key stored in OS secure storage."
            : "Configure an OpenAI API key for voice transcription."}
        </p>
        <SttApiKeyField configured={settings.voice.sttApiKeyConfigured ?? false} />
        <label className="mt-2 flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={settings.voice.confirmBeforeSend}
            onChange={(e) =>
              update({
                ...settings,
                voice: { ...settings.voice, confirmBeforeSend: e.target.checked },
              })
            }
          />
          Confirm before sending transcription
        </label>
      </section>

      <section>
        <h3 className="mb-2 font-medium">Keyboard shortcuts</h3>
        <ul className="space-y-1 text-sm text-zinc-400">
          <li>Quick Bubble: {settings.shortcuts.quickBubble}</li>
          <li>Panel: {settings.shortcuts.openPanel}</li>
          <li>Voice: {settings.shortcuts.voiceInput}</li>
          <li>Stop session: {settings.shortcuts.stopSession}</li>
        </ul>
      </section>

      <LocalDataSection />

      <MobileRemoteSection />

      <section>
        <h3 className="mb-2 font-medium">Cursor CLI</h3>
        <Input
          className="mb-3"
          placeholder="Executable path (optional)"
          value={settings.cursor.executablePath ?? ""}
          onChange={(e) =>
            patchCursor({
              executablePath: e.target.value || undefined,
            })
          }
        />
        <label className="mb-3 flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={settings.cursor.usePty}
            onChange={(e) => patchCursor({ usePty: e.target.checked })}
          />
          Use PTY for agent process
        </label>
        <div className="mb-3 grid grid-cols-2 gap-3">
          <div>
            <label className="mb-1 block text-sm text-zinc-400">Terminal cols</label>
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
            <label className="mb-1 block text-sm text-zinc-400">Terminal rows</label>
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
        <label className="mb-1 block text-sm text-zinc-400">
          Headless stream arg templates (comma-separated or JSON array)
        </label>
        <Input
          className="mb-2 font-mono text-xs"
          value={argTemplateText}
          onChange={(e) => setArgTemplateText(e.target.value)}
        />
        {argError && <p className="mb-2 text-xs text-red-400">{argError}</p>}
        <Button size="sm" variant="secondary" onClick={saveArgTemplates}>
          Apply arg templates
        </Button>
      </section>

      <Button onClick={() => update(settings)}>Save Settings</Button>
    </div>
  );
}
