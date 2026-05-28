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

When risk is high (or medium if setting enabled):

1. Emit `approval.required` event
2. Pause session until user approves/rejects
3. UI shows Approve / Reject in Chat + Quick Bubble

MVP uses button confirmation — no MFA.

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

Stop flow:

1. Send graceful stop signal
2. Wait 3 seconds
3. Force kill process
4. Mark session `cancelled`
5. Save logs

All child PIDs tracked in process registry to prevent orphans.

## Settings

```typescript
security: {
  requireApprovalForHighRiskCommands: true;
  requireApprovalForMediumRiskCommands: false;
  redactSecrets: true;
  allowShellCommands: true;
}
```
