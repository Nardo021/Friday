import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  finishOnboarding,
  getSettings,
  saveCursorApiKey,
  saveSettings,
} from "@/lib/tauri";
import type { FridaySettings } from "@friday/agent-core";

export function OnboardingWindow() {
  const [apiKey, setApiKey] = useState("");
  const [settings, setSettings] = useState<FridaySettings | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void getSettings().then(setSettings);
  }, []);

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
      setError(e instanceof Error ? e.message : "Could not save settings");
    } finally {
      setBusy(false);
    }
  };

  if (!settings) {
    return (
      <div className="flex h-screen items-center justify-center bg-zinc-950 text-zinc-400">
        Loading…
      </div>
    );
  }

  return (
    <div className="flex h-screen flex-col bg-zinc-950 px-8 py-10 text-zinc-100">
      <div className="mb-6">
        <p className="text-xs font-medium uppercase tracking-wide text-indigo-400">
          First launch
        </p>
        <h1 className="mt-2 text-2xl font-semibold">Welcome to Friday</h1>
        <p className="mt-2 text-sm leading-relaxed text-zinc-400">
          Friday is your desktop companion for Cursor CLI. Connect your Cursor
          API key to enable cloud features, or skip and configure later in
          Settings.
        </p>
      </div>

      <div className="space-y-2">
        <label className="text-sm font-medium text-zinc-300" htmlFor="api-key">
          Cursor API key
        </label>
        <Input
          id="api-key"
          type="password"
          autoComplete="off"
          placeholder="key_… or sk-…"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          disabled={busy}
        />
        <p className="text-xs text-zinc-500">
          Encrypted in your OS credential store (Windows Credential Manager /
          macOS Keychain). Never sent to Friday servers.
        </p>
      </div>

      {error && <p className="mt-4 text-sm text-red-400">{error}</p>}

      <div className="mt-auto flex flex-col gap-2 pt-8">
        <Button
          disabled={busy || !apiKey.trim()}
          onClick={() => void complete(false)}
        >
          Save &amp; continue
        </Button>
        <Button
          variant="secondary"
          disabled={busy}
          onClick={() => void complete(true)}
        >
          Skip for now
        </Button>
      </div>
    </div>
  );
}
