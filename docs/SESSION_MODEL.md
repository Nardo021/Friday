# Friday Session Model

Friday is organized around **Agent Sessions**, not raw CLI output.

## Session Types

| Type | Ownership | Control | Description |
|---|---|---|---|
| `external_cli` | external | observe | Discovered `cursor-agent` process; no kill/input |
| `friday_owned_cli` | friday | full | Launched by Friday via CursorCliLauncher + PTY |
| `cursor_sdk_local` | friday | full | v2 — Cursor SDK worker (stub in v1) |
| `cursor_cloud` | friday | full | v2 — Cloud Agent API (stub in v1) |

## FridaySession

Core fields: `id`, `title`, `type`, `ownership`, `adapterId`, `status`, `controlLevel`, `repo`, `process`, `cloud`, timestamps.

### Status lifecycle

```
discovered → starting → thinking → {reading|editing|running_command|waiting_permission|testing}
  → done | error | stopped
```

External sessions may stay at `discovered` with limited status inference.

## Multi-Session Rules (v1)

- Multiple **external_cli** observe sessions allowed
- At most **one** running `friday_owned_cli` session
- `activeSessionId` drives Panel focus and Pet summary

## Safe Close

| Type | Behavior |
|---|---|
| external_cli | Detach observer only — never kill process |
| friday_owned_cli | Ctrl+C / SIGINT → wait 3s → kill → persist logs |
| cloud / sdk | v2+ |

## Events

All adapters emit unified `AgentEvent` (see `packages/agent-core/src/events.ts`). UI never parses Cursor CLI directly.
