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
