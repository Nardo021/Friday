# Friday Security

## Project Allowlist

Only directories added as **Projects** may run agents.

- Allowed: user-added project paths
- Blocked: home root, system dirs, unregistered paths

Enforced in `project_allowlist.rs` before session start.

## Command Risk Classification

| Risk | Examples |
|---|---|
| **Low** | `pnpm test`, `git status`, `ls`, `cat` |
| **Medium** | `pnpm install`, `git checkout`, `git pull` |
| **High** | `rm -rf`, `git reset --hard`, `curl \| bash`, `.env` edits, mass deletes |

Classifier: `risk_classifier.rs` + patterns in `command_policy.rs`.

## Approval Layer

When a high-risk (or medium, if enabled) shell command is observed in the Cursor CLI stream:

1. Emit `approval.required` (non-blocking — the CLI may already be running the command)
2. Mark session `waiting_permission` in the UI
3. **Acknowledge** clears the approval card; **Stop session** kills the Friday-owned agent

This is observe-and-stop, not a Cursor permission gate. MVP uses button confirmation — no MFA.

## Secret Redaction

Before persisting or displaying logs:

- `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `CURSOR_TOKEN`, `GITHUB_TOKEN`
- `DATABASE_URL`, generic `*_TOKEN`, `*_SECRET`, `sk-*` patterns

Replaced with `[REDACTED]` via `secret_redactor.rs`.

## Local Data Storage

All Friday data stays on the user's machine.

| Data | Location | Protection |
|---|---|---|
| Cursor API key | OS credential store (`keyring` crate) | Windows Credential Manager / macOS Keychain / Linux Secret Service |
| Data encryption key | OS credential store (`Friday/friday_data_key`) | AES-256-GCM key for message/event payloads |
| Settings, sessions, projects | `%APPDATA%\com.leo.friday\friday.db` (Windows) | User profile ACL; API key not in SQLite |
| Message & event payloads | SQLite `messages` / `session_events` | AES-256-GCM (`enc:v1:` prefix); key in credential store |
| Session logs | `%APPDATA%\com.leo.friday\logs\` | Same as app data directory |
Legacy installs used `%APPDATA%\Friday\`; data is migrated to `com.leo.friday` on startup.

API keys previously saved in SQLite are migrated to the OS credential store automatically.

### In-app wipe

Command Center → Settings → **Delete all local data** writes a wipe marker, clears credential-store entries, restarts, then deletes the data directory on next launch (avoids SQLite file lock on Windows).
### Uninstaller (Windows NSIS)

During uninstall, check **删除本地数据（设置、会话、API 密钥等）** to remove:

- `%APPDATA%\com.leo.friday\` (Tauri default)
- Legacy `%APPDATA%\Friday\`
- Cursor API key in Windows Credential Manager (`Friday/cursor_api_key`)
- Data encryption key (`Friday/friday_data_key`)
Configured in `bundle/windows/nsis-hooks.nsh`.

## Process Kill Switch

Stop flow (PTY / owned CLI):

1. Send Ctrl+C (SIGINT) into the PTY
2. Wait up to 3 seconds
3. Force kill if still alive
4. Mark session `stopped`
5. Save logs

Child PIDs are recorded on the session and excluded from external `cursor-agent` discovery to avoid duplicate observe sessions.

## Settings

```typescript
security: {
  requireApprovalForHighRiskCommands: true;
  requireApprovalForMediumRiskCommands: false;
  redactSecrets: true;
  allowShellCommands: true;
}
```
