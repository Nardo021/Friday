# Agent Adapter Specification

## AgentAdapter Interface

Every coding agent integrates via a common adapter contract.

### Required Methods

| Method | Description |
|---|---|
| `id` | Unique adapter id (e.g. `cursor-cli`) |
| `name` | Display name |
| `capabilities` | Feature flags for UI |
| `start_session` | Spawn agent with prompt + cwd |
| `send_message` | Send follow-up (if supported) |
| `stop_session` | Graceful then force stop |
| `on_event` | Callback for normalized `AgentEvent` |

### Optional Methods

| Method | Description |
|---|---|
| `approve` | Approve pending command/tool |
| `reject` | Reject pending command/tool |

## AgentCapabilities

```typescript
{
  supportsStreaming: boolean;
  supportsInteractiveInput: boolean;
  supportsApprovals: boolean | "partial";
  supportsFileChangeEvents: boolean | "parsed";
  supportsCommandEvents: boolean | "parsed";
  supportsSessionResume: boolean;
  supportsStop: boolean;
}
```

## Cursor CLI Adapter (v1)

**Executable:** `cursor-agent` (configurable path)

**Default command:**

```bash
cursor-agent -p --output-format stream-json --stream-partial-output "<prompt>"
```

**Parser rules:**

1. Each stdout line → try JSON parse
2. `assistant` + `timestamp_ms` → streaming text delta
3. `assistant` without `timestamp_ms` → skip (buffered duplicate)
4. `tool_use` / tool events → `tool.started` / `tool.completed`
5. `result` → `session.completed` with final text
6. Non-JSON lines → plain-text fallback → `agent.status` or `agent.message`

**Event mapping:**

| Cursor | Friday Event |
|---|---|
| File read | `agent.status: reading` |
| File edit | `file.changed` + `agent.status: editing` |
| Shell command | `command.started` / `command.completed` |
| Assistant text | `agent.message` |
| Done | `session.completed` |
| Error | `session.error` |

## Future Adapters (stub in registry)

- `claude-code`
- `codex-cli`
- `gemini-cli`
- `piecez-agent`
- `mono-agent`

UI must only depend on `AgentEvent` and `AgentCapabilities`, never adapter-specific output formats.
